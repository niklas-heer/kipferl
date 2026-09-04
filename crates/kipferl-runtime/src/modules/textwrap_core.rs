const WRAP_BUFFER_LIMIT: usize = 4096;

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| character.is_ascii_whitespace() || character == '\u{b}')
        .filter(|word| !word.is_empty())
}

fn collapse_whitespace(text: &str) -> String {
    words(text).collect::<Vec<_>>().join(" ")
}

fn collapse_for_wrap(text: &str) -> String {
    let mut collapsed = String::with_capacity(text.len().min(WRAP_BUFFER_LIMIT));
    let mut need_space = false;
    for word in words(text) {
        if need_space
            && collapsed.len().saturating_add(1).saturating_add(word.len()) <= WRAP_BUFFER_LIMIT
        {
            collapsed.push(' ');
        }
        need_space = true;
        if collapsed.len().saturating_add(word.len()) <= WRAP_BUFFER_LIMIT {
            collapsed.push_str(word);
        }
    }
    collapsed
}

pub(super) fn wrap(text: &str, width: i64) -> Vec<String> {
    let width = usize::try_from(width)
        .ok()
        .filter(|width| *width > 0)
        .unwrap_or(70);
    let collapsed = collapse_for_wrap(text);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in words(&collapsed) {
        if line.is_empty() {
            line.push_str(word);
        } else if line.len().saturating_add(1).saturating_add(word.len()) <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            // The legacy wrap API consumes the overflowing word without emitting it.
            lines.push(std::mem::take(&mut line));
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub(super) fn fill(text: &str, width: i64) -> String {
    let width = usize::try_from(width)
        .ok()
        .filter(|width| *width > 0)
        .unwrap_or(1);
    let mut output = String::with_capacity(text.len());
    let mut line_len: usize = 0;
    for word in words(text) {
        if line_len != 0 {
            if line_len.saturating_add(1).saturating_add(word.len()) <= width {
                output.push(' ');
                line_len = line_len.saturating_add(1);
            } else {
                output.push('\n');
                line_len = 0;
            }
        }
        output.push_str(word);
        line_len = line_len.saturating_add(word.len());
    }
    output
}

pub(super) fn dedent(text: &str) -> String {
    let indent = text
        .split('\n')
        .filter_map(|line| {
            let count = line
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
            (count != line.len()).then_some(count)
        })
        .min()
        .unwrap_or(0);
    text.split('\n')
        .map(|line| {
            let count = line
                .bytes()
                .take(indent)
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
            line.get(count..).unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn indent(text: &str, prefix: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut output = String::with_capacity(text.len().saturating_add(prefix.len()));
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
    const PLACEHOLDER: &str = "...";
    let Ok(width) = usize::try_from(width) else {
        return String::new();
    };
    if width == 0 {
        return String::new();
    }
    let collapsed = collapse_whitespace(text);
    if collapsed.len() <= width {
        return collapsed;
    }
    if width <= PLACEHOLDER.len() {
        return ".".repeat(width);
    }
    let maximum_body = width.saturating_sub(PLACEHOLDER.len());
    let mut output = String::new();
    for word in words(&collapsed) {
        let needed = word.len().saturating_add(usize::from(!output.is_empty()));
        if output.len().saturating_add(needed) > maximum_body {
            break;
        }
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(word);
    }
    output.push_str(PLACEHOLDER);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_and_extreme_widths_preserve_whole_words() {
        assert_eq!(fill("é\u{b}界 🙂", i64::MAX), "é 界 🙂");
        assert_eq!(wrap("é 界 🙂", i64::MAX), ["é 界 🙂"]);
        assert_eq!(shorten("é 界 🙂", i64::MAX), "é 界 🙂");
        assert_eq!(shorten("é 界 🙂", 5), "é...");
        assert_eq!(dedent("  é\n  界"), "é\n界");
    }

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
