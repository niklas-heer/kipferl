//! Safe, allocation-free argument parsing primitives shared by the native
//! `args` module.

/// Returns whether `value` is the integer syntax accepted by the Zig runtime.
#[must_use]
pub fn is_valid_integer(value: &str) -> bool {
    let digits = value.strip_prefix('-').unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

/// Returns whether `value` is the float syntax accepted by the Zig runtime.
///
/// This intentionally accepts integers and forms such as `.5` or `1.`, while
/// rejecting exponents and a leading plus sign, matching the existing API.
#[must_use]
pub fn is_valid_float(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    let mut has_dot = false;
    let mut has_digit = false;

    for byte in value.bytes() {
        if byte.is_ascii_digit() {
            has_digit = true;
        } else if byte == b'.' && !has_dot {
            has_dot = true;
        } else {
            return false;
        }
    }
    has_digit
}

/// Parses an integer using the legacy core's digit accumulation semantics.
///
/// Callers normally validate with [`is_valid_integer`] first. Non-digits are
/// ignored for exact compatibility with the original low-level helper.
#[must_use]
pub fn parse_integer(value: &str) -> i64 {
    let negative = value.starts_with('-');
    let digits = value.strip_prefix('-').unwrap_or(value);
    let parsed = digits.bytes().fold(0_i64, |result, byte| {
        if byte.is_ascii_digit() {
            result
                .wrapping_mul(10)
                .wrapping_add(i64::from(byte.wrapping_sub(b'0')))
        } else {
            result
        }
    });
    if negative {
        parsed.wrapping_neg()
    } else {
        parsed
    }
}

#[must_use]
pub fn is_long_flag(value: &str) -> bool {
    value.starts_with("--") && value.len() > 2
}

#[must_use]
pub fn is_short_flag(value: &str) -> bool {
    value.starts_with('-') && !value.starts_with("--") && value.len() > 1
}

#[must_use]
pub fn is_double_dash(value: &str) -> bool {
    value == "--"
}

#[must_use]
pub fn flag_name(value: &str) -> &str {
    value
        .strip_prefix("--")
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value)
}

#[must_use]
pub fn is_negative_number(value: &str) -> bool {
    value
        .strip_prefix('-')
        .and_then(|rest| rest.as_bytes().first())
        .is_some_and(u8::is_ascii_digit)
}

#[must_use]
pub fn is_truthy(value: &str) -> bool {
    matches!(
        value,
        "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "1" | "on" | "On" | "ON"
    )
}

#[must_use]
pub fn is_falsy(value: &str) -> bool {
    matches!(
        value,
        "false" | "False" | "FALSE" | "no" | "No" | "NO" | "0" | "off" | "Off" | "OFF"
    )
}

#[must_use]
pub fn is_negated_flag(value: &str) -> bool {
    value
        .strip_prefix("no-")
        .is_some_and(|remainder| !remainder.is_empty())
}

#[must_use]
pub fn negated_base(value: &str) -> &str {
    value
        .strip_prefix("no-")
        .filter(|rest| !rest.is_empty())
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::{
        flag_name, is_double_dash, is_falsy, is_long_flag, is_negated_flag, is_negative_number,
        is_short_flag, is_truthy, is_valid_float, is_valid_integer, negated_base, parse_integer,
    };

    #[test]
    fn integer_vectors_match_zig() {
        for valid in ["123", "-456", "0", "-0"] {
            assert!(is_valid_integer(valid), "{valid}");
        }
        for invalid in ["", "-", "+1", "abc", "12.34", " 1", "1 "] {
            assert!(!is_valid_integer(invalid), "{invalid}");
        }
        assert_eq!(parse_integer("123"), 123);
        assert_eq!(parse_integer("-456"), -456);
        assert_eq!(parse_integer("0"), 0);
        assert_eq!(parse_integer("12x3"), 123);
    }

    #[test]
    fn float_vectors_match_zig() {
        for valid in ["123", "12.34", "-12.34", ".5", "-.5", "1."] {
            assert!(is_valid_float(valid), "{valid}");
        }
        for invalid in ["", "-", ".", "abc", "1.2.3", "+1.0", "1e3"] {
            assert!(!is_valid_float(invalid), "{invalid}");
        }
    }

    #[test]
    fn flag_vectors_match_zig() {
        assert!(is_long_flag("--name"));
        assert!(!is_long_flag("--"));
        assert!(!is_long_flag("-n"));
        assert!(is_short_flag("-n"));
        assert!(is_short_flag("-1"));
        assert!(!is_short_flag("--name"));
        assert!(!is_short_flag("-"));
        assert!(is_double_dash("--"));
        assert!(!is_double_dash("---"));
        assert_eq!(flag_name("--name"), "name");
        assert_eq!(flag_name("-n"), "n");
        assert_eq!(flag_name("name"), "name");
        assert!(is_negative_number("-1"));
        assert!(is_negative_number("-9x"));
        assert!(!is_negative_number("-.5"));
        assert!(!is_negative_number("--name"));
    }

    #[test]
    fn boolean_and_negation_vectors_match_zig() {
        for value in [
            "true", "True", "TRUE", "yes", "Yes", "YES", "1", "on", "On", "ON",
        ] {
            assert!(is_truthy(value), "{value}");
        }
        for value in [
            "false", "False", "FALSE", "no", "No", "NO", "0", "off", "Off", "OFF",
        ] {
            assert!(is_falsy(value), "{value}");
        }
        for value in ["true ", "TRUEE", "y", "2", "enabled"] {
            assert!(!is_truthy(value), "{value}");
            assert!(!is_falsy(value), "{value}");
        }
        assert!(is_negated_flag("no-verbose"));
        assert!(!is_negated_flag("no-"));
        assert!(!is_negated_flag("verbose"));
        assert_eq!(negated_base("no-verbose"), "verbose");
        assert_eq!(negated_base("verbose"), "verbose");
    }
}
