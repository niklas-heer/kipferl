const STANDARD_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URLSAFE_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HexDecodeError {
    OddLength,
    InvalidDigit,
}

#[expect(
    clippy::indexing_slicing,
    reason = "Every alphabet index is formed from at most six bits, within the fixed 64-byte alphabet."
)]
pub(super) fn base64_encode(input: &[u8], urlsafe: bool) -> Vec<u8> {
    let alphabet = if urlsafe {
        URLSAFE_ALPHABET
    } else {
        STANDARD_ALPHABET
    };
    let mut output = Vec::with_capacity(input.len().div_ceil(3).saturating_mul(4));

    for chunk in input.chunks(3) {
        let Some((&first, _)) = chunk.split_first() else {
            continue;
        };
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        output.push(alphabet[usize::from(first >> 2)]);
        output.push(alphabet[usize::from(((first & 0x03) << 4) | (second >> 4))]);
        if chunk.len() > 1 {
            output.push(alphabet[usize::from(((second & 0x0f) << 2) | (third >> 6))]);
        } else {
            output.push(b'=');
        }
        if chunk.len() > 2 {
            output.push(alphabet[usize::from(third & 0x3f)]);
        } else {
            output.push(b'=');
        }
    }

    output
}

pub(super) fn base64_decode(input: &[u8], urlsafe: bool) -> Result<Vec<u8>, ()> {
    if !input.len().is_multiple_of(4) {
        return Err(());
    }
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let padding = if input.ends_with(b"==") {
        2
    } else {
        usize::from(input.ends_with(b"="))
    };
    if input
        .get(..input.len().checked_sub(padding).ok_or(())?)
        .ok_or(())?
        .contains(&b'=')
    {
        return Err(());
    }

    let output_length = (input.len() / 4)
        .checked_mul(3)
        .and_then(|length| length.checked_sub(padding))
        .ok_or(())?;
    let mut output = Vec::with_capacity(output_length);
    let chunk_count = input.len() / 4;

    for (index, chunk) in input.chunks_exact(4).enumerate() {
        let last = index == chunk_count.saturating_sub(1);
        let [first_byte, second_byte, third_byte, fourth_byte] = chunk else {
            return Err(());
        };
        let chunk_padding = if last { padding } else { 0 };
        let first = decode_digit(*first_byte, urlsafe).ok_or(())?;
        let second = decode_digit(*second_byte, urlsafe).ok_or(())?;
        let third = if chunk_padding < 2 {
            decode_digit(*third_byte, urlsafe).ok_or(())?
        } else {
            0
        };
        let fourth = if chunk_padding == 0 {
            decode_digit(*fourth_byte, urlsafe).ok_or(())?
        } else {
            0
        };

        if chunk_padding == 2 && second & 0x0f != 0 {
            return Err(());
        }
        if chunk_padding == 1 && third & 0x03 != 0 {
            return Err(());
        }

        output.push((first << 2) | (second >> 4));
        if chunk_padding < 2 {
            output.push((second << 4) | (third >> 2));
        }
        if chunk_padding == 0 {
            output.push((third << 6) | fourth);
        }
    }

    debug_assert_eq!(output.len(), output_length);
    Ok(output)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "Match arms restrict ASCII inputs before subtracting their range start; resulting values are at most 61."
)]
const fn decode_digit(value: u8, urlsafe: bool) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'-' if urlsafe => Some(62),
        b'_' if urlsafe => Some(63),
        _ => None,
    }
}

#[expect(
    clippy::indexing_slicing,
    reason = "Both lookup indices are four-bit nibbles, within the fixed 16-byte hex alphabet."
)]
pub(super) fn hex_encode(input: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = Vec::with_capacity(input.len().saturating_mul(2));
    for byte in input {
        output.push(HEX[usize::from(byte >> 4)]);
        output.push(HEX[usize::from(byte & 0x0f)]);
    }
    output
}

pub(super) fn hex_decode(input: &[u8]) -> Result<Vec<u8>, HexDecodeError> {
    if !input.len().is_multiple_of(2) {
        return Err(HexDecodeError::OddLength);
    }
    let mut output = Vec::with_capacity(input.len() / 2);
    for pair in input.chunks_exact(2) {
        let [high_byte, low_byte] = pair else {
            return Err(HexDecodeError::OddLength);
        };
        let high = hex_digit(*high_byte).ok_or(HexDecodeError::InvalidDigit)?;
        let low = hex_digit(*low_byte).ok_or(HexDecodeError::InvalidDigit)?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn hex_digit(value: u8) -> Option<u8> {
    u8::try_from(char::from(value).to_digit(16)?).ok()
}

pub(super) fn crc32(input: &[u8]) -> u32 {
    let mut checksum = u32::MAX;
    for byte in input {
        checksum ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(checksum & 1);
            checksum = (checksum >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !checksum
}

#[cfg(test)]
mod tests {
    use super::{HexDecodeError, base64_decode, base64_encode, crc32, hex_decode, hex_encode};

    #[test]
    fn base64_vectors_match_zig() {
        for (input, encoded) in [
            (&b""[..], &b""[..]),
            (&b"f"[..], &b"Zg=="[..]),
            (&b"fo"[..], &b"Zm8="[..]),
            (&b"foo"[..], &b"Zm9v"[..]),
            (&b"foobar"[..], &b"Zm9vYmFy"[..]),
            (&b"\xfb\xfc"[..], &b"+/w="[..]),
        ] {
            assert_eq!(base64_encode(input, false), encoded);
            assert_eq!(base64_decode(encoded, false), Ok(input.to_vec()));
        }
        assert_eq!(base64_encode(b"\xfb\xfc", true), b"-_w=");
        assert_eq!(base64_decode(b"-_w=", true), Ok(vec![0xfb, 0xfc]));
        assert_eq!(base64_decode(b"+/w=", true), Ok(vec![0xfb, 0xfc]));
    }

    #[test]
    fn base64_decoder_rejects_noncanonical_or_malformed_input() {
        for value in [
            &b"abc"[..],
            &b"YR=="[..],
            &b"YQ="[..],
            &b"YQ==="[..],
            &b"YWJj="[..],
            &b"AA=A"[..],
            &b"===="[..],
            &b" YQ=="[..],
            &b"YQ==\n"[..],
        ] {
            assert_eq!(base64_decode(value, false), Err(()), "{value:?}");
        }
    }

    #[test]
    fn all_bytes_roundtrip_both_alphabets() {
        let input: Vec<u8> = (0..=255).collect();
        for urlsafe in [false, true] {
            let encoded = base64_encode(&input, urlsafe);
            assert_eq!(base64_decode(&encoded, urlsafe), Ok(input.clone()));
        }
    }

    #[test]
    fn hex_and_crc_vectors_match_zig() {
        assert_eq!(hex_encode(b"\x00\xde\xad\xbe\xef\xff"), b"00deadbeefff");
        assert_eq!(hex_decode(b"DeAdBeEf"), Ok(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(hex_decode(b"a"), Err(HexDecodeError::OddLength));
        assert_eq!(hex_decode(b"gg"), Err(HexDecodeError::InvalidDigit));
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"123456789"), 3_421_780_262);
        assert_eq!(crc32(&(0..=255).collect::<Vec<_>>()), 688_229_491);
    }
}
