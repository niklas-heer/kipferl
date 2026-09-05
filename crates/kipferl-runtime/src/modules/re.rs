use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;
use regex_lite::{Captures, Regex};

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value, execute_module,
    return_string, return_string_list, return_value, type_error, value_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"_captures(pattern, text, anchored=False)",
        callback: captures,
    },
    NativeSignature {
        signature: c"_all_captures(pattern, text)",
        callback: all_captures,
    },
    NativeSignature {
        signature: c"_sub(pattern, replacement, text, count=0)",
        callback: substitute,
    },
    NativeSignature {
        signature: c"_split(pattern, text, maxsplit=0)",
        callback: split,
    },
];

const COMPATIBILITY_SOURCE: &str = r#"
class Match:
    def __init__(self, text, spans):
        self._text = text
        self._spans = spans

    def group(self, index=0):
        span = self._spans[index]
        if span[0] < 0:
            return None
        return self._text[span[0]:span[1]]

    def groups(self):
        values = []
        for index in range(1, len(self._spans)):
            values.append(self.group(index))
        return tuple(values)

    def start(self, index=0):
        return self._spans[index][0]

    def end(self, index=0):
        return self._spans[index][1]

    def span(self, index=0):
        return self._spans[index]


class Pattern:
    def __init__(self, pattern):
        self.pattern = pattern

    def search(self, text):
        return _make_match(text, _captures(self.pattern, text, False))

    def findall(self, text):
        return _findall(self.pattern, text)

    def sub(self, replacement, text, count=0):
        return _sub(self.pattern, replacement, text, count)

    def split(self, text, maxsplit=0):
        return _split(self.pattern, text, maxsplit)


def _make_match(text, spans):
    if spans is None:
        return None
    return Match(text, spans)


def _pattern_match(self, text):
    return _make_match(text, _captures(self.pattern, text, True))


def _findall(pattern, text):
    results = []
    for spans in _all_captures(pattern, text):
        if len(spans) == 1:
            results.append(text[spans[0][0]:spans[0][1]])
        elif len(spans) == 2:
            span = spans[1]
            results.append("" if span[0] < 0 else text[span[0]:span[1]])
        else:
            groups = []
            for index in range(1, len(spans)):
                span = spans[index]
                groups.append("" if span[0] < 0 else text[span[0]:span[1]])
            results.append(tuple(groups))
    return results


def _module_match(pattern, text):
    return _make_match(text, _captures(pattern, text, True))


def search(pattern, text):
    return _make_match(text, _captures(pattern, text, False))


def findall(pattern, text):
    return _findall(pattern, text)


def sub(pattern, replacement, text, count=0):
    return _sub(pattern, replacement, text, count)


def split(pattern, text, maxsplit=0):
    return _split(pattern, text, maxsplit)


def compile(pattern):
    _captures(pattern, "", False)
    return Pattern(pattern)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"re",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
#[expect(
    clippy::expect_used,
    reason = "The just-executed checked-in module source defines match, Pattern, and Pattern.match before lookup."
)]
fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded re compatibility layer failed");
    }
    let module_match = module
        .attribute(c"_module_match")
        .expect("embedded re match function exists");
    module.set_attribute(c"match", module_match);
    let pattern = module
        .attribute(c"Pattern")
        .expect("embedded re Pattern class exists");
    let pattern_match = module
        .attribute(c"_pattern_match")
        .expect("embedded re Pattern.match function exists");
    pattern.set_attribute(c"match", pattern_match);
}

fn compile_pattern(pattern: &str) -> Result<Regex, regex_lite::Error> {
    Regex::new(&normalize_octal_escapes(pattern))
}

fn normalize_octal_escapes(pattern: &str) -> String {
    let mut output = String::with_capacity(pattern.len());
    let mut characters = pattern.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let Some(escaped) = characters.next() else {
            output.push('\\');
            break;
        };
        // Python treats a zero-prefixed escape, or exactly three octal
        // digits, as an octal character both inside and outside classes.
        // Short nonzero forms stay untouched: outside classes they are
        // numeric backreferences, which regex-lite deliberately rejects.
        let octal = escaped == '0'
            || (matches!(escaped, '1'..='7')
                && characters
                    .clone()
                    .take(2)
                    .filter(|digit| matches!(digit, '0'..='7'))
                    .count()
                    == 2);
        if !octal {
            output.push('\\');
            output.push(escaped);
            continue;
        }
        let mut original = String::from("\\");
        original.push(escaped);
        let mut value = escaped.to_digit(8).unwrap_or(0);
        for _ in 0..2 {
            let Some(digit) = characters.next_if(|digit| matches!(digit, '0'..='7')) else {
                break;
            };
            original.push(digit);
            value = (value << 3) | digit.to_digit(8).unwrap_or(0);
        }
        if value > 0o377 {
            // Preserve invalid octal text so the engine rejects it in an
            // expression, but still ignores it inside verbose comments.
            output.push_str(&original);
            continue;
        }
        // Hex escapes preserve regex metacharacters and verbose whitespace.
        // Both nibbles are bounded to 0..=15 by the octal range check above.
        output.push_str("\\x");
        output.push(char::from_digit(value >> 4, 16).unwrap_or('0'));
        output.push(char::from_digit(value & 15, 16).unwrap_or('0'));
    }
    output
}

unsafe extern "C" fn captures(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    let Some(text) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    let anchored = arguments.get(2).and_then(Value::boolean).unwrap_or(false);
    let Ok(regex) = compile_pattern(&pattern) else {
        return value_error(c"invalid regular expression");
    };
    let Some(found) = regex.captures(&text) else {
        return return_none();
    };
    if anchored && found.get(0).is_none_or(|matched| matched.start() != 0) {
        return return_none();
    }
    return_spans(&regex, &found)
}

unsafe extern "C" fn all_captures(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    let Some(text) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    let Ok(regex) = compile_pattern(&pattern) else {
        return value_error(c"invalid regular expression");
    };
    let mut roots = RootFrame::new();
    let output = roots.list();
    for found in regex.captures_iter(&text) {
        let Some(spans) = spans_value(&mut roots, &regex, &found) else {
            return value_error(c"too many regular expression captures");
        };
        output.list_append(spans);
    }
    return_value(output)
}

#[expect(
    clippy::expect_used,
    clippy::string_slice,
    reason = "regex-lite guarantees capture zero exists and match offsets are ordered UTF-8 boundaries within the unchanged input string."
)]
unsafe extern "C" fn substitute(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    let Some(replacement) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"replacement must be a string");
    };
    let Some(text) = arguments.get(2).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    let count = arguments.get(3).and_then(Value::integer).unwrap_or(0);
    if count < 0 {
        return return_string(&text);
    }
    let Ok(regex) = compile_pattern(&pattern) else {
        return value_error(c"invalid regular expression");
    };
    let limit = usize::try_from(count).unwrap_or(usize::MAX);
    let mut output = String::new();
    let mut previous = 0;
    for (replaced, found) in regex.captures_iter(&text).enumerate() {
        if limit != 0 && replaced >= limit {
            break;
        }
        let matched = found.get(0).expect("capture zero always exists");
        output.push_str(&text[previous..matched.start()]);
        expand_replacement(&replacement, &found, &mut output);
        previous = matched.end();
    }
    output.push_str(&text[previous..]);
    return_string(&output)
}

#[expect(
    clippy::string_slice,
    reason = "regex-lite match offsets are ordered UTF-8 boundaries within the unchanged input string."
)]
unsafe extern "C" fn split(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    let Some(text) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    let maxsplit = arguments.get(2).and_then(Value::integer).unwrap_or(0);
    if maxsplit < 0 {
        return return_string_list(&[text]);
    }
    let Ok(regex) = compile_pattern(&pattern) else {
        return value_error(c"invalid regular expression");
    };
    let limit = usize::try_from(maxsplit).unwrap_or(usize::MAX);
    let mut pieces = Vec::new();
    let mut previous = 0;
    for (splits, matched) in regex.find_iter(&text).enumerate() {
        if limit != 0 && splits >= limit {
            break;
        }
        pieces.push(text[previous..matched.start()].to_owned());
        previous = matched.end();
    }
    pieces.push(text[previous..].to_owned());
    return_string_list(&pieces)
}

fn return_spans(regex: &Regex, captures: &Captures<'_>) -> bool {
    let mut roots = RootFrame::new();
    let Some(spans) = spans_value(&mut roots, regex, captures) else {
        return value_error(c"too many regular expression captures");
    };
    return_value(spans)
}

fn spans_value(roots: &mut RootFrame, regex: &Regex, captures: &Captures<'_>) -> Option<Value> {
    let output = roots.list();
    for index in 0..regex.captures_len() {
        let span = roots.tuple(2)?;
        let (start, end) = captures.get(index).map_or((-1, -1), |matched| {
            (
                i64::try_from(matched.start()).unwrap_or(i64::MAX),
                i64::try_from(matched.end()).unwrap_or(i64::MAX),
            )
        });
        let start = roots.integer(start);
        let end = roots.integer(end);
        if !span.tuple_set(0, start) || !span.tuple_set(1, end) {
            return None;
        }
        output.list_append(span);
    }
    Some(output)
}

fn expand_replacement(replacement: &str, captures: &Captures<'_>, output: &mut String) {
    let mut characters = replacement.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let Some(escaped) = characters.next() else {
            output.push('\\');
            break;
        };
        if let Some(group) = escaped
            .to_digit(10)
            .and_then(|group| usize::try_from(group).ok())
        {
            if let Some(capture) = captures.get(group) {
                output.push_str(capture.as_str());
            }
            continue;
        }
        match escaped {
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'f' => output.push('\x0c'),
            'v' => output.push('\x0b'),
            'a' => output.push('\x07'),
            'b' => output.push('\x08'),
            '\\' => output.push('\\'),
            value if !value.is_ascii() => {
                output.push('\\');
                output.push(value);
            }
            value => output.push(value),
        }
    }
}

fn return_none() -> bool {
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}
