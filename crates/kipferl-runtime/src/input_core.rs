//! Allocation-free primitives shared by Kipferl's interactive input module.

#[must_use]
pub fn string_length(value: &str) -> usize {
    value.len()
}

#[must_use]
pub fn strings_equal(left: &str, right: &str) -> bool {
    left == right
}

#[must_use]
pub fn starts_with(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
}

/// Clamps without panicking when the legacy caller supplies an inverted range.
#[must_use]
pub fn clamp(value: i32, minimum: i32, maximum: i32) -> i32 {
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}

#[must_use]
pub fn wrap_index(value: i32, count: i32) -> i32 {
    if count <= 0 {
        0
    } else {
        value.rem_euclid(count)
    }
}

#[cfg(test)]
mod tests {
    use super::{clamp, starts_with, string_length, strings_equal, wrap_index};

    #[test]
    fn string_vectors_match_zig() {
        assert_eq!(string_length("hello"), 5);
        assert_eq!(string_length(""), 0);
        assert!(strings_equal("hello", "hello"));
        assert!(!strings_equal("hello", "world"));
        assert!(!strings_equal("hello", "hell"));
        assert!(starts_with("hello world", "hello"));
        assert!(!starts_with("hello", "world"));
        assert!(starts_with("hello", ""));
    }

    #[test]
    fn navigation_vectors_match_and_extend_zig() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
        assert_eq!(clamp(5, 10, 0), 10);

        assert_eq!(wrap_index(0, 5), 0);
        assert_eq!(wrap_index(1, 5), 1);
        assert_eq!(wrap_index(5, 5), 0);
        assert_eq!(wrap_index(-1, 5), 4);
        assert_eq!(wrap_index(1, 0), 0);
        assert_eq!(wrap_index(1, -1), 0);
    }
}
