use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use kipferl_format::Trailer;
use kipferl_loader::inspect;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = env::args_os().collect();
    let [_, source_bundle, rust_loader, output] = arguments.as_slice() else {
        return Err("usage: repack_for_test SOURCE_BUNDLE RUST_LOADER OUTPUT".into());
    };

    let mut source = File::open(source_bundle)?;
    let source_size = source.metadata()?.len();
    let metadata = inspect(&mut source, source_size)?;
    let runtime = read_payload(
        &mut source,
        metadata.trailer.runtime_offset,
        metadata.trailer.runtime_size,
    )?;
    let python = read_payload(
        &mut source,
        metadata.trailer.python_offset,
        metadata.trailer.python_size,
    )?;
    let loader = fs::read(rust_loader)?;

    let runtime_offset = u64::try_from(loader.len())?;
    let runtime_size = u64::try_from(runtime.len())?;
    let trailer = Trailer {
        runtime_offset,
        runtime_size,
        python_offset: runtime_offset
            .checked_add(runtime_size)
            .ok_or("bundle offset exceeds u64")?,
        python_size: u64::try_from(python.len())?,
    };
    let output = PathBuf::from(output);
    let mut file = File::create(&output)?;
    file.write_all(&loader)?;
    file.write_all(&runtime)?;
    file.write_all(&python)?;
    file.write_all(&trailer.encode())?;
    file.set_permissions(fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn read_payload(file: &mut File, offset: u64, size: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let size: usize = size.try_into()?;
    let mut payload = vec![0; size];
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut payload)?;
    Ok(payload)
}
