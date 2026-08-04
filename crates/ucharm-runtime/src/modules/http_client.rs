use std::ffi::{c_int, c_void};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, dict_apply,
    execute_module, os_error, return_value, type_error, value_error,
};

const MAX_RESPONSE_SIZE: usize = 8 * 1024 * 1024;

const COMPATIBILITY_SOURCE: &str = r#"
class HTTPResponse:
    def __init__(self, status=0, reason="", headers=None, body=None):
        self.status = status
        self.reason = reason
        self.headers = {} if headers is None else headers
        self.body = b"" if body is None else body

    def read(self):
        return self.body

    def getheader(self, name, default=None):
        return self.headers.get(name.lower(), default)


class HTTPConnection:
    def __init__(self, host, port=80, timeout=None):
        if not isinstance(host, str):
            raise TypeError("host must be str")
        self.host = host
        self.port = port
        self.timeout = timeout
        self._last_response = None

    def request(self, method, url, body=None, headers=None):
        result = _request(self.host, self.port, method, url, body, headers, self.timeout)
        self._last_response = HTTPResponse(result[0], result[1], result[2], result[3])

    def getresponse(self):
        if self._last_response is None:
            raise RuntimeError("no response available")
        response = self._last_response
        self._last_response = None
        return response
"#;

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_request",
    callback: request,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"http.client",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: module initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded http.client compatibility layer failed");
    }
}

#[derive(Default)]
struct HeaderCollector {
    headers: Vec<(String, String)>,
    valid: bool,
}

unsafe extern "C" fn collect_header(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: `dict_apply` passes the live dictionary values and our collector.
    let key = unsafe { Value::from_raw(key) };
    // SAFETY: same lifetime argument as `key` above.
    let value = unsafe { Value::from_raw(value) };
    // SAFETY: `request` supplies this exact collector for the synchronous walk.
    let collector = unsafe { &mut *context.cast::<HeaderCollector>() };
    let Some(key) = key.string() else {
        collector.valid = false;
        return true;
    };
    let value = if let Some(value) = value.string() {
        value
    } else if let Some(value) = value.integer() {
        value.to_string()
    } else if let Some(value) = value.number() {
        value.to_string()
    } else if let Some(value) = value.boolean() {
        value.to_string()
    } else {
        collector.valid = false;
        return true;
    };
    if key.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
        collector.valid = false;
        return true;
    }
    collector.headers.push((key, value));
    true
}

unsafe extern "C" fn request(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from PocketPy with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(7, 7) {
        return false;
    }
    let Some(host) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"host must be str");
    };
    let Some(port) = arguments.get(1).and_then(Value::integer) else {
        return type_error(c"port must be int");
    };
    let Ok(port) = u16::try_from(port) else {
        return value_error(c"port must be in range 0..65536");
    };
    let Some(method) = arguments.get(2).and_then(Value::string) else {
        return type_error(c"method must be str");
    };
    let Some(url) = arguments.get(3).and_then(Value::string) else {
        return type_error(c"url must be str");
    };
    if method.contains(['\r', '\n']) || url.contains(['\r', '\n']) {
        return value_error(c"invalid HTTP request line");
    }
    let body = match arguments.get(4) {
        Some(value) if value.is_none() => Vec::new(),
        Some(value) if value.is_type(ffi::py_PredefinedType_tp_bytes) => {
            value.bytes().expect("bytes checked")
        }
        Some(value) if value.is_type(ffi::py_PredefinedType_tp_str) => {
            value.string().expect("string checked").into_bytes()
        }
        _ => return type_error(c"body must be str, bytes, or None"),
    };

    let mut collector = HeaderCollector {
        valid: true,
        ..HeaderCollector::default()
    };
    if let Some(headers) = arguments.get(5)
        && !headers.is_none()
    {
        if !headers.is_type(ffi::py_PredefinedType_tp_dict) {
            return type_error(c"headers must be dict or None");
        }
        if !dict_apply(
            headers,
            collect_header,
            (&mut collector as *mut HeaderCollector).cast(),
        ) {
            return false;
        }
        if !collector.valid {
            return type_error(c"header names and values must be scalar values without newlines");
        }
    }

    let timeout = arguments.get(6).and_then(|value| {
        if value.is_none() {
            None
        } else {
            value.number().filter(|seconds| *seconds >= 0.0)
        }
    });
    let path = request_path(&url);
    let mut encoded = Vec::with_capacity(256 + body.len());
    if write!(&mut encoded, "{method} {path} HTTP/1.1\r\n").is_err() {
        return value_error(c"request is too large");
    }
    if !has_header(&collector.headers, "host") && write!(&mut encoded, "Host: {host}\r\n").is_err()
    {
        return value_error(c"request is too large");
    }
    if !has_header(&collector.headers, "connection") {
        encoded.extend_from_slice(b"Connection: close\r\n");
    }
    if !body.is_empty() && !has_header(&collector.headers, "content-length") {
        let _ = write!(&mut encoded, "Content-Length: {}\r\n", body.len());
    }
    for (name, value) in &collector.headers {
        let _ = write!(&mut encoded, "{name}: {value}\r\n");
    }
    encoded.extend_from_slice(b"\r\n");
    encoded.extend_from_slice(&body);

    let Ok(mut addresses) = (host.as_str(), port).to_socket_addrs() else {
        return os_error(c"failed to resolve HTTP host");
    };
    let Some(address) = addresses.next() else {
        return os_error(c"HTTP host has no addresses");
    };
    let connected = timeout
        .map(|seconds| TcpStream::connect_timeout(&address, Duration::from_secs_f64(seconds)))
        .unwrap_or_else(|| TcpStream::connect(address));
    let Ok(mut stream) = connected else {
        return os_error(c"failed to connect HTTP socket");
    };
    if let Some(seconds) = timeout {
        let duration = Some(Duration::from_secs_f64(seconds));
        let _ = stream.set_read_timeout(duration);
        let _ = stream.set_write_timeout(duration);
    }
    if stream.write_all(&encoded).is_err() {
        return os_error(c"failed to send HTTP request");
    }

    let Ok(response) = read_response(&mut stream, !method.eq_ignore_ascii_case("HEAD")) else {
        return os_error(c"failed to read HTTP response");
    };
    return_response(response)
}

fn request_path(url: &str) -> &str {
    if let Some(rest) = url.strip_prefix("http://") {
        return rest.find('/').map_or("/", |index| &rest[index..]);
    }
    if url.is_empty() { "/" } else { url }
}

fn has_header(headers: &[(String, String)], needle: &str) -> bool {
    headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case(needle))
}

struct Response {
    status: i64,
    reason: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn read_response(stream: &mut impl Read, expects_body: bool) -> Result<Response, ()> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = stream.read(&mut buffer).map_err(|_| ())?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > MAX_RESPONSE_SIZE {
            return Err(());
        }
        if response_is_complete(&bytes, expects_body) {
            break;
        }
    }
    let header_end = find_bytes(&bytes, b"\r\n\r\n").ok_or(())?;
    let head = std::str::from_utf8(&bytes[..header_end]).map_err(|_| ())?;
    let mut lines = head.split("\r\n");
    let mut status_parts = lines.next().ok_or(())?.splitn(3, ' ');
    let protocol = status_parts.next().ok_or(())?;
    if !protocol.starts_with("HTTP/") {
        return Err(());
    }
    let status = status_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let reason = status_parts.next().unwrap_or("").to_owned();
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<Vec<_>>();
    let encoded_body = &bytes[header_end + 4..];
    let body = if !expects_body || (100..200).contains(&status) || matches!(status, 204 | 304) {
        Vec::new()
    } else if headers
        .iter()
        .any(|(name, value)| name == "transfer-encoding" && value.eq_ignore_ascii_case("chunked"))
    {
        decode_chunked(encoded_body)?
    } else if let Some(length) = content_length(&headers) {
        encoded_body.get(..length).ok_or(())?.to_vec()
    } else {
        encoded_body.to_vec()
    };
    Ok(Response {
        status,
        reason,
        headers,
        body,
    })
}

fn response_is_complete(bytes: &[u8], expects_body: bool) -> bool {
    let Some(header_end) = find_bytes(bytes, b"\r\n\r\n") else {
        return false;
    };
    let Ok(head) = std::str::from_utf8(&bytes[..header_end]) else {
        return false;
    };
    let status = head
        .split("\r\n")
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<i64>().ok());
    if !expects_body
        || status.is_some_and(|status| (100..200).contains(&status) || matches!(status, 204 | 304))
    {
        return true;
    }
    let headers = head
        .split("\r\n")
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<Vec<_>>();
    let body = &bytes[header_end + 4..];
    if let Some(length) = content_length(&headers) {
        return body.len() >= length;
    }
    headers
        .iter()
        .any(|(name, value)| name == "transfer-encoding" && value.eq_ignore_ascii_case("chunked"))
        && find_bytes(body, b"\r\n0\r\n\r\n").is_some()
}

fn content_length(headers: &[(String, String)]) -> Option<usize> {
    headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse().ok())
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn decode_chunked(mut encoded: &[u8]) -> Result<Vec<u8>, ()> {
    let mut decoded = Vec::new();
    loop {
        let line_end = find_bytes(encoded, b"\r\n").ok_or(())?;
        let line = std::str::from_utf8(&encoded[..line_end]).map_err(|_| ())?;
        let size =
            usize::from_str_radix(line.split(';').next().ok_or(())?.trim(), 16).map_err(|_| ())?;
        encoded = &encoded[line_end + 2..];
        if size == 0 {
            return Ok(decoded);
        }
        let chunk = encoded.get(..size).ok_or(())?;
        if decoded.len().saturating_add(chunk.len()) > MAX_RESPONSE_SIZE {
            return Err(());
        }
        decoded.extend_from_slice(chunk);
        encoded = encoded.get(size + 2..).ok_or(())?;
    }
}

fn return_response(response: Response) -> bool {
    let mut roots = RootFrame::new();
    let Some(output) = roots.tuple(4) else {
        return value_error(c"HTTP response is too large");
    };
    let status = roots.integer(response.status);
    output.tuple_set(0, status);
    let Some(reason) = roots.string(&response.reason) else {
        return value_error(c"HTTP reason is too large");
    };
    output.tuple_set(1, reason);
    let headers = roots.dict();
    for (name, value) in response.headers {
        let Some(name) = roots.string(&name) else {
            return value_error(c"HTTP header is too large");
        };
        let Some(value) = roots.string(&value) else {
            return value_error(c"HTTP header is too large");
        };
        if !headers.dict_set(name, value) {
            return false;
        }
    }
    output.tuple_set(2, headers);
    let Some(body) = roots.bytes(&response.body) else {
        return value_error(c"HTTP body is too large");
    };
    output.tuple_set(3, body);
    return_value(output)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{decode_chunked, read_response, request_path, response_is_complete};

    #[test]
    fn parses_content_length_response() {
        let bytes = b"HTTP/1.1 201 Created\r\nContent-Length: 4\r\nX-Test: yes\r\n\r\nrust";
        let response = read_response(&mut Cursor::new(bytes), true).expect("valid response");
        assert_eq!(response.status, 201);
        assert_eq!(response.reason, "Created");
        assert_eq!(response.body, b"rust");
        assert_eq!(response.headers[1], ("x-test".into(), "yes".into()));
    }

    #[test]
    fn parses_chunked_response_and_absolute_url() {
        let encoded = b"4\r\nrust\r\n5\r\n-http\r\n0\r\n\r\n";
        assert_eq!(decode_chunked(encoded).unwrap(), b"rust-http");
        let mut response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        response.extend_from_slice(encoded);
        assert!(response_is_complete(&response, true));
        assert_eq!(request_path("http://example.com/a?q=1"), "/a?q=1");
        assert_eq!(request_path("http://example.com"), "/");
    }

    #[test]
    fn accepts_head_and_no_content_responses_without_a_body() {
        let head = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n";
        let response = read_response(&mut Cursor::new(head), false).expect("valid HEAD response");
        assert!(response.body.is_empty());

        let no_content = b"HTTP/1.1 204 No Content\r\nContent-Length: 10\r\n\r\n";
        let response =
            read_response(&mut Cursor::new(no_content), true).expect("valid no-content response");
        assert!(response.body.is_empty());
    }
}
