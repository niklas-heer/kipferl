// HTTP inputs cross an extern-C callback: keep conversions and indexing fallible.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::panic_in_result_fn
)]

use std::ffi::{c_int, c_void};
use std::time::Duration;

use kipferl_pocketpy_sys as ffi;
use ureq::{Agent, http};

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, dict_apply,
    execute_module, os_error, return_value, type_error, value_error,
};

const MAX_RESPONSE_SIZE: u64 = 8 * 1024 * 1024;

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
    _scheme = "http"

    def __init__(self, host, port=80, timeout=None):
        if not isinstance(host, str):
            raise TypeError("host must be str")
        self.host = host
        self.port = port
        self.timeout = timeout
        self._last_response = None

    def request(self, method, url, body=None, headers=None):
        result = _request(self._scheme, self.host, self.port, method, url, body, headers, self.timeout)
        self._last_response = HTTPResponse(result[0], result[1], result[2], result[3])

    def getresponse(self):
        if self._last_response is None:
            raise RuntimeError("no response available")
        response = self._last_response
        self._last_response = None
        return response


class HTTPSConnection(HTTPConnection):
    _scheme = "https"

    def __init__(self, host, port=443, timeout=None):
        HTTPConnection.__init__(self, host, port, timeout)
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

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
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

#[expect(
    clippy::too_many_lines,
    reason = "Keep ordered input validation, request execution, and bounded response construction together at the FFI boundary for review."
)]
unsafe extern "C" fn request(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(8, 8) {
        return false;
    }
    let Some(scheme) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"scheme must be str");
    };
    if !matches!(scheme.as_str(), "http" | "https") {
        return value_error(c"unsupported HTTP scheme");
    }
    let Some(host) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"host must be str");
    };
    let Some(port) = arguments.get(2).and_then(Value::integer) else {
        return type_error(c"port must be int");
    };
    let Ok(port) = u16::try_from(port) else {
        return value_error(c"port must be in range 0..65536");
    };
    let Some(method) = arguments.get(3).and_then(Value::string) else {
        return type_error(c"method must be str");
    };
    let Some(url) = arguments.get(4).and_then(Value::string) else {
        return type_error(c"url must be str");
    };
    if method.contains(['\r', '\n']) || url.contains(['\r', '\n']) {
        return value_error(c"invalid HTTP request line");
    }
    let body = match arguments.get(5) {
        Some(value) if value.is_none() => Vec::new(),
        Some(value) => match value
            .bytes()
            .or_else(|| value.string().map(String::into_bytes))
        {
            Some(body) => body,
            None => return type_error(c"body must be str, bytes, or None"),
        },
        _ => return type_error(c"body must be str, bytes, or None"),
    };

    let mut collector = HeaderCollector {
        valid: true,
        ..HeaderCollector::default()
    };
    if let Some(headers) = arguments.get(6)
        && !headers.is_none()
    {
        if !headers.is_type(ffi::py_PredefinedType_tp_dict) {
            return type_error(c"headers must be dict or None");
        }
        if !dict_apply(
            headers,
            collect_header,
            std::ptr::from_mut(&mut collector).cast(),
        ) {
            return false;
        }
        if !collector.valid {
            return type_error(c"header names and values must be scalar values without newlines");
        }
    }

    let timeout = arguments.get(7).and_then(|value| {
        if value.is_none() {
            None
        } else {
            value
                .number()
                .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
        }
    });
    if !arguments
        .get(7)
        .is_some_and(super::super::native::Value::is_none)
        && timeout.is_none()
    {
        return value_error(c"timeout must be a finite non-negative number or None");
    }

    let Some(endpoint) = endpoint(&scheme, &host, port, &url) else {
        return value_error(c"invalid HTTP URL");
    };
    let Ok(method) = http::Method::from_bytes(method.as_bytes()) else {
        return value_error(c"invalid HTTP method");
    };
    let mut builder = http::Request::builder().method(method).uri(endpoint);
    for (name, value) in collector.headers {
        builder = builder.header(name, value);
    }
    let Ok(request) = builder.body(body) else {
        return value_error(c"invalid HTTP request");
    };

    let mut config = Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .proxy(None)
        .user_agent("")
        .accept("");
    if let Some(seconds) = timeout {
        let Ok(duration) = Duration::try_from_secs_f64(seconds) else {
            return value_error(c"timeout is too large");
        };
        // The HTTP client computes deadlines using Instant addition. A valid
        // Duration can still exceed the platform clock's representable range.
        if std::time::Instant::now().checked_add(duration).is_none() {
            return value_error(c"timeout is too large");
        }
        config = config.timeout_global(Some(duration));
    }
    let agent: Agent = config.build().into();
    let Ok(mut response) = agent.run(request) else {
        return os_error(c"HTTP request failed");
    };
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("").to_owned();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_ascii_lowercase(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();
    let Ok(body) = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_SIZE)
        .read_to_vec()
    else {
        return os_error(c"failed to read HTTP response");
    };
    return_response(Response {
        status: i64::from(status.as_u16()),
        reason,
        headers,
        body,
    })
}

fn request_path(url: &str) -> &str {
    if let Some(rest) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    {
        return rest
            .find('/')
            .and_then(|index| rest.get(index..))
            .unwrap_or("/");
    }
    if url.is_empty() {
        "/"
    } else if url.starts_with('/') {
        url
    } else {
        ""
    }
}

fn endpoint(scheme: &str, host: &str, port: u16, url: &str) -> Option<String> {
    let path = request_path(url);
    if path.is_empty() || host.is_empty() {
        return None;
    }
    let host = if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        format!("[{host}]")
    } else {
        host.to_owned()
    };
    Some(format!("{scheme}://{host}:{port}{path}"))
}

struct Response {
    status: i64,
    reason: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
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
    use super::{endpoint, request_path};

    #[test]
    fn builds_http_and_https_endpoints() {
        assert_eq!(request_path("http://example.com/a?q=1"), "/a?q=1");
        assert_eq!(request_path("https://example.com/a?q=1"), "/a?q=1");
        assert_eq!(request_path("http://example.com"), "/");
        assert_eq!(
            endpoint("https", "example.com", 443, "/health"),
            Some("https://example.com:443/health".into())
        );
        assert_eq!(
            endpoint("http", "::1", 8080, "/health"),
            Some("http://[::1]:8080/health".into())
        );
        assert_eq!(endpoint("http", "", 80, "/"), None);
        assert_eq!(endpoint("http", "example.com", 80, "health"), None);
    }
}
