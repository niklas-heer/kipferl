use std::process::{Command, Output};

#[test]
fn exposes_argument_accessors_and_positional_filtering() {
    let output = run(
        "import args, sys\n\
assert args.raw() is sys.argv\n\
assert args.count() == 11\n\
assert args.get(0) == '-c'\n\
assert args.get(-1) == 'tail'\n\
marker = []\n\
assert args.get(99, marker) is marker\n\
assert args.get(99) is None\n\
assert args.has('--name')\n\
assert not args.has('--missing')\n\
assert args.value('--name') == 'alice'\n\
assert args.value('--count') == '42'\n\
assert args.value('--missing', marker) is marker\n\
assert args.int_value('--count') == 42\n\
assert args.int_value('--bad', 9) == 9\n\
assert args.int_value('--missing') == 0\n\
assert args.positional() == ['alpha', 'tail']",
        &[
            "alpha",
            "--name",
            "alice",
            "--count=42",
            "--bad",
            "nope",
            "-v",
            "file",
            "--",
            "tail",
        ],
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn parses_aliases_types_defaults_negation_and_positionals() {
    let output = run(
        "import args\n\
spec = {\n\
    '--name': str,\n\
    '--count': (int, 7),\n\
    '--verbose': bool,\n\
    '--color': bool,\n\
    '--mode': (str, 'safe'),\n\
    '--switch': (bool,),\n\
    '-n': '--name',\n\
    '-v': '--verbose',\n\
}\n\
parsed = args.parse(spec)\n\
assert parsed == {\n\
    '_': ['first', 'orphan', 'file', 'tail'],\n\
    'name': 'alice',\n\
    'count': 42,\n\
    'verbose': True,\n\
    'color': False,\n\
    'mode': 'safe',\n\
    'switch': False,\n\
}",
        &[
            "first",
            "-n",
            "alice",
            "--count=42",
            "--unknown",
            "orphan",
            "-v",
            "file",
            "--no-color",
            "--",
            "tail",
        ],
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_invalid_integer_and_missing_value_behavior() {
    let output = run(
        "import args\n\
parsed = args.parse({\n\
    '--count': (int, 7),\n\
    '--name': (str, 'fallback'),\n\
    '--empty': str,\n\
})\n\
assert parsed == {\n\
    '_': [],\n\
    'empty': '',\n\
    'count': 7,\n\
    'name': 'fallback',\n\
}",
        &["--count", "not-an-int", "--empty="],
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn roots_constructed_values_across_repeated_allocations() {
    let output = run(
        "import args\nspec = {'--name': str, '--count': (int, 0), '--verbose': bool}\nfor i in range(2000):\n    parsed = args.parse(spec)\n    assert parsed['name'] == 'alice'\n    assert parsed['count'] == 42\n    assert parsed['verbose'] is True\n    assert parsed['_'] == ['tail']",
        &["--name=alice", "--count", "42", "--verbose", "tail"],
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_argument_errors() {
    for (source, expected) in [
        ("import args; args.raw(1)", "TypeError: too many arguments"),
        ("import args; args.get()", "TypeError: too few arguments"),
        ("import args; args.get('0')", "TypeError: index must be int"),
        (
            "import args; args.has(1)",
            "TypeError: flag must be a string",
        ),
        (
            "import args; args.parse([])",
            "TypeError: spec must be a dict",
        ),
    ] {
        let output = run(source, &[]);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            text(&output.stdout).contains(expected),
            "{}",
            diagnostic(&output)
        );
        assert!(text(&output.stderr).contains("Python execution failed"));
    }
}

fn run(source: &str, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .args(arguments)
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(output: &impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(output.as_ref()).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
