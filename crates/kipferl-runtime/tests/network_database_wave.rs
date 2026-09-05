use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};

#[test]
fn passes_all_network_database_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "http.client",
            include_str!("../../../tests/cpython/test_http_client.py"),
            "Results: 9 passed, 0 failed, 0 skipped",
        ),
        (
            "sqlite3",
            include_str!("../../../tests/cpython/test_sqlite3.py"),
            "Results: 2 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run(source);
        assert!(
            output.status.success(),
            "{module} fixture failed:\n{}",
            diagnostic(&output)
        );
        assert!(
            text(&output.stdout).contains(summary),
            "{module} summary missing:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{module}");
    }
}

#[test]
fn sends_and_parses_a_real_loopback_http_exchange() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback server");
    let port = listener.local_addr().expect("loopback address").port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept HTTP request");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("set server timeout");
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = stream.read(&mut buffer).expect("read HTTP request");
            assert_ne!(count, 0, "client closed before sending headers");
            request.extend_from_slice(&buffer[..count]);
        }
        let request = String::from_utf8(request).expect("ASCII HTTP request");
        assert!(request.starts_with("GET /health HTTP/1.1\r\n"));
        assert!(request.to_ascii_lowercase().contains("x-kipferl: yes\r\n"));
        stream
            .write_all(
                b"HTTP/1.1 201 Created\r\nTransfer-Encoding: chunked\r\nX-Test: value\r\n\r\n4\r\nrust\r\n8\r\n-http-ok\r\n0\r\n\r\n",
            )
            .expect("write HTTP response");
    });

    let output = run(&format!(
        concat!(
            "import http.client as http\n",
            "connection = http.HTTPConnection('127.0.0.1', {port}, 2)\n",
            "connection.request('GET', '/health', None, {{'X-Kipferl': 'yes'}})\n",
            "response = connection.getresponse()\n",
            "assert response.status == 201 and response.reason == 'Created'\n",
            "assert response.getheader('X-Test') == 'value'\n",
            "assert response.read() == b'rust-http-ok'\n",
        ),
        port = port,
    ));
    server.join().expect("loopback server completed");
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_sqlite_types_persistence_lifecycle_and_errors() {
    let path = std::env::temp_dir().join(format!(
        "kipferl-network-database-{}-{}.sqlite3",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let source = format!(
        concat!(
            "import sqlite3\n",
            "path = {path:?}\n",
            "connection = sqlite3.connect(path)\n",
            "cursor = connection.cursor()\n",
            "cursor.execute('CREATE TABLE values_table (n INTEGER, f REAL, t TEXT, b BLOB, z TEXT)')\n",
            "cursor.execute('INSERT INTO values_table VALUES (?, ?, ?, ?, ?)', (7, 1.5, 'rust', b'bytes', None))\n",
            "connection.commit()\n",
            "cursor.close()\n",
            "connection.close()\n",
            "connection = sqlite3.connect(path)\n",
            "cursor = connection.execute('SELECT n, f, t, b, z FROM values_table')\n",
            "assert cursor.fetchone() == (7, 1.5, 'rust', b'bytes', None)\n",
            "assert cursor.fetchone() is None\n",
            "cursor.close()\n",
            "failed = False\n",
            "try:\n",
            "    cursor.fetchone()\n",
            "except RuntimeError:\n",
            "    failed = True\n",
            "assert failed\n",
            "connection.close()\n",
        ),
        path = path.to_string_lossy(),
    );
    let output = run(&source);
    let _ = std::fs::remove_file(&path);
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
