use std::process::{Command, Output};

#[test]
fn exposes_constants_styles_and_legacy_visible_width() {
    let output = run("import charm\n\
print([charm.BORDER_ROUNDED, charm.BORDER_SQUARE, charm.BORDER_DOUBLE, charm.BORDER_HEAVY, charm.BORDER_NONE, charm.ALIGN_LEFT, charm.ALIGN_RIGHT, charm.ALIGN_CENTER])\n\
print(repr(charm.style('Hi', fg='red', bg='#abc', bold=True, underline=True)))\n\
print(repr(charm.style('x', fg='purple')))\n\
print(charm.visible_len('\\x1b[31mé界🙂\\x1b[0m'))");

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "[0, 1, 2, 3, 4, 0, 1, 2]\n\
'\\x1b[1;4;31;48;2;170;187;204mHi\\x1b[0m'\n\
'x'\n\
5\n"
    );
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn renders_status_messages_and_rules_byte_for_byte() {
    let output = run("import charm\n\
charm.success('ok')\n\
charm.error('bad')\n\
charm.warning('hmm')\n\
charm.info('note')\n\
charm.rule()\n\
charm.rule('Title', width=12)\n\
charm.rule('T', color='=', align='red', width=8)");

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        output.stdout,
        b"\x1b[1;32m\xe2\x9c\x93 \x1b[0mok\n\
\x1b[1;31m\xe2\x9c\x97 \x1b[0mbad\n\
\x1b[1;33m\xe2\x9a\xa0 \x1b[0mhmm\n\
\x1b[1;34m\xe2\x84\xb9 \x1b[0mnote\n\
\n\
\xe2\x94\x80\xe2\x94\x80 Title \xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\n\
\x1b[31m==\x1b[0m T \x1b[31m===\x1b[0m\n"
    );
}

#[test]
fn renders_boxes_and_preserves_the_zig_keyword_binding() {
    let titled = run("import charm; charm.box('Hi', title='T')");
    assert!(titled.status.success(), "{}", text(&titled.stderr));
    assert_eq!(
        text(&titled.stdout),
        "╭─\x1b[1m T \x1b[0m─╮\n│ Hi  │\n╰─────╯\n"
    );

    // The production Zig signature names these slots differently from the
    // callback. Keep that observable behavior until a deliberate API break.
    let keyword_bound = run(
        "import charm; charm.box('Hi', title='T', border_color='double', padding='red', border_style=2)",
    );
    assert!(
        keyword_bound.status.success(),
        "{}",
        text(&keyword_bound.stderr)
    );
    assert_eq!(
        text(&keyword_bound.stdout),
        "\x1b[31m╔═\x1b[0m\x1b[1m T \x1b[0m\x1b[31m═══╗\x1b[0m\n\
\x1b[31m║\x1b[0m  Hi   \x1b[31m║\x1b[0m\n\
\x1b[31m╚═══════╝\x1b[0m\n"
    );
}

#[test]
fn renders_progress_spinners_and_tables_byte_for_byte() {
    let progress = run("import charm\n\
charm.progress(5, 10, 'Load', 10, 'cyan', 1.25)\n\
charm.progress_done()\n\
charm.spinner(13, 'Wait', '#abc')\n\
charm.progress_done()\n\
print(repr(charm.spinner_frame(13)))");
    assert!(progress.status.success(), "{}", text(&progress.stderr));
    assert_eq!(
        text(&progress.stdout),
        "\rLoad \x1b[36m█████░░░░░\x1b[0m 50%  1.3s\x1b[K\n\
\r\x1b[38;2;170;187;204m⠸\x1b[0m Wait\x1b[K\n\
'⠸'\n"
    );

    let table = run("import charm; charm.table([['Name','界'],['Ana','7']], True)");
    assert!(table.status.success(), "{}", text(&table.stderr));
    assert_eq!(
        text(&table.stdout),
        "┌──────┬────┐\n\
│ \x1b[1mName\x1b[0m │ \x1b[1m界\x1b[0m │\n\
├──────┼────┤\n\
│ Ana  │ 7  │\n\
└──────┴────┘\n"
    );
}

#[test]
fn preserves_empty_results_and_argument_errors() {
    let empty = run("import charm; print(charm.table([])); print(charm.table([[]]))");
    assert!(empty.status.success(), "{}", text(&empty.stderr));
    assert_eq!(text(&empty.stdout), "None\nNone\n");

    for (source, expected) in [
        (
            "import charm; charm.visible_len('x' * 4096)",
            "ValueError: text too long",
        ),
        (
            "import charm; charm.style(1)",
            "TypeError: text must be a string",
        ),
        (
            "import charm; charm.progress('1', 2)",
            "TypeError: current must be int",
        ),
        (
            "import charm; charm.spinner_frame('1')",
            "TypeError: index must be int",
        ),
        (
            "import charm; charm.table(1)",
            "TypeError: rows must be a list",
        ),
        (
            "import charm; charm.table([1])",
            "TypeError: each row must be a list",
        ),
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
