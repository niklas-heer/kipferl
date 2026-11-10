//! The single decision whether Kipferl's own CLI output may contain styling.
//!
//! Commands write their styled text unconditionally. `main` wraps stdout and
//! stderr in an [`Output`] that, per stream, either passes bytes through or
//! removes every ANSI SGR sequence (colour, bold, dim, reset). Other control
//! sequences, such as `dev --clear`'s screen reset, are left intact.
//!
//! The rule, checked in this order:
//!
//! 1. `NO_COLOR` set to a non-empty value disables styling (<https://no-color.org>).
//! 2. `CLICOLOR_FORCE` set to a non-empty value other than `0`, or
//!    `FORCE_COLOR` set to a non-empty value other than `0` or `false`,
//!    enables styling even when the stream is not a terminal.
//! 3. Otherwise the stream is styled only when it is a terminal and `TERM` is
//!    not `dumb`.
//!
//! Output of user scripts started by `run` and `dev` does not pass through
//! this filter; the runtime applies its own documented rule.

use std::ffi::OsString;
use std::io::{self, Write};

const ESCAPE: u8 = 0x1b;
/// Longest escape sequence buffered before it is passed through unchanged.
const MAX_SEQUENCE: usize = 32;

/// Decide from the process environment whether a stream may be styled.
#[must_use]
pub fn enabled(is_terminal: bool) -> bool {
    decide(is_terminal, |name| std::env::var_os(name))
}

fn decide(is_terminal: bool, variable: impl Fn(&str) -> Option<OsString>) -> bool {
    let set = |name: &str| variable(name).filter(|value| !value.is_empty());
    if set("NO_COLOR").is_some() {
        return false;
    }
    let forced = set("CLICOLOR_FORCE").is_some_and(|value| value != "0")
        || set("FORCE_COLOR").is_some_and(|value| value != "0" && value != "false");
    forced || (is_terminal && variable("TERM").is_none_or(|term| term != "dumb"))
}

/// A writer that removes ANSI SGR sequences unless styling is enabled.
///
/// Sequences split across writes are recognized; an incomplete sequence left
/// at the end is written unchanged when the writer is dropped.
pub struct Output<W: Write> {
    inner: W,
    styled: bool,
    pending: Vec<u8>,
}

impl<W: Write> Output<W> {
    pub const fn new(inner: W, styled: bool) -> Self {
        Self {
            inner,
            styled,
            pending: Vec::new(),
        }
    }

    fn filter(&mut self, byte: u8, plain: &mut Vec<u8>) {
        match self.pending.as_slice() {
            [] if byte == ESCAPE => self.pending.push(byte),
            [] => plain.push(byte),
            [_] if byte == b'[' => self.pending.push(byte),
            [_, b'[', ..] if (0x20..=0x3f).contains(&byte) && self.pending.len() < MAX_SEQUENCE => {
                self.pending.push(byte);
            }
            [_, b'[', ..] if byte == b'm' => self.pending.clear(),
            [_, b'[', ..] if (0x40..=0x7e).contains(&byte) => {
                plain.append(&mut self.pending);
                plain.push(byte);
            }
            _ => {
                plain.append(&mut self.pending);
                self.filter(byte, plain);
            }
        }
    }
}

impl<W: Write> Write for Output<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.styled {
            return self.inner.write(buffer);
        }
        let mut plain = Vec::with_capacity(buffer.len());
        for &byte in buffer {
            self.filter(byte, &mut plain);
        }
        self.inner.write_all(&plain)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> Drop for Output<W> {
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            let _ = self.inner.write_all(&self.pending);
            let _ = self.inner.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::io::Write;

    use super::{Output, decide};

    fn with(variables: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let variables: Vec<(String, String)> = variables
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        move |name| {
            variables
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    #[test]
    fn follows_no_color_force_and_terminal_rules_in_order() {
        for (terminal, variables, expected) in [
            (true, &[][..], true),
            (false, &[][..], false),
            (true, &[("NO_COLOR", "1")][..], false),
            (true, &[("NO_COLOR", "")][..], true),
            (true, &[("TERM", "dumb")][..], false),
            (true, &[("TERM", "xterm-256color")][..], true),
            (false, &[("CLICOLOR_FORCE", "1")][..], true),
            (false, &[("CLICOLOR_FORCE", "0")][..], false),
            (false, &[("CLICOLOR_FORCE", "")][..], false),
            (false, &[("FORCE_COLOR", "1")][..], true),
            (false, &[("FORCE_COLOR", "3")][..], true),
            (false, &[("FORCE_COLOR", "0")][..], false),
            (false, &[("FORCE_COLOR", "false")][..], false),
            (true, &[("TERM", "dumb"), ("FORCE_COLOR", "1")][..], true),
            (
                false,
                &[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")][..],
                false,
            ),
            (true, &[("NO_COLOR", "x"), ("FORCE_COLOR", "1")][..], false),
        ] {
            assert_eq!(
                decide(terminal, with(variables)),
                expected,
                "terminal={terminal} {variables:?}"
            );
        }
    }

    fn plain(chunks: &[&[u8]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut output = Output::new(&mut bytes, false);
            for chunk in chunks {
                output.write_all(chunk).expect("write to memory");
            }
        }
        bytes
    }

    #[test]
    fn removes_styles_even_when_split_and_keeps_other_sequences() {
        assert_eq!(
            plain(&[b"\x1b[36m\x1b[1mKipferl\x1b[0m \x1b[2mv1\x1b[0m\n"]),
            b"Kipferl v1\n"
        );
        assert_eq!(plain(&[b"a\x1b", b"[1;3", b"1mb\x1b[", b"0m"]), b"ab");
        assert_eq!(
            plain(&[b"\x1b[2J\x1b[H\x1b[?25hx"]),
            b"\x1b[2J\x1b[H\x1b[?25hx"
        );
        assert_eq!(plain(&[b"\x1bx \x1b\x1b[1my"]), b"\x1bx \x1by");
        assert_eq!(plain(&[b"tail\x1b["]), b"tail\x1b[");
        assert_eq!(plain(&["✓ ü".as_bytes()]), "✓ ü".as_bytes());
    }

    #[test]
    fn passes_bytes_through_when_styled() {
        let mut bytes = Vec::new();
        Output::new(&mut bytes, true)
            .write_all(b"\x1b[1mbold\x1b[0m")
            .expect("write to memory");
        assert_eq!(bytes, b"\x1b[1mbold\x1b[0m");
    }
}
