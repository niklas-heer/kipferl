use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    return_string, type_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"quote(string, safe='/')",
        callback: quote,
    },
    NativeSignature {
        signature: c"unquote(string)",
        callback: unquote,
    },
];

const SOURCE: &str = r#"
class ParseResult:
    def __init__(self, scheme, netloc, path, params, query, fragment):
        self.scheme = scheme
        self.netloc = netloc
        self.path = path
        self.params = params
        self.query = query
        self.fragment = fragment

    def __getitem__(self, i):
        return (self.scheme, self.netloc, self.path, self.params, self.query, self.fragment)[i]

    def __iter__(self):
        return iter((self.scheme, self.netloc, self.path, self.params, self.query, self.fragment))

def urlparse(url, scheme='', allow_fragments=True):
    fragment = ''
    if allow_fragments and '#' in url:
        index = url.index('#')
        fragment = url[index + 1:]
        url = url[:index]
    query = ''
    if '?' in url:
        index = url.index('?')
        query = url[index + 1:]
        url = url[:index]
    parsed_scheme = scheme
    had_slashes = '://' in url
    if had_slashes:
        index = url.index('://')
        parsed_scheme = url[:index]
        url = url[index + 3:]
    netloc = ''
    path = url
    if had_slashes:
        if '/' in url:
            index = url.index('/')
            netloc = url[:index]
            path = url[index:]
        else:
            netloc = url
            path = ''
    return ParseResult(parsed_scheme, netloc, path, '', query, fragment)

def urlunparse(components):
    if hasattr(components, 'scheme'):
        scheme, netloc, path = components.scheme, components.netloc, components.path
        params, query, fragment = components.params, components.query, components.fragment
    else:
        scheme, netloc, path, params, query, fragment = components
    result = (scheme + '://' if scheme else '') + netloc + path
    if params:
        result += ';' + params
    if query:
        result += '?' + query
    if fragment:
        result += '#' + fragment
    return result

def urljoin(base, url, allow_fragments=True):
    if '://' in url:
        return url
    parsed = urlparse(base)
    if url.startswith('/'):
        return urlunparse((parsed.scheme, parsed.netloc, url, '', '', ''))
    path = parsed.path
    slash = 0
    for i in range(len(path)):
        if path[i] == '/':
            slash = i
    return urlunparse((parsed.scheme, parsed.netloc, path[:slash + 1] + url, '', '', ''))

def urlencode(query, doseq=False, safe='', encoding=None, errors=None, quote_via=None):
    items = list(query.items()) if hasattr(query, 'items') else list(query)
    parts = []
    for key, value in items:
        parts.append(quote(str(key), '').replace('%20', '+') + '=' + quote(str(value), '').replace('%20', '+'))
    return '&'.join(parts)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"urllib.parse",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, SOURCE),
        "embedded urllib.parse module"
    );
}

unsafe extern "C" fn quote(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(input) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"quote() requires a string");
    };
    let safe = arguments
        .get(1)
        .and_then(Value::string)
        .unwrap_or_else(|| "/".to_owned());
    let mut output = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric()
            || b"_.-~".contains(&byte)
            || safe.as_bytes().contains(&byte)
        {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            let _ = write!(output, "%{byte:02X}");
        }
    }
    return_string(&output)
}

unsafe extern "C" fn unquote(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(input) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"unquote() requires a string");
    };
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            output.push((high << 4) | low);
            index += 3;
            continue;
        }
        output.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    let Ok(output) = String::from_utf8(output) else {
        return type_error(c"decoded URL is not UTF-8");
    };
    return_string(&output)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
