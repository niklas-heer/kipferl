struct CharacterClass {
    matched: bool,
    consumed: usize,
}

fn match_character_class(pattern: &[u8], character: u8) -> CharacterClass {
    let unmatched = CharacterClass {
        matched: false,
        consumed: 0,
    };
    let Some((&b'[', mut remaining)) = pattern.split_first() else {
        return unmatched;
    };
    let negated = matches!(remaining.first(), Some(b'!' | b'^'));
    if negated {
        remaining = remaining.get(1..).unwrap_or_default();
    }
    let mut first = true;
    let mut matched = false;
    while let Some((&byte, tail)) = remaining.split_first() {
        if byte == b']' && !first {
            return CharacterClass {
                matched: matched != negated,
                consumed: pattern.len().saturating_sub(tail.len()),
            };
        }
        first = false;
        if let [b'-', end, rest @ ..] = tail
            && *end != b']'
        {
            matched |= (byte..=*end).contains(&character);
            remaining = rest;
            continue;
        }
        matched |= byte == character;
        remaining = tail;
    }
    unmatched
}

pub(super) fn matches(pattern: &str, text: &str) -> bool {
    let mut pattern = pattern.as_bytes();
    let mut text = text.as_bytes();
    let mut star_pattern = None;
    let mut star_text: Option<&[u8]> = None;
    while let Some((&character, text_tail)) = text.split_first() {
        if let Some((&token, pattern_tail)) = pattern.split_first() {
            if token == b'[' {
                let class = match_character_class(pattern, character);
                if class.consumed > 0 && class.matched {
                    pattern = pattern.get(class.consumed..).unwrap_or_default();
                    text = text_tail;
                    continue;
                }
                if class.consumed == 0 && character == token {
                    pattern = pattern_tail;
                    text = text_tail;
                    continue;
                }
            } else if token == character || token == b'?' {
                pattern = pattern_tail;
                text = text_tail;
                continue;
            } else if token == b'*' {
                star_pattern = Some(pattern_tail);
                star_text = Some(text);
                pattern = pattern_tail;
                continue;
            }
        }
        match (star_pattern, star_text.and_then(<[u8]>::split_first)) {
            (Some(retry_pattern), Some((_, retry_text))) => {
                pattern = retry_pattern;
                text = retry_text;
                star_text = Some(retry_text);
            }
            _ => return false,
        }
    }
    pattern.iter().all(|token| *token == b'*')
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
