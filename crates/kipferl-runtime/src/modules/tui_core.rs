use std::fmt::Write as _;

pub(super) const BORDER_ROUNDED: i64 = 0;
pub(super) const BORDER_SQUARE: i64 = 1;
pub(super) const BORDER_DOUBLE: i64 = 2;
pub(super) const BORDER_HEAVY: i64 = 3;
pub(super) const BORDER_NONE: i64 = 4;

pub(super) const SYMBOL_SUCCESS: &str = "✓";
pub(super) const SYMBOL_ERROR: &str = "✗";
pub(super) const SYMBOL_WARNING: &str = "⚠";
pub(super) const SYMBOL_INFO: &str = "ℹ";

const PROGRESS_FILL: &str = "█";
const PROGRESS_EMPTY: &str = "░";
const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone, Copy)]
pub(super) enum BorderStyle {
    Rounded,
    Square,
    Double,
    Heavy,
    None,
}

impl BorderStyle {
    pub fn for_box(value: &str) -> Self {
        match value {
            "square" => Self::Square,
            "double" => Self::Double,
            "heavy" => Self::Heavy,
            "none" => Self::None,
            _ => Self::Rounded,
        }
    }

    pub fn for_table(value: &str) -> Self {
        match value {
            "rounded" => Self::Rounded,
            "double" => Self::Double,
            "heavy" => Self::Heavy,
            "none" => Self::None,
            _ => Self::Square,
        }
    }
}

pub(super) struct BoxChars {
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub h: &'static str,
    pub v: &'static str,
}

pub(super) const fn box_chars(style: BorderStyle) -> BoxChars {
    match style {
        BorderStyle::Rounded => BoxChars {
            tl: "╭",
            tr: "╮",
            bl: "╰",
            br: "╯",
            h: "─",
            v: "│",
        },
        BorderStyle::Square => BoxChars {
            tl: "┌",
            tr: "┐",
            bl: "└",
            br: "┘",
            h: "─",
            v: "│",
        },
        BorderStyle::Double => BoxChars {
            tl: "╔",
            tr: "╗",
            bl: "╚",
            br: "╝",
            h: "═",
            v: "║",
        },
        BorderStyle::Heavy => BoxChars {
            tl: "┏",
            tr: "┓",
            bl: "┗",
            br: "┛",
            h: "━",
            v: "┃",
        },
        BorderStyle::None => BoxChars {
            tl: " ",
            tr: " ",
            bl: " ",
            br: " ",
            h: " ",
            v: " ",
        },
    }
}

pub(super) struct TableChars {
    pub h: &'static str,
    pub v: &'static str,
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub th: &'static str,
    pub bh: &'static str,
    pub lv: &'static str,
    pub rv: &'static str,
    pub cross: &'static str,
}

pub(super) const fn table_chars(style: BorderStyle) -> TableChars {
    let (h, v, th, bh, lv, rv, cross) = match style {
        BorderStyle::Double => ("═", "║", "╦", "╩", "╠", "╣", "╬"),
        BorderStyle::Heavy => ("━", "┃", "┳", "┻", "┣", "┫", "╋"),
        BorderStyle::None => (" ", " ", " ", " ", " ", " ", " "),
        BorderStyle::Rounded | BorderStyle::Square => ("─", "│", "┬", "┴", "├", "┤", "┼"),
    };
    let (tl, tr, bl, br) = match style {
        BorderStyle::Rounded => ("╭", "╮", "╰", "╯"),
        BorderStyle::Square => ("┌", "┐", "└", "┘"),
        BorderStyle::Double => ("╔", "╗", "╚", "╝"),
        BorderStyle::Heavy => ("┏", "┓", "┗", "┛"),
        BorderStyle::None => (" ", " ", " ", " "),
    };
    TableChars {
        h,
        v,
        tl,
        tr,
        bl,
        br,
        th,
        bh,
        lv,
        rv,
        cross,
    }
}

/// Mirrors the legacy byte-oriented width algorithm, including its treatment
/// of every three- and four-byte UTF-8 scalar as double-width.
pub(super) fn visible_len(value: &str) -> usize {
    let mut characters = value
        .split('\0')
        .next()
        .unwrap_or_default()
        .chars()
        .peekable();
    let mut length = 0_usize;
    while let Some(character) = characters.next() {
        if character == '\x1b' && characters.peek() == Some(&'[') {
            characters.next();
            for escaped in characters.by_ref() {
                if matches!(escaped, 'm' | 'H' | 'J' | 'K') {
                    break;
                }
            }
        } else {
            length = length.saturating_add(if character.len_utf8() >= 3 { 2 } else { 1 });
        }
    }
    length
}

pub(super) fn color_code(name: &str) -> Option<i32> {
    match name {
        "black" => Some(30),
        "red" => Some(31),
        "green" => Some(32),
        "yellow" => Some(33),
        "blue" => Some(34),
        "magenta" => Some(35),
        "cyan" => Some(36),
        "white" => Some(37),
        "gray" | "grey" => Some(90),
        _ => None,
    }
}

pub(super) fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    super::ansi_core::parse_hex(value)
}

#[expect(
    clippy::fn_params_excessive_bools,
    reason = "The five independent decorations correspond directly to the existing Python style API flags."
)]
#[expect(
    clippy::expect_used,
    reason = "fmt::Write for String is infallible; every write here formats only integer channels into a String."
)]
pub(super) fn style_code(
    foreground: Option<&str>,
    background: Option<&str>,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
) -> String {
    let mut output = String::new();
    if bold {
        push_style_separator(&mut output);
        output.push('1');
    }
    if dim {
        push_style_separator(&mut output);
        output.push('2');
    }
    if italic {
        push_style_separator(&mut output);
        output.push('3');
    }
    if underline {
        push_style_separator(&mut output);
        output.push('4');
    }
    if strikethrough {
        push_style_separator(&mut output);
        output.push('9');
    }
    if let Some(foreground) = foreground {
        if let Some(code) = color_code(foreground) {
            push_style_separator(&mut output);
            write!(&mut output, "{code}").expect("writing to a String cannot fail");
        } else if let Some((r, g, b)) = parse_hex(foreground) {
            push_style_separator(&mut output);
            write!(&mut output, "38;2;{r};{g};{b}").expect("writing to a String cannot fail");
        }
    }
    if let Some(background) = background {
        if let Some(code) = color_code(background) {
            push_style_separator(&mut output);
            write!(&mut output, "{}", code.saturating_add(10))
                .expect("writing to a String cannot fail");
        } else if let Some((r, g, b)) = parse_hex(background) {
            push_style_separator(&mut output);
            write!(&mut output, "48;2;{r};{g};{b}").expect("writing to a String cannot fail");
        }
    }

    if !output.is_empty() {
        output.push('m');
    }
    output
}

fn push_style_separator(output: &mut String) {
    if output.is_empty() {
        output.push_str("\x1b[");
    } else {
        output.push(';');
    }
}

pub(super) fn repeat(pattern: &str, count: usize) -> String {
    pattern.repeat(count)
}

pub(super) fn pad_left(value: &str, width: usize) -> String {
    let visible = visible_len(value);
    if visible >= width {
        value.to_owned()
    } else {
        format!("{value}{}", " ".repeat(width.saturating_sub(visible)))
    }
}

pub(super) fn progress_bar(current: u32, total: u32, width: u32) -> String {
    if total == 0 || width == 0 {
        return String::new();
    }
    let filled = u64::from(width).min(
        u64::from(width)
            .saturating_mul(u64::from(current))
            .checked_div(u64::from(total))
            .unwrap_or(0),
    );
    let filled = usize::try_from(filled).unwrap_or(usize::MAX);
    let width = usize::try_from(width).unwrap_or(usize::MAX);
    format!(
        "{}{}",
        PROGRESS_FILL.repeat(filled),
        PROGRESS_EMPTY.repeat(width.saturating_sub(filled))
    )
}

pub(super) fn percent_string(current: u32, total: u32) -> String {
    if total == 0 {
        return "0%".to_owned();
    }
    let value = 100_u32
        .wrapping_mul(current)
        .checked_div(total)
        .unwrap_or(0);
    let digits = if value >= 100 {
        3
    } else if value >= 10 {
        2
    } else {
        1
    };
    format!("{:0digits$}%", value % 1000)
}

pub(super) fn spinner_frame(index: u32) -> &'static str {
    usize::try_from(index % 10)
        .ok()
        .and_then(|index| SPINNER_FRAMES.get(index))
        .copied()
        .unwrap_or("⠋")
}

pub(super) fn elapsed_string(value: f64) -> String {
    // Zig's decimal formatter rounds midpoint values away from zero, while
    // Rust's formatting uses round-to-even. Round explicitly for parity.
    format!("{:.1}", (value * 10.0).round() / 10.0)
}

#[cfg(test)]
mod tests {
    use super::{
        BorderStyle, box_chars, color_code, elapsed_string, pad_left, parse_hex, percent_string,
        progress_bar, spinner_frame, style_code, table_chars, visible_len,
    };

    #[test]
    fn visible_width_vectors_match_zig() {
        assert_eq!(visible_len("hello"), 5);
        assert_eq!(visible_len(""), 0);
        assert_eq!(visible_len("\x1b[31mhello\x1b[0m"), 5);
        assert_eq!(visible_len("é"), 1);
        assert_eq!(visible_len("界"), 2);
        assert_eq!(visible_len("🙂"), 2);
        assert_eq!(visible_len("a\0ignored"), 1);
    }

    #[test]
    fn colors_and_hex_vectors_match_zig() {
        assert_eq!(color_code("red"), Some(31));
        assert_eq!(color_code("green"), Some(32));
        assert_eq!(color_code("purple"), None);
        assert_eq!(parse_hex("#ff5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex("#abc"), Some((170, 187, 204)));
        assert_eq!(parse_hex("#ggg"), None);
        assert_eq!(
            style_code(Some("red"), Some("#abc"), true, false, false, true, false),
            "\x1b[1;4;31;48;2;170;187;204m"
        );
    }

    #[test]
    fn progress_and_spinner_vectors_match_zig() {
        assert_eq!(progress_bar(5, 10, 10), "█████░░░░░");
        assert_eq!(progress_bar(1, 0, 10), "");
        assert_eq!(percent_string(5, 10), "50%");
        assert_eq!(percent_string(1, 0), "0%");
        assert_eq!(percent_string(150, 10), "500%");
        assert_eq!(spinner_frame(0), "⠋");
        assert_eq!(spinner_frame(10), "⠋");
        assert_eq!(elapsed_string(1.25), "1.3");
        assert_eq!(elapsed_string(-1.25), "-1.3");
    }

    #[test]
    fn progress_uses_the_full_counter_range_without_wrapping() {
        assert_eq!(progress_bar(u32::MAX, u32::MAX, 10), "██████████");
        assert_eq!(progress_bar(u32::MAX / 2, u32::MAX, 10), "████░░░░░░");
        assert_eq!(progress_bar(u32::MAX, 1, 10), "██████████");
        assert_eq!(visible_len("é界🙂\x1b[xyz"), 5);
    }

    #[test]
    fn layout_vectors_match_zig() {
        assert_eq!(pad_left("x", 3), "x  ");
        assert_eq!(pad_left("界", 3), "界 ");
        assert_eq!(box_chars(BorderStyle::Rounded).tl, "╭");
        assert_eq!(table_chars(BorderStyle::Double).cross, "╬");
    }
}
