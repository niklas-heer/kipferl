use std::io::{self, Read};
use std::sync::OnceLock;

use flate2::read::GzDecoder;

const FULL_RUNTIME_GZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/full-runtime.gz"));
const CORE_RUNTIME_GZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/core-runtime.gz"));

include!(concat!(env!("OUT_DIR"), "/embedded_runtime_keys.rs"));

static FULL_RUNTIME: OnceLock<io::Result<Vec<u8>>> = OnceLock::new();
static CORE_RUNTIME: OnceLock<io::Result<Vec<u8>>> = OnceLock::new();

pub fn full() -> io::Result<&'static [u8]> {
    cached_runtime(&FULL_RUNTIME, FULL_RUNTIME_GZIP, "full")
}

pub fn core() -> io::Result<&'static [u8]> {
    cached_runtime(&CORE_RUNTIME, CORE_RUNTIME_GZIP, "core")
}

fn cached_runtime(
    cache: &'static OnceLock<io::Result<Vec<u8>>>,
    compressed: &[u8],
    profile: &str,
) -> io::Result<&'static [u8]> {
    cache
        .get_or_init(|| decompress(compressed, profile))
        .as_deref()
        .map_err(|error| io::Error::new(error.kind(), error.to_string()))
}

pub const fn full_key() -> u64 {
    EMBEDDED_FULL_RUNTIME_KEY
}

#[cfg(test)]
pub const fn core_key() -> u64 {
    EMBEDDED_CORE_RUNTIME_KEY
}

fn decompress(compressed: &[u8], profile: &str) -> io::Result<Vec<u8>> {
    let mut runtime = Vec::new();
    GzDecoder::new(compressed)
        .read_to_end(&mut runtime)
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("embedded {profile} runtime is corrupt: {error}"),
            )
        })?;
    Ok(runtime)
}

#[cfg(test)]
mod tests {
    use super::{core, core_key, decompress, full, full_key};
    use crate::run_command::stable_hash;

    #[test]
    fn corrupt_runtime_has_actionable_error() {
        let error = decompress(b"not gzip", "core").expect_err("corrupt asset");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            error
                .to_string()
                .contains("embedded core runtime is corrupt")
        );
    }

    #[test]
    fn embedded_runtime_keys_match_decompressed_content() {
        assert_eq!(stable_hash(full().expect("full runtime")), full_key());
        assert_eq!(stable_hash(core().expect("core runtime")), core_key());
        assert!(core().expect("core runtime").len() < full().expect("full runtime").len());
    }
}
