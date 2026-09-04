use std::process::{Command, Output};

#[test]
fn passes_all_tooling_format_wave_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "argparse",
            include_str!("../../../tests/cpython/test_argparse.py"),
            "Results: 26 passed, 0 failed, 0 skipped",
        ),
        (
            "configparser",
            include_str!("../../../tests/cpython/test_configparser.py"),
            "Results: 26 passed, 0 failed, 0 skipped",
        ),
        (
            "contextlib",
            include_str!("../../../tests/cpython/test_contextlib.py"),
            "Results: 10 passed, 0 failed, 4 skipped",
        ),
        (
            "unittest",
            include_str!("../../../tests/cpython/test_unittest.py"),
            "Results: 40 passed, 0 failed, 0 skipped",
        ),
        (
            "urllib.parse",
            include_str!("../../../tests/cpython/test_urllib_parse.py"),
            "Results: 24 passed, 0 failed, 0 skipped",
        ),
        (
            "tomllib",
            include_str!("../../../tests/cpython/test_tomllib.py"),
            "Results: 4 passed, 0 failed, 0 skipped",
        ),
        (
            "toml",
            include_str!("../../../tests/cpython/test_toml.py"),
            "Results: 9 passed, 0 failed, 0 skipped",
        ),
        (
            "xml.etree.ElementTree",
            include_str!("../../../tests/cpython/test_xml_etree_elementtree.py"),
            "Results: 12 passed, 0 failed, 0 skipped",
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
fn matches_cpython_for_deterministic_tooling_and_format_operations() {
    let source = concat!(
        "import argparse, configparser, tomllib\n",
        "from urllib.parse import quote, unquote, urljoin, urlparse\n",
        "from xml.etree.ElementTree import Element, SubElement, tostring\n",
        "parser = argparse.ArgumentParser(description='demo')\n",
        "parser.add_argument('--count', type=int, default=2)\n",
        "parser.add_argument('name')\n",
        "args = parser.parse_args(['--count', '3', 'alice'])\n",
        "print(args.name, args.count, parser.description)\n",
        "config = configparser.ConfigParser()\n",
        "config.read_string('[core]\\ncount = 3\\nenabled = yes\\n')\n",
        "print(config.sections(), config.getint('core', 'count'), config.getboolean('core', 'enabled'))\n",
        "url = 'https://example.com/a b?q=one#two'\n",
        "parsed = urlparse(url)\n",
        "print(parsed.scheme, parsed.netloc, parsed.path, parsed.query, parsed.fragment)\n",
        "print(unquote(quote('μ charm')), urljoin('https://example.com/a/', 'b'))\n",
        "document = tomllib.loads(\"name='kipferl'\\n[tool]\\ncount=3\\n\")\n",
        "print(document['name'], document['tool']['count'])\n",
        "root = Element('root', {'kind': 'demo'})\n",
        "SubElement(root, 'child').text = 'a < b'\n",
        "print(tostring(root, 'unicode'))\n",
    );
    let rust = run(source);
    assert!(rust.status.success(), "{}", diagnostic(&rust));
    let cpython = Command::new("python3")
        .args(["-c", source])
        .output()
        .expect("run CPython differential oracle");
    assert!(cpython.status.success(), "{}", diagnostic(&cpython));
    assert_eq!(rust.stdout, cpython.stdout);
    assert_eq!(text(&rust.stderr), "");
}

#[test]
fn preserves_state_and_nested_formats_under_stress() {
    let output = run(concat!(
        "import argparse, configparser, contextlib, tomllib, unittest\n",
        "from urllib.parse import quote, unquote\n",
        "from xml.etree.ElementTree import fromstring, tostring\n",
        "closed = []\n",
        "@contextlib.contextmanager\n",
        "def managed(value):\n",
        "    yield value\n",
        "    closed.append(value)\n",
        "case = unittest.TestCase()\n",
        "for i in range(250):\n",
        "    parser = argparse.ArgumentParser()\n",
        "    parser.add_argument('--value', type=int, required=True)\n",
        "    parser.add_argument('items', nargs='+')\n",
        "    args = parser.parse_args(['--value', str(i), 'a', 'b'])\n",
        "    assert args.value == i and args.items == ['a', 'b']\n",
        "    config = configparser.ConfigParser()\n",
        "    config.read_string('[item]\\nvalue=' + str(i) + '\\nenabled=true\\n')\n",
        "    assert config.getint('item', 'value') == i and config.getboolean('item', 'enabled')\n",
        "    encoded = quote('μ value ' + str(i))\n",
        "    assert unquote(encoded) == 'μ value ' + str(i)\n",
        "    assert unquote('%Aμ') == '%Aμ'\n",
        "    data = tomllib.loads(\"title='a#b' # comment\\n[outer.inner]\\nvalue=\" + str(i))\n",
        "    assert data['title'] == 'a#b' and data['outer']['inner']['value'] == i\n",
        "    xml = fromstring('<root id=\"' + str(i) + '\"><child>a &lt; b</child></root>')\n",
        "    assert xml.attrib['id'] == str(i) and list(xml)[0].text == 'a < b'\n",
        "    assert 'a &lt; b' in tostring(xml, 'unicode')\n",
        "    with managed(i) as value:\n",
        "        case.assertEqual(value, i)\n",
        "assert len(closed) == 250 and closed[-1] == 249\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn rejects_invalid_arguments_formats_and_assertions() {
    let output = run(concat!(
        "import argparse, configparser, tomllib, unittest\n",
        "from xml.etree.ElementTree import fromstring\n",
        "def must_fail(expected, operation):\n",
        "    try:\n",
        "        operation()\n",
        "    except expected:\n",
        "        return\n",
        "    raise AssertionError('operation unexpectedly succeeded')\n",
        "parser = argparse.ArgumentParser()\n",
        "parser.add_argument('--mode', choices=['a', 'b'], required=True)\n",
        "must_fail(SystemExit, lambda: parser.parse_args([]))\n",
        "must_fail(SystemExit, lambda: parser.parse_args(['--mode', 'c']))\n",
        "must_fail(SystemExit, lambda: parser.parse_args(['--mode', 'a', 'extra']))\n",
        "must_fail(ValueError, lambda: configparser.ConfigParser().read_string('missing header = true'))\n",
        "must_fail(Exception, lambda: tomllib.loads('value=nope'))\n",
        "must_fail(Exception, lambda: fromstring('<root><child></root>'))\n",
        "case = unittest.TestCase()\n",
        "must_fail(AssertionError, lambda: case.assertEqual(1, 2))\n",
        "case.assertRaises(ValueError, lambda: int('not-an-int'))\n",
    ));
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
