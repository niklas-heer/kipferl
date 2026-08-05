use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use kipferl_format::{DecodeError, LayoutError, TRAILER_SIZE, Trailer};
use md5::{Digest, Md5};

const HASH_SAMPLE_SIZE: u64 = 1024;
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleMetadata {
    pub trailer: Trailer,
    pub content_hash: [u8; 8],
}

impl BundleMetadata {
    #[must_use]
    pub fn cache_key(self) -> String {
        let mut key = String::with_capacity(16);
        for byte in self.content_hash {
            write!(&mut key, "{byte:02x}").expect("writing to a String cannot fail");
        }
        key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedBundle {
    pub cache_dir: PathBuf,
    pub runtime_path: PathBuf,
    pub python_path: PathBuf,
    pub cache_hit: bool,
}

#[derive(Debug)]
pub enum LoaderError {
    Io(io::Error),
    Decode(DecodeError),
    Layout(LayoutError),
}

impl fmt::Display for LoaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Decode(error) => write!(formatter, "invalid universal trailer: {error}"),
            Self::Layout(error) => write!(formatter, "invalid universal layout: {error}"),
        }
    }
}

impl Error for LoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::Layout(error) => Some(error),
        }
    }
}

impl From<io::Error> for LoaderError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<DecodeError> for LoaderError {
    fn from(error: DecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<LayoutError> for LoaderError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}

pub fn inspect<R: Read + Seek>(
    reader: &mut R,
    file_size: u64,
) -> Result<BundleMetadata, LoaderError> {
    if file_size < TRAILER_SIZE as u64 {
        return Err(LayoutError::FileTooSmall.into());
    }

    reader.seek(SeekFrom::Start(file_size - TRAILER_SIZE as u64))?;
    let mut trailer_bytes = [0; TRAILER_SIZE];
    reader.read_exact(&mut trailer_bytes)?;
    let trailer = Trailer::decode(&trailer_bytes)?;
    trailer.validate_layout(file_size)?;
    let content_hash = calculate_content_hash(reader, trailer)?;

    Ok(BundleMetadata {
        trailer,
        content_hash,
    })
}

pub fn prepare_path(bundle_path: &Path, cache_root: &Path) -> Result<PreparedBundle, LoaderError> {
    let mut bundle = File::open(bundle_path)?;
    let file_size = bundle.metadata()?.len();
    let metadata = inspect(&mut bundle, file_size)?;
    let cache_dir = cache_root.join(format!("kipferl-{}", metadata.cache_key()));
    let runtime_path = cache_dir.join("m");
    let python_path = cache_dir.join("a.py");

    ensure_cache_directory(&cache_dir)?;
    let cache_hit = cache_is_valid(metadata.trailer, &runtime_path, &python_path)?;

    if !cache_hit {
        write_payload_atomically(
            &mut bundle,
            metadata.trailer.runtime_offset,
            metadata.trailer.runtime_size,
            &runtime_path,
            0o755,
        )?;
        write_payload_atomically(
            &mut bundle,
            metadata.trailer.python_offset,
            metadata.trailer.python_size,
            &python_path,
            0o600,
        )?;
    }

    Ok(PreparedBundle {
        cache_dir,
        runtime_path,
        python_path,
        cache_hit,
    })
}

pub fn calculate_content_hash<R: Read + Seek>(
    reader: &mut R,
    trailer: Trailer,
) -> Result<[u8; 8], io::Error> {
    let mut hasher = Md5::new();
    hash_sample(
        reader,
        trailer.runtime_offset,
        trailer.runtime_size,
        &mut hasher,
    )?;
    hash_sample(
        reader,
        trailer.python_offset,
        trailer.python_size,
        &mut hasher,
    )?;
    let digest = hasher.finalize();
    let mut short_hash = [0; 8];
    short_hash.copy_from_slice(&digest[..8]);
    Ok(short_hash)
}

fn hash_sample<R: Read + Seek>(
    reader: &mut R,
    offset: u64,
    size: u64,
    hasher: &mut Md5,
) -> Result<(), io::Error> {
    reader.seek(SeekFrom::Start(offset))?;
    let sample_size = size.min(HASH_SAMPLE_SIZE) as usize;
    let mut sample = [0; HASH_SAMPLE_SIZE as usize];
    reader.read_exact(&mut sample[..sample_size])?;
    hasher.update(&sample[..sample_size]);
    Ok(())
}

fn ensure_cache_directory(path: &Path) -> Result<(), io::Error> {
    let mut builder = DirBuilder::new();
    builder.recursive(true).mode(0o700).create(path)?;
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cache path is not a directory",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn cache_is_valid(
    trailer: Trailer,
    runtime_path: &Path,
    python_path: &Path,
) -> Result<bool, io::Error> {
    if !cached_file_has_size(runtime_path, trailer.runtime_size)? {
        return Ok(false);
    }
    if fs::metadata(runtime_path)?.permissions().mode() & 0o111 == 0 {
        return Ok(false);
    }

    cached_file_has_size(python_path, trailer.python_size)
}

fn cached_file_has_size(path: &Path, expected_size: u64) -> Result<bool, io::Error> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    Ok(metadata.is_file() && metadata.len() == expected_size)
}

fn write_payload_atomically<R: Read + Seek>(
    source: &mut R,
    source_offset: u64,
    size: u64,
    destination: &Path,
    mode: u32,
) -> Result<(), io::Error> {
    let (temporary_path, mut temporary) = create_temporary_file(destination, mode)?;
    let result = (|| {
        source.seek(SeekFrom::Start(source_offset))?;
        let copied = io::copy(&mut source.take(size), &mut temporary)?;
        if copied != size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("payload ended after {copied} of {size} bytes"),
            ));
        }
        temporary.flush()?;
        temporary.set_permissions(fs::Permissions::from_mode(mode))?;
        drop(temporary);
        fs::rename(&temporary_path, destination)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn create_temporary_file(destination: &Path, mode: u32) -> Result<(PathBuf, File), io::Error> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "cache path has no parent"))?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid cache filename"))?;

    loop {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".{name}.{}.{}", std::process::id(), counter));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::os::unix::fs::symlink;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::*;

    const RUNTIME: &[u8] = include_bytes!("../tests/fixtures/cache_runtime_v1.txt");
    const PYTHON: &[u8] = include_bytes!("../tests/fixtures/cache_python_v1.py");

    #[test]
    fn reads_metadata_and_preserves_the_zig_cache_key() {
        let bytes = bundle_bytes(b"loader-stub", RUNTIME, PYTHON);
        let mut reader = Cursor::new(&bytes);
        let metadata = inspect(&mut reader, bytes.len() as u64).expect("inspect bundle");

        assert_eq!(
            metadata.content_hash,
            decode_hash(include_str!("../tests/fixtures/cache_hash_v1.hex"))
        );
        assert_eq!(metadata.cache_key(), "6695b943da86e0e4");
        assert_eq!(
            metadata.trailer,
            trailer_for(b"loader-stub", RUNTIME, PYTHON)
        );
    }

    #[test]
    fn rejects_small_corrupt_overlapping_and_truncated_bundles() {
        let mut small = Cursor::new([0; 47]);
        assert!(matches!(
            inspect(&mut small, 47),
            Err(LoaderError::Layout(LayoutError::FileTooSmall))
        ));

        let mut corrupt = bundle_bytes(b"stub", RUNTIME, PYTHON);
        let trailer_start = corrupt.len() - TRAILER_SIZE;
        corrupt[trailer_start] = 0;
        let mut reader = Cursor::new(&corrupt);
        assert!(matches!(
            inspect(&mut reader, corrupt.len() as u64),
            Err(LoaderError::Decode(DecodeError::InvalidLeadingMagic))
        ));

        let mut overlap = bundle_bytes(b"stub", RUNTIME, PYTHON);
        let mut trailer = trailer_for(b"stub", RUNTIME, PYTHON);
        trailer.runtime_size += 1;
        replace_trailer(&mut overlap, trailer);
        let mut reader = Cursor::new(&overlap);
        assert!(matches!(
            inspect(&mut reader, overlap.len() as u64),
            Err(LoaderError::Layout(LayoutError::RuntimeOverlapsPython))
        ));

        let mut truncated = bundle_bytes(b"stub", RUNTIME, PYTHON);
        let mut trailer = trailer_for(b"stub", RUNTIME, PYTHON);
        trailer.python_size += 1;
        replace_trailer(&mut truncated, trailer);
        let mut reader = Cursor::new(&truncated);
        assert!(matches!(
            inspect(&mut reader, truncated.len() as u64),
            Err(LoaderError::Layout(LayoutError::PayloadOverlapsTrailer))
        ));
    }

    #[test]
    fn extracts_reuses_and_repairs_the_cache_with_paths_containing_spaces() {
        let temporary = TestDirectory::new("cache with spaces");
        let bundle_path = temporary.path.join("bundle with spaces");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");

        let first = prepare_path(&bundle_path, &temporary.path).expect("first extraction");
        assert!(!first.cache_hit);
        assert_eq!(
            fs::read(&first.runtime_path).expect("read runtime"),
            RUNTIME
        );
        assert_eq!(fs::read(&first.python_path).expect("read Python"), PYTHON);
        assert_ne!(
            fs::metadata(&first.runtime_path)
                .expect("runtime metadata")
                .permissions()
                .mode()
                & 0o111,
            0
        );

        let second = prepare_path(&bundle_path, &temporary.path).expect("cache hit");
        assert!(second.cache_hit);

        fs::write(&second.runtime_path, b"truncated").expect("truncate cache");
        let repaired = prepare_path(&bundle_path, &temporary.path).expect("repair cache");
        assert!(!repaired.cache_hit);
        assert_eq!(
            fs::read(repaired.runtime_path).expect("read repair"),
            RUNTIME
        );

        let changed_bundle_path = temporary.path.join("changed bundle");
        let mut changed_runtime = RUNTIME.to_vec();
        changed_runtime[0] ^= 1;
        fs::write(
            &changed_bundle_path,
            bundle_bytes(b"stub", &changed_runtime, PYTHON),
        )
        .expect("write changed bundle");
        let changed = prepare_path(&changed_bundle_path, &temporary.path).expect("changed hash");
        assert!(!changed.cache_hit);
        assert_ne!(changed.cache_dir, repaired.cache_dir);
    }

    #[test]
    fn concurrent_extraction_never_exposes_partial_payloads() {
        let temporary = TestDirectory::new("concurrent");
        let bundle_path = temporary.path.join("bundle");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");
        let barrier = Arc::new(Barrier::new(8));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let bundle_path = bundle_path.clone();
                let cache_root = temporary.path.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let prepared = prepare_path(&bundle_path, &cache_root).expect("prepare");
                    assert_eq!(fs::read(prepared.runtime_path).expect("runtime"), RUNTIME);
                    assert_eq!(fs::read(prepared.python_path).expect("Python"), PYTHON);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("worker thread");
        }
    }

    #[test]
    fn rejects_a_symlinked_cache_directory() {
        let temporary = TestDirectory::new("symlink");
        let bundle_path = temporary.path.join("bundle");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");
        let metadata = {
            let bytes = fs::read(&bundle_path).expect("read bundle");
            inspect(&mut Cursor::new(&bytes), bytes.len() as u64).expect("inspect")
        };
        let destination = temporary.path.join("destination");
        fs::create_dir(&destination).expect("create destination");
        symlink(
            &destination,
            temporary
                .path
                .join(format!("kipferl-{}", metadata.cache_key())),
        )
        .expect("create cache symlink");

        assert!(matches!(
            prepare_path(&bundle_path, &temporary.path),
            Err(LoaderError::Io(error)) if error.kind() == io::ErrorKind::InvalidData
        ));
    }

    fn bundle_bytes(stub: &[u8], runtime: &[u8], python: &[u8]) -> Vec<u8> {
        let trailer = trailer_for(stub, runtime, python);
        [stub, runtime, python, &trailer.encode()].concat()
    }

    fn trailer_for(stub: &[u8], runtime: &[u8], python: &[u8]) -> Trailer {
        Trailer {
            runtime_offset: stub.len() as u64,
            runtime_size: runtime.len() as u64,
            python_offset: (stub.len() + runtime.len()) as u64,
            python_size: python.len() as u64,
        }
    }

    fn replace_trailer(bundle: &mut [u8], trailer: Trailer) {
        let start = bundle.len() - TRAILER_SIZE;
        bundle[start..].copy_from_slice(&trailer.encode());
    }

    fn decode_hash(hex: &str) -> [u8; 8] {
        let hex = hex.trim().as_bytes();
        assert_eq!(hex.len(), 16);
        let mut hash = [0; 8];
        for (index, byte) in hash.iter_mut().enumerate() {
            *byte = hex_nibble(hex[index * 2]) << 4 | hex_nibble(hex[index * 2 + 1]);
        }
        hash
    }

    fn hex_nibble(byte: u8) -> u8 {
        match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => panic!("invalid hash fixture"),
        }
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "kipferl-loader-test-{}-{counter}-{name}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
