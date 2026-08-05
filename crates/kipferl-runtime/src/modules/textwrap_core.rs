const WRAP_BUFFER_LIMIT: usize = 4096;

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\t' | b'\r' | 0x0b | 0x0c)
}

fn collapse_whitespace(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut collapsed = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut seen_word = false;

    while index < bytes.len() {
        while index < bytes.len() && is_whitespace(bytes[index]) {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        let start = index;
        while index < bytes.len() && !is_whitespace(bytes[index]) {
            index += 1;
        }
        if seen_word {
            collapsed.push(b' ');
        }
        collapsed.extend_from_slice(&bytes[start..index]);
        seen_word = true;
    }
    collapsed
}

fn collapse_for_wrap(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut collapsed = Vec::with_capacity(bytes.len().min(WRAP_BUFFER_LIMIT));
    let mut index = 0;
    let mut need_space = false;

    while index < bytes.len() {
        while index < bytes.len() && is_whitespace(bytes[index]) {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        let start = index;
        while index < bytes.len() && !is_whitespace(bytes[index]) {
            index += 1;
        }
        let word = &bytes[start..index];
        if need_space && collapsed.len() + 1 + word.len() <= WRAP_BUFFER_LIMIT {
            collapsed.push(b' ');
        }
        need_space = true;
        if collapsed.len() + word.len() <= WRAP_BUFFER_LIMIT {
            collapsed.extend_from_slice(word);
        }
    }
    collapsed
}

pub(super) fn wrap(text: &str, width: i64) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let width = if width > 0 { width as usize } else { 70 };
    let collapsed = collapse_for_wrap(text);
    let mut lines = Vec::new();
    let mut index = 0;

    while index < collapsed.len() {
        while index < collapsed.len() && collapsed[index] == b' ' {
            index += 1;
        }
        if index == collapsed.len() {
            break;
        }
        let line_start = index;
        let mut line_end = index;

        while index < collapsed.len() {
            while index < collapsed.len() && collapsed[index] != b' ' {
                index += 1;
            }
            let word_end = index;
            if line_end == line_start || word_end - line_start <= width {
                line_end = word_end;
            } else {
                break;
            }
            while index < collapsed.len() && collapsed[index] == b' ' {
                index += 1;
            }
        }

        if line_end > line_start {
            lines.push(
                String::from_utf8(collapsed[line_start..line_end].to_vec())
                    .expect("text originated as UTF-8"),
            );
        }
    }
    lines
}

pub(super) fn fill(text: &str, width: i64) -> String {
    let width = if width > 0 { width as usize } else { 1 };
    let collapsed = collapse_whitespace(text);
    let mut lines: Vec<&[u8]> = Vec::new();
    let mut index = 0;
    let mut line_start = 0;
    let mut line_len = 0;

    while index < collapsed.len() {
        let word_start = index;
        while index < collapsed.len() && collapsed[index] != b' ' {
            index += 1;
        }
        let word_len = index - word_start;
        if line_len == 0 {
            line_start = word_start;
            line_len = word_len;
        } else if line_len + 1 + word_len <= width {
            line_len += 1 + word_len;
        } else {
            lines.push(&collapsed[line_start..line_start + line_len]);
            line_start = word_start;
            line_len = word_len;
        }
        if index < collapsed.len() {
            index += 1;
        }
    }
    if line_len > 0 {
        lines.push(&collapsed[line_start..line_start + line_len]);
    }

    let mut output = Vec::with_capacity(collapsed.len());
    for (line_index, line) in lines.iter().enumerate() {
        if line_index != 0 {
            output.push(b'\n');
        }
        output.extend_from_slice(line);
    }
    String::from_utf8(output).expect("text originated as UTF-8")
}

pub(super) fn dedent(text: &str) -> String {
    let indent = text
        .split('\n')
        .filter_map(|line| {
            let count = line
                .as_bytes()
                .iter()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
            (count != line.len()).then_some(count)
        })
        .min()
        .unwrap_or(0);

    text.split('\n')
        .map(|line| {
            let mut start = 0;
            while start < indent
                && start < line.len()
                && matches!(line.as_bytes()[start], b' ' | b'\t')
            {
                start += 1;
            }
            &line[start..]
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn indent(text: &str, prefix: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut output = String::with_capacity(text.len() + prefix.len());
    for (index, line) in text.split('\n').enumerate() {
        if index != 0 {
            output.push('\n');
        }
        output.push_str(prefix);
        output.push_str(line);
    }
    output
}

pub(super) fn shorten(text: &str, width: i64) -> String {
    if width <= 0 {
        return String::new();
    }
    let width = width as usize;
    let collapsed = collapse_whitespace(text);
    if collapsed.len() <= width {
        return String::from_utf8(collapsed).expect("text originated as UTF-8");
    }

    const PLACEHOLDER: &[u8] = b"...";
    if width <= PLACEHOLDER.len() {
        return String::from_utf8(PLACEHOLDER[..width].to_vec()).expect("ASCII placeholder");
    }
    let maximum_body = width - PLACEHOLDER.len();
    let mut output = Vec::with_capacity(width);
    let mut index = 0;

    while index < collapsed.len() {
        while index < collapsed.len() && collapsed[index] == b' ' {
            index += 1;
        }
        if index == collapsed.len() {
            break;
        }
        let start = index;
        while index < collapsed.len() && collapsed[index] != b' ' {
            index += 1;
        }
        let word = &collapsed[start..index];
        let needed = word.len() + usize::from(!output.is_empty());
        if output.len() + needed > maximum_body {
            break;
        }
        if !output.is_empty() {
            output.push(b' ');
        }
        output.extend_from_slice(word);
    }
    output.extend_from_slice(PLACEHOLDER);
    String::from_utf8(output).expect("text originated as UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_legacy_wrap_overflow_behavior() {
        assert_eq!(wrap("a bb ccc dddd", 1), ["a", "ccc"]);
        assert_eq!(wrap("  a\tb\nc\r\u{b} d\u{c}", 3), ["a b", "d"]);
        assert_eq!(wrap("abcdef gh", 3), ["abcdef"]);
    }

    #[test]
    fn preserves_fill_width_and_long_words() {
        assert_eq!(fill("a bb ccc", 0), "a\nbb\nccc");
        assert_eq!(fill("abcdef gh", 3), "abcdef\ngh");
    }

    #[test]
    fn preserves_indent_dedent_and_shorten_quirks() {
        assert_eq!(dedent("  a\n\tb\n    c\n"), " a\nb\n   c\n");
        assert_eq!(indent("a\n\nb\n", ">"), ">a\n>\n>b\n>");
        assert_eq!(shorten("a bb ccc", 1), ".");
        assert_eq!(shorten("a bb ccc", 4), "a...");
    }
}
