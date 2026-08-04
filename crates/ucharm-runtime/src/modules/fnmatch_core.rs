struct CharacterClass {
    matched: bool,
    consumed: usize,
}

fn match_character_class(pattern: &[u8], character: u8) -> CharacterClass {
    if pattern.len() < 2 || pattern[0] != b'[' {
        return CharacterClass {
            matched: false,
            consumed: 0,
        };
    }

    let mut index = 1;
    let mut negated = false;
    let mut matched = false;

    if index < pattern.len() && matches!(pattern[index], b'!' | b'^') {
        negated = true;
        index += 1;
    }

    let start = index;
    while index < pattern.len() {
        if pattern[index] == b']' && index > start {
            break;
        }

        if index + 2 < pattern.len() && pattern[index + 1] == b'-' && pattern[index + 2] != b']' {
            if (pattern[index]..=pattern[index + 2]).contains(&character) {
                matched = true;
            }
            index += 3;
        } else {
            if pattern[index] == character {
                matched = true;
            }
            index += 1;
        }
    }

    if index >= pattern.len() || pattern[index] != b']' {
        return CharacterClass {
            matched: false,
            consumed: 0,
        };
    }

    CharacterClass {
        matched: matched != negated,
        consumed: index + 1,
    }
}

pub(super) fn matches(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut pattern_index = 0;
    let mut text_index = 0;
    let mut star_index = None;
    let mut match_index = 0;

    while text_index < text.len() {
        if pattern_index < pattern.len() {
            if pattern[pattern_index] == b'[' {
                let class = match_character_class(&pattern[pattern_index..], text[text_index]);
                if class.consumed > 0 {
                    if class.matched {
                        pattern_index += class.consumed;
                        text_index += 1;
                        continue;
                    }
                    if let Some(star) = star_index {
                        pattern_index = star + 1;
                        match_index += 1;
                        text_index = match_index;
                        continue;
                    }
                    return false;
                }
            }

            if pattern[pattern_index] == text[text_index] || pattern[pattern_index] == b'?' {
                pattern_index += 1;
                text_index += 1;
                continue;
            }

            if pattern[pattern_index] == b'*' {
                star_index = Some(pattern_index);
                match_index = text_index;
                pattern_index += 1;
                continue;
            }
        }

        if let Some(star) = star_index {
            pattern_index = star + 1;
            match_index += 1;
            text_index = match_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

pub(super) fn translate(pattern: &str) -> Vec<u8> {
    const LIMIT: usize = 1024;

    let mut output = Vec::with_capacity(LIMIT);
    output.extend_from_slice(b"(?s)");

    for character in pattern.bytes() {
        if output.len() >= LIMIT - 4 {
            break;
        }
        match character {
            b'*' => output.extend_from_slice(b".*"),
            b'?' => output.push(b'.'),
            b'.' | b'^' | b'$' | b'+' | b'{' | b'}' | b'|' | b'(' | b')' | b'\\' => {
                output.push(b'\\');
                output.push(character);
            }
            _ => output.push(character),
        }
    }

    if output.len() < LIMIT {
        output.push(b'\\');
    }
    if output.len() < LIMIT {
        output.push(b'Z');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{matches, translate};

    #[test]
    fn matches_literals_wildcards_ranges_and_negated_classes() {
        for (pattern, text, expected) in [
            ("foo", "foo", true),
            ("f*o", "foo", true),
            ("foo*bar", "foobazbar", true),
            ("a?c", "abc", true),
            ("[a-z]", "m", true),
            ("[a-z]", "M", false),
            ("[!a-z]", "M", true),
            ("a[0-9]b", "a1b", true),
            ("a[0-9]b", "aXb", false),
            ("[", "[", true),
            ("", "", true),
            ("", "a", false),
        ] {
            assert_eq!(matches(pattern, text), expected, "{text:?} vs {pattern:?}");
        }
    }

    #[test]
    fn translates_the_zig_compatibility_syntax() {
        assert_eq!(translate("*"), b"(?s).*\\Z");
        assert_eq!(translate("?.py"), b"(?s).\\.py\\Z");
        assert_eq!(translate("[a-z]"), b"(?s)[a-z]\\Z");
    }
}
