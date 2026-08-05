#[derive(Debug, Eq, PartialEq)]
pub(super) enum DecodedKey<'a> {
    None,
    Named(&'static str),
    Text(&'a [u8]),
}

pub(super) fn decode_key(bytes: &[u8]) -> DecodedKey<'_> {
    if bytes.is_empty() {
        return DecodedKey::None;
    }

    if bytes.len() >= 3 && bytes[0] == 0x1b && bytes[1] == b'[' {
        let named = match bytes[2] {
            b'A' => Some("up"),
            b'B' => Some("down"),
            b'C' => Some("right"),
            b'D' => Some("left"),
            b'H' => Some("home"),
            b'F' => Some("end"),
            _ => None,
        };
        if let Some(named) = named {
            return DecodedKey::Named(named);
        }

        if bytes.get(3) == Some(&b'~') {
            let named = match bytes[2] {
                b'3' => Some("delete"),
                b'5' => Some("pageup"),
                b'6' => Some("pagedown"),
                _ => None,
            };
            if let Some(named) = named {
                return DecodedKey::Named(named);
            }
        }
    }

    if bytes.len() == 1 {
        let named = match bytes[0] {
            b'\r' | b'\n' => Some("enter"),
            0x1b => Some("escape"),
            0x7f | 0x08 => Some("backspace"),
            b'\t' => Some("tab"),
            3 => Some("ctrl-c"),
            _ => None,
        };
        if let Some(named) = named {
            return DecodedKey::Named(named);
        }
    }

    DecodedKey::Text(bytes)
}

pub(super) fn cursor_position(x: i64, y: i64) -> Option<String> {
    bounded(
        format!("\x1b[{};{}H", y.wrapping_add(1), x.wrapping_add(1)),
        32,
    )
}

pub(super) fn cursor_move(count: i64, direction: char) -> Option<String> {
    debug_assert!(matches!(direction, 'A' | 'B' | 'C' | 'D'));
    bounded(format!("\x1b[{count}{direction}"), 16)
}

fn bounded(value: String, capacity: usize) -> Option<String> {
    (value.len() <= capacity).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{DecodedKey, cursor_move, cursor_position, decode_key};

    #[test]
    fn key_vectors_match_zig() {
        for (bytes, expected) in [
            (&b"\x1b[A"[..], "up"),
            (&b"\x1b[B"[..], "down"),
            (&b"\x1b[C"[..], "right"),
            (&b"\x1b[D"[..], "left"),
            (&b"\x1b[H"[..], "home"),
            (&b"\x1b[F"[..], "end"),
            (&b"\x1b[3~"[..], "delete"),
            (&b"\x1b[5~"[..], "pageup"),
            (&b"\x1b[6~"[..], "pagedown"),
            (&b"\r"[..], "enter"),
            (&b"\n"[..], "enter"),
            (&b"\x1b"[..], "escape"),
            (&b"\x7f"[..], "backspace"),
            (&b"\x08"[..], "backspace"),
            (&b"\t"[..], "tab"),
            (&b"\x03"[..], "ctrl-c"),
        ] {
            assert_eq!(decode_key(bytes), DecodedKey::Named(expected));
        }
        assert_eq!(decode_key(b""), DecodedKey::None);
        assert_eq!(decode_key(b"x"), DecodedKey::Text(b"x"));
        assert_eq!(decode_key("é".as_bytes()), DecodedKey::Text("é".as_bytes()));
        assert_eq!(decode_key(b"\x1b[Z"), DecodedKey::Text(b"\x1b[Z"));
    }

    #[test]
    fn terminal_sequences_match_zig() {
        assert_eq!(cursor_position(2, 3).as_deref(), Some("\x1b[4;3H"));
        assert_eq!(cursor_position(-1, -1).as_deref(), Some("\x1b[0;0H"));
        assert_eq!(cursor_move(1, 'A').as_deref(), Some("\x1b[1A"));
        assert_eq!(cursor_move(-2, 'D').as_deref(), Some("\x1b[-2D"));
        assert!(cursor_position(i64::MAX, i64::MAX).is_none());
        assert!(cursor_move(i64::MIN, 'A').is_none());
    }
}
