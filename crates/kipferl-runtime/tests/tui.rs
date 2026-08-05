use std::process::{Command, Output};

#[test]
fn exposes_constants_styles_and_legacy_visible_width() {
    let output = run("import tui\n\
print([tui.BORDER_ROUNDED, tui.BORDER_SQUARE, tui.BORDER_DOUBLE, tui.BORDER_HEAVY, tui.BORDER_NONE, tui.ALIGN_LEFT, tui.ALIGN_RIGHT, tui.ALIGN_CENTER])\n\
print(repr(tui.style('Hi', fg='red', bg='#abc', bold=True, underline=True)))\n\
print(repr(tui.style('x', fg='purple')))\n\
print(tui.visible_len('\\x1b[31mé界🙂\\x1b[0m'))");

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
    let output = run("import tui\n\
tui.success('ok')\n\
tui.error('bad')\n\
tui.warning('hmm')\n\
tui.info('note')\n\
tui.rule()\n\
tui.rule('Title', width=12)\n\
tui.rule('T', color='=', align='red', width=8)");

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
    let titled = run("import tui; tui.box('Hi', title='T')");
    assert!(titled.status.success(), "{}", text(&titled.stderr));
    assert_eq!(
        text(&titled.stdout),
        "╭─\x1b[1m T \x1b[0m─╮\n│ Hi  │\n╰─────╯\n"
    );

    // The production Zig signature names these slots differently from the
    // callback. Keep that observable behavior until a deliberate API break.
    let keyword_bound = run(
        "import tui; tui.box('Hi', title='T', border_color='double', padding='red', border_style=2)",
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
    let progress = run("import tui\n\
tui.progress(5, 10, 'Load', 10, 'cyan', 1.25)\n\
tui.progress_done()\n\
tui.spinner(13, 'Wait', '#abc')\n\
tui.progress_done()\n\
print(repr(tui.spinner_frame(13)))");
    assert!(progress.status.success(), "{}", text(&progress.stderr));
    assert_eq!(
        text(&progress.stdout),
        "\rLoad \x1b[36m█████░░░░░\x1b[0m 50%  1.3s\x1b[K\n\
\r\x1b[38;2;170;187;204m⠸\x1b[0m Wait\x1b[K\n\
'⠸'\n"
    );

    let table = run("import tui; tui.table([['Name','界'],['Ana','7']], True)");
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
    let empty = run("import tui; print(tui.table([])); print(tui.table([[]]))");
    assert!(empty.status.success(), "{}", text(&empty.stderr));
    assert_eq!(text(&empty.stdout), "None\nNone\n");

    for (source, expected) in [
        (
            "import tui; tui.visible_len('x' * 4096)",
            "ValueError: text too long",
        ),
        (
            "import tui; tui.style(1)",
            "TypeError: text must be a string",
        ),
        (
            "import tui; tui.progress('1', 2)",
            "TypeError: current must be int",
        ),
        (
            "import tui; tui.spinner_frame('1')",
            "TypeError: index must be int",
        ),
        ("import tui; tui.table(1)", "TypeError: rows must be a list"),
        (
            "import tui; tui.table([1])",
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
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
