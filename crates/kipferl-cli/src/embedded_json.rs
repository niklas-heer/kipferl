//! Bounded decoding for compressed, immutable compatibility evidence.
use std::io::{self, Read};

use flate2::read::GzDecoder;

pub fn decode(bytes: &[u8]) -> io::Result<String> {
    decode_with_limit(bytes, 8 * 1024 * 1024)
}

fn decode_with_limit(bytes: &[u8], limit: u64) -> io::Result<String> {
    let mut content = Vec::new();
    GzDecoder::new(bytes)
        .take(limit.saturating_add(1))
        .read_to_end(&mut content)?;
    if u64::try_from(content.len()).map_err(io::Error::other)? > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "embedded compatibility evidence exceeds its decompression limit",
        ));
    }
    String::from_utf8(content).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use flate2::{Compression, GzBuilder};

    use super::decode_with_limit;

    fn compressed(content: &[u8]) -> io::Result<Vec<u8>> {
        let mut encoder = GzBuilder::new()
            .mtime(0)
            .write(Vec::new(), Compression::best());
        encoder.write_all(content)?;
        encoder.finish()
    }

    #[test]
    fn exact_limit_and_invalid_content_fail_closed() -> io::Result<()> {
        let bytes = compressed(b"valid")?;
        if decode_with_limit(&bytes, 5)? != "valid"
            || decode_with_limit(&bytes, 4).is_ok()
            || decode_with_limit(b"invalid gzip", 100).is_ok()
            || decode_with_limit(&compressed(&[0xff])?, 100).is_ok()
            || decode_with_limit(
                bytes
                    .get(..bytes.len().saturating_sub(1))
                    .unwrap_or_default(),
                100,
            )
            .is_ok()
        {
            return Err(io::Error::other("compressed evidence validation failed"));
        }
        Ok(())
    }
}
