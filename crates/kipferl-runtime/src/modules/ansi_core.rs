const STANDARD_COLORS: &[(&str, u8)] = &[
    ("black", 0),
    ("red", 1),
    ("green", 2),
    ("yellow", 3),
    ("blue", 4),
    ("magenta", 5),
    ("cyan", 6),
    ("white", 7),
    ("gray", 8),
    ("grey", 8),
];

const FOREGROUND_CODES: &[&str; 16] = &[
    "30", "31", "32", "33", "34", "35", "36", "37", "90", "91", "92", "93", "94", "95", "96", "97",
];

const BACKGROUND_CODES: &[&str; 16] = &[
    "40", "41", "42", "43", "44", "45", "46", "47", "100", "101", "102", "103", "104", "105",
    "106", "107",
];

pub(super) fn foreground(value: i64) -> String {
    match u8::try_from(value) {
        Ok(index @ 0..=15) => format!("\x1b[{}m", FOREGROUND_CODES[index as usize]),
        Ok(index) => format!("\x1b[38;5;{index}m"),
        Err(_) => String::new(),
    }
}

pub(super) fn background(value: i64) -> String {
    match u8::try_from(value) {
        Ok(index @ 0..=15) => format!("\x1b[{}m", BACKGROUND_CODES[index as usize]),
        Ok(index) => format!("\x1b[48;5;{index}m"),
        Err(_) => String::new(),
    }
}

pub(super) fn named_foreground(value: &str) -> String {
    match color_index(value) {
        Some(index @ 0..=15) => foreground(i64::from(index)),
        _ => String::new(),
    }
}

pub(super) fn named_background(value: &str) -> String {
    match color_index(value) {
        Some(index @ 0..=15) => background(i64::from(index)),
        _ => String::new(),
    }
}

pub(super) fn hex_foreground(value: &str) -> String {
    parse_hex(value).map_or_else(String::new, |(red, green, blue)| {
        rgb(red, green, blue, false)
    })
}

pub(super) fn hex_background(value: &str) -> String {
    parse_hex(value).map_or_else(String::new, |(red, green, blue)| {
        rgb(red, green, blue, true)
    })
}

pub(super) fn rgb(red: u8, green: u8, blue: u8, background: bool) -> String {
    let channel = if background { 48 } else { 38 };
    format!("\x1b[{channel};2;{red};{green};{blue}m")
}

fn color_index(value: &str) -> Option<u8> {
    if let Some(name) = value.strip_prefix("bright_") {
        return STANDARD_COLORS
            .iter()
            .find_map(|&(candidate, index)| (candidate == name).then_some(index + 8));
    }
    STANDARD_COLORS
        .iter()
        .find_map(|&(candidate, index)| (candidate == value).then_some(index))
}

fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.strip_prefix('#')?;
    match value.as_bytes() {
        [red, green, blue] => Some((
            hex_digit(*red)? * 17,
            hex_digit(*green)? * 17,
            hex_digit(*blue)? * 17,
        )),
        [
            red_high,
            red_low,
            green_high,
            green_low,
            blue_high,
            blue_low,
        ] => Some((
            hex_digit(*red_high)? * 16 + hex_digit(*red_low)?,
            hex_digit(*green_high)? * 16 + hex_digit(*green_low)?,
            hex_digit(*blue_high)? * 16 + hex_digit(*blue_low)?,
        )),
        _ => None,
    }
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{background, foreground, hex_background, hex_foreground, named_foreground, rgb};

    #[test]
    fn matches_zig_color_vectors() {
        assert_eq!(named_foreground("red"), "\x1b[31m");
        assert_eq!(named_foreground("bright_red"), "\x1b[91m");
        assert_eq!(named_foreground("bright_gray"), "");
        assert_eq!(named_foreground("purple"), "");
        assert_eq!(foreground(196), "\x1b[38;5;196m");
        assert_eq!(foreground(-1), "");
        assert_eq!(background(255), "\x1b[48;5;255m");
        assert_eq!(hex_foreground("#ff5500"), "\x1b[38;2;255;85;0m");
        assert_eq!(hex_background("#f50"), "\x1b[48;2;255;85;0m");
        assert_eq!(hex_foreground("ff5500"), "");
        assert_eq!(rgb(1, 2, 3, false), "\x1b[38;2;1;2;3m");
    }
}
