use std::io::Read;
use std::sync::OnceLock;

use flate2::read::GzDecoder;

const FULL_RUNTIME_GZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/full-runtime.gz"));
const CORE_RUNTIME_GZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/core-runtime.gz"));

include!(concat!(env!("OUT_DIR"), "/embedded_runtime_keys.rs"));

static FULL_RUNTIME: OnceLock<Vec<u8>> = OnceLock::new();
static CORE_RUNTIME: OnceLock<Vec<u8>> = OnceLock::new();

pub(crate) fn full() -> &'static [u8] {
    FULL_RUNTIME
        .get_or_init(|| decompress(FULL_RUNTIME_GZIP, "full"))
        .as_slice()
}

pub(crate) fn core() -> &'static [u8] {
    CORE_RUNTIME
        .get_or_init(|| decompress(CORE_RUNTIME_GZIP, "core"))
        .as_slice()
}

pub(crate) const fn full_key() -> u64 {
    EMBEDDED_FULL_RUNTIME_KEY
}

#[cfg(test)]
pub(crate) const fn core_key() -> u64 {
    EMBEDDED_CORE_RUNTIME_KEY
}

fn decompress(compressed: &[u8], profile: &str) -> Vec<u8> {
    let mut runtime = Vec::new();
    GzDecoder::new(compressed)
        .read_to_end(&mut runtime)
        .unwrap_or_else(|error| panic!("embedded {profile} runtime is corrupt: {error}"));
    runtime
}

#[cfg(test)]
mod tests {
    use super::{core, core_key, full, full_key};
    use crate::run_command::stable_hash;

    #[test]
    fn embedded_runtime_keys_match_decompressed_content() {
        assert_eq!(stable_hash(full()), full_key());
        assert_eq!(stable_hash(core()), core_key());
        assert!(core().len() < full().len());
    }
}
