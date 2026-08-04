use std::process::{Command, Output};

#[test]
fn exposes_the_native_ansi_module_to_python() {
    let output = run("import ansi\n\
print(repr(ansi.reset()))\n\
print(repr(ansi.fg('red')))\n\
print(repr(ansi.fg('bright_red')))\n\
print(repr(ansi.fg(196)))\n\
print(repr(ansi.bg('#f50')))\n\
print(repr(ansi.rgb(1, 2, 3)))\n\
print(repr(ansi.rgb(1, 2, 3, True)))\n\
print(repr(ansi.rgb(256, -1, 511)))\n\
print(repr(ansi.strikethrough()))");

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "'\\x1b[0m'\n\
'\\x1b[31m'\n\
'\\x1b[91m'\n\
'\\x1b[38;5;196m'\n\
'\\x1b[48;2;255;85;0m'\n\
'\\x1b[38;2;1;2;3m'\n\
'\\x1b[48;2;1;2;3m'\n\
'\\x1b[38;2;0;255;255m'\n\
'\\x1b[9m'\n"
    );
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_ansi_argument_errors() {
    for (source, expected) in [
        ("import ansi; ansi.fg()", "TypeError: too few arguments"),
        (
            "import ansi; ansi.fg([])",
            "TypeError: color must be a string or int",
        ),
        (
            "import ansi; ansi.rgb('1', 2, 3)",
            "TypeError: r must be int",
        ),
        ("import ansi; ansi.bold(1)", "TypeError: too many arguments"),
    ] {
        let output = run(source);
        assert_eq!(output.status.code(), Some(1));
        assert!(text(&output.stdout).contains(expected));
        assert!(text(&output.stderr).contains("Python execution failed"));
    }
}

fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm-rs"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
