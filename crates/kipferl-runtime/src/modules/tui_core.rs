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

pub(super) fn box_chars(style: BorderStyle) -> BoxChars {
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

pub(super) fn table_chars(style: BorderStyle) -> TableChars {
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
    let bytes = value.as_bytes();
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let mut index = 0;
    let mut length = 0;
    while index < end {
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
            index += 2;
            while index < end && !matches!(bytes[index], b'm' | b'H' | b'J' | b'K') {
                index += 1;
            }
            if index < end {
                index += 1;
            }
        } else {
            let byte = bytes[index];
            if byte < 0x80 {
                length += 1;
                index += 1;
            } else if byte < 0xe0 {
                length += 1;
                index += 2;
            } else if byte < 0xf0 {
                length += 2;
                index += 3;
            } else {
                length += 2;
                index += 4;
            }
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
    let digits = value.strip_prefix('#')?;
    let digits = digits.as_bytes();
    let nibble = |value: u8| match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    };
    match digits.len() {
        3 => {
            let r = nibble(digits[0])?;
            let g = nibble(digits[1])?;
            let b = nibble(digits[2])?;
            Some((r * 17, g * 17, b * 17))
        }
        6 => {
            let r = nibble(digits[0])? * 16 + nibble(digits[1])?;
            let g = nibble(digits[2])? * 16 + nibble(digits[3])?;
            let b = nibble(digits[4])? * 16 + nibble(digits[5])?;
            Some((r, g, b))
        }
        _ => None,
    }
}

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
            write!(&mut output, "{}", code + 10).expect("writing to a String cannot fail");
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
        format!("{value}{}", " ".repeat(width - visible))
    }
}

pub(super) fn progress_bar(current: u32, total: u32, width: u32) -> String {
    if total == 0 || width == 0 {
        return String::new();
    }
    let filled = width.min(width.wrapping_mul(current) / total);
    format!(
        "{}{}",
        PROGRESS_FILL.repeat(filled as usize),
        PROGRESS_EMPTY.repeat((width - filled) as usize)
    )
}

pub(super) fn percent_string(current: u32, total: u32) -> String {
    if total == 0 {
        return "0%".to_owned();
    }
    let value = 100_u32.wrapping_mul(current) / total;
    let mut output = String::new();
    if value >= 100 {
        output.push(char::from(b'0' + ((value / 100) % 10) as u8));
    }
    if value >= 10 {
        output.push(char::from(b'0' + ((value / 10) % 10) as u8));
    }
    output.push(char::from(b'0' + (value % 10) as u8));
    output.push('%');
    output
}

pub(super) fn spinner_frame(index: u32) -> &'static str {
    SPINNER_FRAMES[index as usize % SPINNER_FRAMES.len()]
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
    fn layout_vectors_match_zig() {
        assert_eq!(pad_left("x", 3), "x  ");
        assert_eq!(pad_left("界", 3), "界 ");
        assert_eq!(box_chars(BorderStyle::Rounded).tl, "╭");
        assert_eq!(table_chars(BorderStyle::Double).cross, "╬");
    }
}
