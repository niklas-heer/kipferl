#![no_std]
#![forbid(unsafe_code)]
// Pilot stricter API/documentation checks in the small, stable wire-format crate.
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

use core::error::Error;
use core::fmt;

pub const TRAILER_MAGIC: [u8; 8] = *b"MCHARM01";
pub const TRAILER_SIZE: usize = 48;
const TRAILER_SIZE_U64: u64 = 48;

/// Version 1 trailer appended to every Kipferl universal executable.
///
/// The encoded field order remains compatible with the original Zig loader,
/// where the runtime fields were named `micropython_offset` and
/// `micropython_size`.
///
/// ```
/// use kipferl_format::Trailer;
///
/// let trailer = Trailer {
///     runtime_offset: 1,
///     runtime_size: 2,
///     python_offset: 3,
///     python_size: 4,
/// };
/// let decoded = Trailer::decode(&trailer.encode())?;
/// assert_eq!(decoded, trailer);
/// # Ok::<(), kipferl_format::DecodeError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trailer {
    pub runtime_offset: u64,
    pub runtime_size: u64,
    pub python_offset: u64,
    pub python_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    WrongSize { actual: usize },
    InvalidLeadingMagic,
    InvalidTrailingMagic,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongSize { actual } => write!(
                formatter,
                "universal trailer must be {TRAILER_SIZE} bytes, got {actual}"
            ),
            Self::InvalidLeadingMagic => formatter.write_str("invalid leading trailer magic"),
            Self::InvalidTrailingMagic => formatter.write_str("invalid trailing trailer magic"),
        }
    }
}

impl Error for DecodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    FileTooSmall,
    RuntimeStartsAtZero,
    EmptyRuntime,
    EmptyPython,
    PythonDoesNotFollowRuntime,
    IntegerOverflow,
    RuntimeOverlapsPython,
    PayloadOverlapsTrailer,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FileTooSmall => "file is smaller than the universal trailer",
            Self::RuntimeStartsAtZero => "runtime offset must follow the loader stub",
            Self::EmptyRuntime => "runtime payload is empty",
            Self::EmptyPython => "Python payload is empty",
            Self::PythonDoesNotFollowRuntime => "Python payload must follow the runtime payload",
            Self::IntegerOverflow => "payload range overflows u64",
            Self::RuntimeOverlapsPython => "runtime payload overlaps the Python payload",
            Self::PayloadOverlapsTrailer => "payload overlaps or extends beyond the trailer",
        })
    }
}

impl Error for LayoutError {}

impl Trailer {
    #[must_use]
    pub fn encode(self) -> [u8; TRAILER_SIZE] {
        let mut bytes = [0; TRAILER_SIZE];
        for (destination, field) in bytes.chunks_exact_mut(8).zip([
            TRAILER_MAGIC,
            self.runtime_offset.to_le_bytes(),
            self.runtime_size.to_le_bytes(),
            self.python_offset.to_le_bytes(),
            self.python_size.to_le_bytes(),
            TRAILER_MAGIC,
        ]) {
            destination.copy_from_slice(&field);
        }
        bytes
    }

    /// Decode exactly one trailer, checking its size and both magic markers.
    ///
    /// This does not validate the payload ranges; call [`Self::validate_layout`]
    /// with the complete executable's length before reading payloads.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] if the input is not exactly [`TRAILER_SIZE`]
    /// bytes or either magic marker is invalid.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let bytes: &[u8; TRAILER_SIZE] = bytes.try_into().map_err(|_| DecodeError::WrongSize {
            actual: bytes.len(),
        })?;

        let (fields, _) = bytes.as_chunks::<8>();
        let [
            leading,
            runtime_offset,
            runtime_size,
            python_offset,
            python_size,
            trailing,
        ] = fields
        else {
            return Err(DecodeError::WrongSize {
                actual: bytes.len(),
            });
        };
        if leading != &TRAILER_MAGIC {
            return Err(DecodeError::InvalidLeadingMagic);
        }
        if trailing != &TRAILER_MAGIC {
            return Err(DecodeError::InvalidTrailingMagic);
        }

        Ok(Self {
            runtime_offset: u64::from_le_bytes(*runtime_offset),
            runtime_size: u64::from_le_bytes(*runtime_size),
            python_offset: u64::from_le_bytes(*python_offset),
            python_size: u64::from_le_bytes(*python_size),
        })
    }

    /// Decode the trailer at the end of an in-memory universal executable.
    ///
    /// Payload ranges still require [`Self::validate_layout`].
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] if the file is shorter than [`TRAILER_SIZE`]
    /// or its final trailer has an invalid magic marker.
    pub fn decode_from_end(file: &[u8]) -> Result<Self, DecodeError> {
        let trailer = file
            .last_chunk::<TRAILER_SIZE>()
            .ok_or(DecodeError::WrongSize { actual: file.len() })?;
        Self::decode(trailer)
    }

    /// Match the original Zig loader's basic value checks.
    #[must_use]
    pub const fn is_sane(self) -> bool {
        self.runtime_offset != 0
            && self.runtime_size != 0
            && self.python_size != 0
            && self.python_offset > self.runtime_offset
    }

    /// Validate that both payload ranges fit before the trailer.
    ///
    /// Gaps are accepted for compatibility, but payload overlap, integer
    /// overflow, and truncated files are rejected.
    ///
    /// ```
    /// use kipferl_format::{Trailer, TRAILER_SIZE};
    ///
    /// let trailer = Trailer {
    ///     runtime_offset: 1,
    ///     runtime_size: 2,
    ///     python_offset: 3,
    ///     python_size: 4,
    /// };
    /// let file_size = 7 + TRAILER_SIZE as u64;
    /// assert!(trailer.validate_layout(file_size).is_ok());
    /// assert!(trailer.validate_layout(file_size - 1).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError`] for a file shorter than the trailer, zero runtime
    /// offset, empty payloads, reversed offsets, overflowing ranges, overlapping
    /// payloads, or payloads extending into the trailer or beyond the file.
    pub fn validate_layout(self, file_size: u64) -> Result<(), LayoutError> {
        let trailer_offset = file_size
            .checked_sub(TRAILER_SIZE_U64)
            .ok_or(LayoutError::FileTooSmall)?;
        if self.runtime_offset == 0 {
            return Err(LayoutError::RuntimeStartsAtZero);
        }
        if self.runtime_size == 0 {
            return Err(LayoutError::EmptyRuntime);
        }
        if self.python_size == 0 {
            return Err(LayoutError::EmptyPython);
        }
        if self.python_offset <= self.runtime_offset {
            return Err(LayoutError::PythonDoesNotFollowRuntime);
        }

        let runtime_end = self
            .runtime_offset
            .checked_add(self.runtime_size)
            .ok_or(LayoutError::IntegerOverflow)?;
        let python_end = self
            .python_offset
            .checked_add(self.python_size)
            .ok_or(LayoutError::IntegerOverflow)?;

        if runtime_end > self.python_offset {
            return Err(LayoutError::RuntimeOverlapsPython);
        }

        if python_end > trailer_offset {
            return Err(LayoutError::PayloadOverlapsTrailer);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, LayoutError, TRAILER_MAGIC, TRAILER_SIZE, Trailer};

    const STANDARD: Trailer = Trailer {
        runtime_offset: 50_000,
        runtime_size: 668_000,
        python_offset: 718_000,
        python_size: 35_000,
    };
    const SMALL: Trailer = Trailer {
        runtime_offset: 4_096,
        runtime_size: 8_192,
        python_offset: 12_288,
        python_size: 17,
    };

    #[test]
    fn matches_shared_golden_vectors() {
        for (trailer, fixture) in [
            (
                STANDARD,
                include_str!("../tests/fixtures/universal_trailer_v1_standard.hex"),
            ),
            (
                SMALL,
                include_str!("../tests/fixtures/universal_trailer_v1_small.hex"),
            ),
        ] {
            let golden = decode_hex(fixture);
            assert_eq!(trailer.encode(), golden);
            assert_eq!(Trailer::decode(&golden), Ok(trailer));
        }
    }

    #[test]
    fn decodes_from_the_end_of_a_file() {
        let mut file = [0_u8; 64];
        file[16..].copy_from_slice(&SMALL.encode());
        assert_eq!(Trailer::decode_from_end(&file), Ok(SMALL));
    }

    #[test]
    fn rejects_wrong_size_and_each_magic() {
        assert_eq!(
            Trailer::decode(&[0; 47]),
            Err(DecodeError::WrongSize { actual: 47 })
        );

        let mut bytes = STANDARD.encode();
        bytes[0] = 0;
        assert_eq!(
            Trailer::decode(&bytes),
            Err(DecodeError::InvalidLeadingMagic)
        );

        bytes = STANDARD.encode();
        bytes[TRAILER_SIZE - 1] = 0;
        assert_eq!(
            Trailer::decode(&bytes),
            Err(DecodeError::InvalidTrailingMagic)
        );
    }

    #[test]
    fn validates_payload_layout_without_requiring_contiguity() {
        assert!(STANDARD.is_sane());
        STANDARD
            .validate_layout(STANDARD.python_offset + STANDARD.python_size + 48)
            .expect("contiguous payloads are valid");

        let with_gaps = Trailer {
            runtime_offset: 4_096,
            runtime_size: 1_024,
            python_offset: 8_192,
            python_size: 512,
        };
        with_gaps
            .validate_layout(9_000)
            .expect("legacy-compatible gaps are valid");
    }

    #[test]
    fn preserves_the_original_zig_sanity_checks() {
        for invalid in [
            Trailer {
                runtime_offset: 0,
                ..STANDARD
            },
            Trailer {
                runtime_size: 0,
                ..STANDARD
            },
            Trailer {
                python_offset: STANDARD.runtime_offset,
                ..STANDARD
            },
            Trailer {
                python_size: 0,
                ..STANDARD
            },
        ] {
            assert!(!invalid.is_sane());
        }
    }

    #[test]
    fn rejects_invalid_and_truncated_layouts() {
        assert_eq!(STANDARD.validate_layout(47), Err(LayoutError::FileTooSmall));

        assert_eq!(
            Trailer {
                runtime_offset: 0,
                ..STANDARD
            }
            .validate_layout(STANDARD.python_offset + STANDARD.python_size + 48),
            Err(LayoutError::RuntimeStartsAtZero)
        );

        assert_eq!(
            Trailer {
                runtime_size: 0,
                ..STANDARD
            }
            .validate_layout(STANDARD.python_offset + STANDARD.python_size + 48),
            Err(LayoutError::EmptyRuntime)
        );

        assert_eq!(
            Trailer {
                python_size: 0,
                ..STANDARD
            }
            .validate_layout(STANDARD.python_offset + STANDARD.python_size + 48),
            Err(LayoutError::EmptyPython)
        );

        assert_eq!(
            Trailer {
                python_offset: STANDARD.runtime_offset,
                ..STANDARD
            }
            .validate_layout(STANDARD.python_offset + STANDARD.python_size + 48),
            Err(LayoutError::PythonDoesNotFollowRuntime)
        );

        let overlap = Trailer {
            runtime_size: STANDARD.runtime_size + 1,
            ..STANDARD
        };
        assert_eq!(
            overlap.validate_layout(STANDARD.python_offset + STANDARD.python_size + 48),
            Err(LayoutError::RuntimeOverlapsPython)
        );

        assert_eq!(
            STANDARD.validate_layout(STANDARD.python_offset + STANDARD.python_size + 47),
            Err(LayoutError::PayloadOverlapsTrailer)
        );

        let overflow = Trailer {
            python_offset: u64::MAX,
            python_size: 1,
            ..STANDARD
        };
        assert_eq!(
            overflow.validate_layout(u64::MAX),
            Err(LayoutError::IntegerOverflow)
        );
    }

    fn decode_hex(fixture: &str) -> [u8; TRAILER_SIZE] {
        let mut output = [0; TRAILER_SIZE];
        let mut digits = fixture.bytes().filter(|byte| !byte.is_ascii_whitespace());

        for byte in &mut output {
            let high = hex_nibble(digits.next().expect("fixture has enough hex digits"));
            let low = hex_nibble(digits.next().expect("fixture has enough hex digits"));
            *byte = high << 4 | low;
        }
        assert!(digits.next().is_none(), "fixture has too many hex digits");
        output
    }

    fn hex_nibble(byte: u8) -> u8 {
        char::from(byte)
            .to_digit(16)
            .and_then(|nibble| u8::try_from(nibble).ok())
            .expect("fixture contains a hexadecimal digit")
    }

    #[test]
    fn magic_is_stable() {
        assert_eq!(TRAILER_MAGIC, *b"MCHARM01");
    }
}
