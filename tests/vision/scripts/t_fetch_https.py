def run(fn):
    try:
        fn()
        print("ok")
        return True
    except Exception as exc:
        print("fail:", type(exc).__name__, str(exc))
        return False


CERT_PEM = """-----BEGIN CERTIFICATE-----
MIICpDCCAYwCCQCsKROtYwoR8jANBgkqhkiG9w0BAQsFADAUMRIwEAYDVQQDDAls
b2NhbGhvc3QwHhcNMjUxMjIxMDAzMjA0WhcNMzUxMjE5MDAzMjA0WjAUMRIwEAYD
VQQDDAlsb2NhbGhvc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDQ
VKtKQWi6RujdFVgs8pTo2VlgUMj8vjmP3O7s21nSeHp4GZGuuD5FyggnC2GxWYg7
6oTjZV88c2oqpraYj+gtMqA27jCyX76dA8Fgy/u22H3cdl7DunYP+x5BkftA5gR2
Xn/IdwgopYBQnVtvoaaIPEJH/H6ap0u16NT4/MwyUDU6zKcxYtToWuMJvUKGP2OO
HMKIw4u2Hqk9Q/XoD1oi6QrZfwD2vvK7KhjZhA4j9nohm7nOm6SfPpJiODD36nH/
2frJ/VmGW09Now4RdPEqAjcq4Dh6OZDHBkMzUozXuN7TShJaHeSe0nG3uzQp+/z4
JeROBu0qJj1MIUTR6iI7AgMBAAEwDQYJKoZIhvcNAQELBQADggEBAI05K+EvlYLj
3iWMM0R2FOKwf8PyHAoQ4gOBTICWAWfooNyUokg6u0UG2atcVsqbQMib5JM8Hmgu
2Q9MzYExahf9j2luG/+UhjQQBwzK1hEW8siQK0TesUiVrHR6nUCfKQLAkX5zvZpZ
4ovJMWGJLa8LpqptrDgRXuPWiCmndPZISxiaxB0dTWOhyLc3jlzRFqJtNP9IZ518
a55Sc7moDJXE85Qs2REdc+EaxQZQxqBW/04+xH/D7o6INtF3mI/4qQwUOwIWM3xI
1py9tQnXIMT8PE1rUjZKU3Uj4MUEu9rRcJmI6YUOE1HPvJVjyYhNogKFzrlru3Wi
Tyb8bwmFIjE=
-----END CERTIFICATE-----"""

KEY_PEM = """-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDQVKtKQWi6Rujd
FVgs8pTo2VlgUMj8vjmP3O7s21nSeHp4GZGuuD5FyggnC2GxWYg76oTjZV88c2oq
praYj+gtMqA27jCyX76dA8Fgy/u22H3cdl7DunYP+x5BkftA5gR2Xn/IdwgopYBQ
nVtvoaaIPEJH/H6ap0u16NT4/MwyUDU6zKcxYtToWuMJvUKGP2OOHMKIw4u2Hqk9
Q/XoD1oi6QrZfwD2vvK7KhjZhA4j9nohm7nOm6SfPpJiODD36nH/2frJ/VmGW09N
ow4RdPEqAjcq4Dh6OZDHBkMzUozXuN7TShJaHeSe0nG3uzQp+/z4JeROBu0qJj1M
IUTR6iI7AgMBAAECggEAVwrpPm8xyJKT/LtMSgtYuCkHtLxMHX2FI1yV3xSO5Oc9
lCxqt+t26UXOPjH9MVJNH2uO9kuGjQVT2FordHa79RZv5kOCySRDyeqlw0G2++Bc
Rd6XHYQsi+TJ8W+C4My6FJLLJQDGweTURdpZN7z8jXNP5i/S3d8EPQ743McSsfoc
GGNgFiTvrKJ/Y+Dpg8iJl1QvzyVvYv2smRziikcIlALcJTMdlohkW5IJP4BpVngU
db34xxbZeMqpxg5fXrmTJvCmLheWZCanhe2uyyNl7zWF9OQcgwUQov5JUYehbyk+
5a54EfcrGHgXvk3L2IgQpZ8h9W7/c46lVSJqoFQREQKBgQDs7DM+wug9gvQZ4pWE
LsJkKwdi0WVXBsEJ3aWgpB671llDKEpBpX8ze+lxTIeqb6mUv4ilKr8jze79JEYT
ySjWNDQByyYGeu791sXBrmOWhqKdROtn/ADW4o9oAWxHBMCYnkZkoN7Bwrj3dSsF
GXMAoCpEn67gRJWNKrUO36suDwKBgQDhGxcOjYI415wdB9BaUULevakjBwfCD0wf
6vH5bTYX+oSr9j9cVM1JJfMt5zV8kKEWGBl9RDjxYqvMW7GNwWr2FdhyqLyxfiWV
zTPT8VxmQ05Zrxz79M+WkhIIDdvZRbv9cWD3omd/AkoN18fnWmUJTfs3YEAbrpdH
vUa9yIX1FQKBgQCp8EJRojwy4tt0NbJJPcDxWGvT1Z567b1I9lL3BsGEuhsMsLmS
nMLAiwDG473r4mwg5cF9t0uiwvPJX1tklcVU39zt7Gk5/LOwH315jzyfm7LIW8b+
ryNq/tceIucniaEb12tmgn1FPgaueLyCy95RdJDc6CzncEpVF20HXifKwQKBgEZJ
Ph7GIoX7FHygBvdcbiO8VoZgWJTIT/2bT2iRKBW+nBRRdCExPVP8rHyFt9aoFhQe
/D53wcvlAj1x1/OqE+q4kXfjpd9JwxSOGQOxVid8Foe8PLGTFAowm762DRI/St5s
u1k29Vfb8CF4YaukNu370lfNDtdV4Vh+CguSA/mtAoGAGBOwgeDCNbMRIIv1NF6m
R49lBm9QzzzjKj0nzMSVxfphXJ0XF6RcKfVOB3XmzVZ3izsi8oQMjsaWgq7qibGP
iNEdD4+1STPVowFLCd4X1vdYXML+eDifR1MoRb9B9ftX22VfkdNlt0F7e+95RIS5
XgqTDYGAFlwQoZM4yxeP6rE=
-----END PRIVATE KEY-----"""


def _start_https_server(cert_path, key_path):
    import subprocess

    code = r"""
import http.server, ssl, sys

cert, key = sys.argv[1], sys.argv[2]

class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def do_GET(self):
        if self.headers.get("X-Test") != "1":
            self.send_response(400, "Bad Request")
            self.send_header("Content-Length", "0")
            self.send_header("Connection", "close")
            self.end_headers()
            self.close_connection = True
            return
        body = b"hello"
        self.send_response(200, "OK")
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)
        self.close_connection = True
    def do_POST(self):
        n = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(n) if n else b""
        self.send_response(200, "OK")
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)
        self.close_connection = True
    def log_message(self, fmt, *args):
        pass

httpd = http.server.HTTPServer(("127.0.0.1", 0), Handler)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(cert, key)
httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
print(httpd.server_port, flush=True)
httpd.serve_forever()
    """

    return subprocess.Popen(
        ["python3", "-u", "-c", code, cert_path, key_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


def main():
    import os

    import fetch

    cert_path = "__kipferl_https_cert.pem"
    key_path = "__kipferl_https_key.pem"

    with open(cert_path, "w") as f:
        f.write(CERT_PEM)
    with open(key_path, "w") as f:
        f.write(KEY_PEM)

    proc = _start_https_server(cert_path, key_path)
    try:
        port_line = proc.readline().strip()
        if not port_line:
            raise ValueError("server did not print a port")
        port = int(port_line)
        url = f"https://localhost:{port}/"

        r = fetch.get(url, verify=False, headers={"X-Test": "1"})
        if r.get("status") != 200:
            raise ValueError(f"GET verify=False failed: {r}")
        if r["body"].decode() != "hello":
            raise ValueError(f"GET body mismatch: {r['body']!r}")

        r = fetch.get(url, verify=True, cafile=cert_path, headers={"X-Test": "1"})
        if r.get("status") != 200:
            raise ValueError(f"GET verify=True failed: {r}")

        r = fetch.post(url, json={"x": 1}, verify=False, headers={"X-Test": "1"})
        if r.get("status") != 200:
            raise ValueError(f"POST verify=False failed: {r}")
        if '"x"' not in r["body"].decode():
            raise ValueError(f"POST body missing key: {r['body']!r}")
    except Exception:
        proc.terminate()
        proc.kill()
        os.remove(cert_path)
        os.remove(key_path)
        raise

    proc.terminate()
    proc.kill()
    os.remove(cert_path)
    os.remove(key_path)


if __name__ == "__main__":
    run(main)
