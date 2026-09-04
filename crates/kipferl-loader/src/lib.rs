#![deny(clippy::pedantic, clippy::nursery)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::unreachable,
        clippy::string_slice,
        clippy::panic_in_result_fn,
        clippy::panic,
        clippy::exit,
        clippy::as_conversions
    )
)]

use std::error::Error;
use std::fmt;
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
        format!("{:016x}", u64::from_be_bytes(self.content_hash))
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

/// Decode and validate a bundle before sampling its payloads.
///
/// # Errors
/// Returns a decoding/layout error for an invalid container, or an I/O error
/// when seeking or reading its trailer and payload samples fails.
pub fn inspect<R: Read + Seek>(
    reader: &mut R,
    file_size: u64,
) -> Result<BundleMetadata, LoaderError> {
    let trailer_size = u64::try_from(TRAILER_SIZE).map_err(|_| LayoutError::IntegerOverflow)?;
    let trailer_offset = file_size
        .checked_sub(trailer_size)
        .ok_or(LayoutError::FileTooSmall)?;
    reader.seek(SeekFrom::Start(trailer_offset))?;
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

/// Validate an executable and prepare complete cached payloads for execution.
///
/// # Errors
/// Returns a decoding/layout error for an invalid executable, or an I/O error
/// when opening the bundle, validating the cache, or replacing payloads fails.
pub fn prepare_path(bundle_path: &Path, cache_root: &Path) -> Result<PreparedBundle, LoaderError> {
    let mut bundle = File::open(bundle_path)?;
    let file_size = bundle.metadata()?.len();
    let metadata = inspect(&mut bundle, file_size)?;
    let cache_dir = cache_root.join(format!("kipferl-{}", metadata.cache_key()));
    let runtime_path = cache_dir.join("m");
    let python_path = cache_dir.join("a.py");

    ensure_cache_directory(&cache_dir)?;
    let cache_hit = cache_is_valid(&mut bundle, metadata.trailer, &runtime_path, &python_path)?;

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

/// Compute the legacy cache identifier from bounded payload samples.
///
/// This identifier is not an integrity check; cache reuse compares full payloads.
///
/// # Errors
/// Returns an I/O error if a requested sample cannot be read completely.
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
    for (destination, source) in short_hash.iter_mut().zip(digest.iter()) {
        *destination = *source;
    }
    Ok(short_hash)
}

fn hash_sample<R: Read + Seek>(
    reader: &mut R,
    offset: u64,
    size: u64,
    hasher: &mut Md5,
) -> Result<(), io::Error> {
    reader.seek(SeekFrom::Start(offset))?;
    let sample_size = usize::try_from(size.min(HASH_SAMPLE_SIZE))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "hash sample length overflow"))?;
    let mut sample = [0; 1024];
    let sample = sample
        .get_mut(..sample_size)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "hash sample exceeds buffer"))?;
    reader.read_exact(sample)?;
    hasher.update(sample);
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

fn cache_is_valid<R: Read + Seek>(
    bundle: &mut R,
    trailer: Trailer,
    runtime_path: &Path,
    python_path: &Path,
) -> Result<bool, io::Error> {
    if !cached_file_matches(
        bundle,
        runtime_path,
        trailer.runtime_offset,
        trailer.runtime_size,
        true,
    )? {
        return Ok(false);
    }

    cached_file_matches(
        bundle,
        python_path,
        trailer.python_offset,
        trailer.python_size,
        false,
    )
}

fn cached_file_matches<R: Read + Seek>(
    bundle: &mut R,
    path: &Path,
    offset: u64,
    expected_size: u64,
    executable: bool,
) -> Result<bool, io::Error> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() != expected_size {
        return Ok(false);
    }
    let required_permissions = if executable { 0o500 } else { 0o400 };
    if metadata.permissions().mode() & required_permissions != required_permissions {
        return Ok(false);
    }

    // The legacy cache key samples only the first 1 KiB of each payload.
    // Preserve that key, but compare all bytes before trusting cached code:
    // equal sizes and sampled hashes do not imply equal contents.
    let mut cached = match File::open(path) {
        Ok(cached) => cached,
        // Cache files are disposable. Restore unreadable files (including
        // ACL-denied files), or files removed since the metadata check.
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::NotFound
            ) =>
        {
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    bundle.seek(SeekFrom::Start(offset))?;
    let mut remaining = expected_size;
    // Cache checks can be called from small-stack worker threads. Keep the
    // two 64 KiB buffers on the heap rather than consuming 128 KiB of stack.
    let mut bundled_bytes = vec![0; 65_536];
    let mut cached_bytes = vec![0; 65_536];
    while remaining != 0 {
        let chunk_size = remaining.min(65_536);
        let length = usize::try_from(chunk_size).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "cache chunk length overflow")
        })?;
        let bundled = bundled_bytes.get_mut(..length).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "bundle chunk exceeds buffer")
        })?;
        let cached_chunk = cached_bytes.get_mut(..length).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "cache chunk exceeds buffer")
        })?;
        bundle.read_exact(bundled)?;
        if let Err(error) = cached.read_exact(cached_chunk) {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(false);
            }
            return Err(error);
        }
        if bundled != cached_chunk {
            return Ok(false);
        }
        remaining = remaining.checked_sub(chunk_size).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "cache chunk exceeds payload")
        })?;
    }
    let mut trailing = [0; 1];
    Ok(cached.read(&mut trailing)? == 0)
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
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
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
    fn validates_cached_payloads_on_a_small_worker_stack() {
        let directory = TestDirectory::new("small worker stack");
        let bundle_path = directory.path.join("bundle");
        let cache_root = directory.path.join("cache");
        let bytes = bundle_bytes(b"loader", &vec![42; 131_072], PYTHON);
        fs::write(&bundle_path, bytes).expect("write bundle");
        prepare_path(&bundle_path, &cache_root).expect("populate cache");
        thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || {
                assert!(
                    prepare_path(&bundle_path, &cache_root)
                        .expect("validate cache")
                        .cache_hit
                );
            })
            .expect("start small-stack worker")
            .join()
            .expect("worker completes without overflowing its stack");
    }

    #[test]
    fn reads_metadata_and_preserves_the_zig_cache_key() {
        let bytes = bundle_bytes(b"loader-stub", RUNTIME, PYTHON);
        let mut reader = Cursor::new(&bytes);
        let metadata = inspect(
            &mut reader,
            u64::try_from(bytes.len()).expect("fixture length fits u64"),
        )
        .expect("inspect bundle");

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
            inspect(
                &mut reader,
                u64::try_from(corrupt.len()).expect("fixture length fits u64")
            ),
            Err(LoaderError::Decode(DecodeError::InvalidLeadingMagic))
        ));

        let mut overlap = bundle_bytes(b"stub", RUNTIME, PYTHON);
        let mut trailer = trailer_for(b"stub", RUNTIME, PYTHON);
        trailer.runtime_size += 1;
        replace_trailer(&mut overlap, trailer);
        let mut reader = Cursor::new(&overlap);
        assert!(matches!(
            inspect(
                &mut reader,
                u64::try_from(overlap.len()).expect("fixture length fits u64")
            ),
            Err(LoaderError::Layout(LayoutError::RuntimeOverlapsPython))
        ));

        let mut truncated = bundle_bytes(b"stub", RUNTIME, PYTHON);
        let mut trailer = trailer_for(b"stub", RUNTIME, PYTHON);
        trailer.python_size += 1;
        replace_trailer(&mut truncated, trailer);
        let mut reader = Cursor::new(&truncated);
        assert!(matches!(
            inspect(
                &mut reader,
                u64::try_from(truncated.len()).expect("fixture length fits u64")
            ),
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
    fn repairs_same_length_corruption_beyond_the_hash_sample() {
        let temporary = TestDirectory::new("corrupted payload");
        let bundle_path = temporary.path.join("bundle");
        let runtime = vec![b'r'; 150_000];
        let python = vec![b'p'; 150_000];
        fs::write(&bundle_path, bundle_bytes(b"stub", &runtime, &python)).expect("write bundle");
        let prepared = prepare_path(&bundle_path, &temporary.path).expect("extract bundle");

        for (path, original) in [
            (&prepared.runtime_path, &runtime),
            (&prepared.python_path, &python),
        ] {
            let mut corrupted = original.clone();
            corrupted[149_999] ^= 1;
            fs::write(path, corrupted).expect("corrupt cached payload");
            let repaired = prepare_path(&bundle_path, &temporary.path).expect("repair cache");
            assert!(!repaired.cache_hit);
            assert_eq!(fs::read(path).expect("read repaired payload"), *original);
            assert!(
                prepare_path(&bundle_path, &temporary.path)
                    .expect("reuse repair")
                    .cache_hit
            );
        }
    }

    #[test]
    fn distinguishes_bundles_with_the_same_legacy_cache_key_and_size() {
        let temporary = TestDirectory::new("sample collision");
        let original_path = temporary.path.join("original bundle");
        let changed_path = temporary.path.join("changed bundle");
        let original_python = vec![b'p'; 2048];
        let mut changed_python = original_python.clone();
        changed_python[2047] = b'q';
        fs::write(
            &original_path,
            bundle_bytes(b"stub", RUNTIME, &original_python),
        )
        .expect("write original bundle");
        fs::write(
            &changed_path,
            bundle_bytes(b"stub", RUNTIME, &changed_python),
        )
        .expect("write changed bundle");

        let original = prepare_path(&original_path, &temporary.path).expect("extract original");
        let changed = prepare_path(&changed_path, &temporary.path).expect("extract changed");
        assert_eq!(original.cache_dir, changed.cache_dir);
        assert!(!changed.cache_hit);
        assert_eq!(
            fs::read(&changed.python_path).expect("read changed Python"),
            changed_python
        );
        let restored = prepare_path(&original_path, &temporary.path).expect("restore original");
        assert!(!restored.cache_hit);
        assert_eq!(
            fs::read(&restored.python_path).expect("read original Python"),
            original_python
        );
    }

    #[test]
    fn replaces_symlinked_payloads_without_modifying_their_targets() {
        let temporary = TestDirectory::new("payload symlinks");
        let bundle_path = temporary.path.join("bundle");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");
        let prepared = prepare_path(&bundle_path, &temporary.path).expect("extract bundle");

        for (path, original) in [
            (&prepared.runtime_path, RUNTIME),
            (&prepared.python_path, PYTHON),
        ] {
            let target = temporary.path.join("symlink target");
            fs::write(&target, original).expect("write symlink target");
            fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("target mode");
            fs::remove_file(path).expect("remove cached payload");
            symlink(&target, path).expect("symlink cached payload");

            let repaired = prepare_path(&bundle_path, &temporary.path).expect("repair symlink");
            assert!(!repaired.cache_hit);
            assert!(
                fs::symlink_metadata(path)
                    .expect("payload metadata")
                    .is_file()
            );
            assert_eq!(fs::read(path).expect("read payload"), original);
            assert_eq!(fs::read(&target).expect("read symlink target"), original);
        }
    }

    #[test]
    fn repairs_a_runtime_without_owner_execute_permission() {
        let temporary = TestDirectory::new("runtime permissions");
        let bundle_path = temporary.path.join("bundle");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");
        let prepared = prepare_path(&bundle_path, &temporary.path).expect("extract bundle");
        fs::set_permissions(&prepared.runtime_path, fs::Permissions::from_mode(0o611))
            .expect("remove owner execute permission");

        let repaired = prepare_path(&bundle_path, &temporary.path).expect("repair permissions");
        assert!(!repaired.cache_hit);
        assert_ne!(
            fs::metadata(&repaired.runtime_path)
                .expect("runtime metadata")
                .permissions()
                .mode()
                & 0o100,
            0
        );
    }

    #[test]
    fn repairs_unreadable_cached_payloads() {
        let temporary = TestDirectory::new("unreadable payloads");
        let bundle_path = temporary.path.join("bundle");
        fs::write(&bundle_path, bundle_bytes(b"stub", RUNTIME, PYTHON)).expect("write bundle");
        let prepared = prepare_path(&bundle_path, &temporary.path).expect("extract bundle");

        for (path, mode, restored_mode, original) in [
            (&prepared.runtime_path, 0o111, 0o755, RUNTIME),
            (&prepared.python_path, 0o000, 0o600, PYTHON),
        ] {
            fs::set_permissions(path, fs::Permissions::from_mode(mode))
                .expect("remove read permission");
            let repaired = prepare_path(&bundle_path, &temporary.path).expect("repair permissions");
            assert!(!repaired.cache_hit);
            assert_eq!(fs::read(path).expect("read repaired payload"), original);
            assert_eq!(
                fs::metadata(path)
                    .expect("payload metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                restored_mode
            );
            assert!(
                prepare_path(&bundle_path, &temporary.path)
                    .expect("reuse repair")
                    .cache_hit
            );
        }
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
            inspect(
                &mut Cursor::new(&bytes),
                u64::try_from(bytes.len()).expect("fixture length fits u64"),
            )
            .expect("inspect")
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
            runtime_offset: u64::try_from(stub.len()).expect("fixture length fits u64"),
            runtime_size: u64::try_from(runtime.len()).expect("fixture length fits u64"),
            python_offset: u64::try_from(stub.len())
                .expect("stub length fits u64")
                .checked_add(u64::try_from(runtime.len()).expect("runtime length fits u64"))
                .expect("fixture offset fits u64"),
            python_size: u64::try_from(python.len()).expect("fixture length fits u64"),
        }
    }

    fn replace_trailer(bundle: &mut [u8], trailer: Trailer) {
        bundle
            .last_chunk_mut::<TRAILER_SIZE>()
            .expect("fixture contains a trailer")
            .copy_from_slice(&trailer.encode());
    }

    fn decode_hash(hex: &str) -> [u8; 8] {
        let hex = hex.trim().as_bytes();
        assert_eq!(hex.len(), 16);
        let mut hash = [0; 8];
        for (byte, &[high, low]) in hash.iter_mut().zip(hex.as_chunks::<2>().0) {
            *byte = hex_nibble(high) << 4 | hex_nibble(low);
        }
        hash
    }

    fn hex_nibble(byte: u8) -> u8 {
        char::from(byte)
            .to_digit(16)
            .and_then(|nibble| u8::try_from(nibble).ok())
            .expect("fixture contains a hexadecimal digit")
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
