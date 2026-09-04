use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kipferl-diagnostics-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kipferl"))
            .args(arguments)
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("cache"))
            .output()
            .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn output(result: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    )
}

#[test]
fn runtime_errors_name_original_file_and_line_even_after_multiline_legacy_import() {
    let work = Directory::new();
    fs::write(
        work.0.join("app.py"),
        "from kipferl import (\n    success,\n)\nraise ValueError('look here')\n",
    )
    .unwrap();
    let result = work.run(&["run", "app.py"]);
    assert!(!result.status.success());
    let text = output(&result);
    assert!(text.contains("app.py\", line 4"), "{text}");
    assert!(text.contains("ValueError: look here"), "{text}");
}

#[test]
fn syntax_errors_name_the_original_file_and_line() {
    let work = Directory::new();
    fs::write(work.0.join("syntax.py"), "name = 'hello'\nif :\n    pass\n").unwrap();
    let result = work.run(&["run", "syntax.py"]);
    assert!(!result.status.success());
    let text = output(&result);
    assert!(text.contains("syntax.py\", line 2"), "{text}");
    assert!(text.contains("SyntaxError"), "{text}");
}

#[test]
fn script_identity_and_resource_paths_refer_to_original_source() {
    let work = Directory::new();
    fs::create_dir(work.0.join("project")).unwrap();
    fs::write(work.0.join("project/data.txt"), "the resource").unwrap();
    fs::write(work.0.join("project/app.py"), "import os, sys\nassert sys.argv[0] == __file__\nwith open(os.path.join(os.path.dirname(__file__), 'data.txt'), 'r') as resource:\n    print(resource.read())\nprint(sys.argv[1])\n").unwrap();
    let result = work.run(&["run", "project/app.py", "argument"]);
    assert!(result.status.success(), "{}", output(&result));
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "the resource\nargument\n"
    );
}

#[test]
fn unreadable_source_diagnostics_include_the_actionable_cause() {
    let work = Directory::new();
    fs::write(work.0.join("invalid.py"), [0xff]).unwrap();
    let result = work.run(&["run", "invalid.py"]);
    assert!(!result.status.success());
    let text = output(&result);
    assert!(text.contains("invalid.py"), "{text}");
    assert!(text.contains("utf-8") || text.contains("UTF-8"), "{text}");
}

#[test]
fn sys_exit_preserves_status_without_an_internal_traceback() {
    let work = Directory::new();
    for (argument, status, message) in [
        ("None", 0, ""),
        ("0", 0, ""),
        ("7", 7, ""),
        ("'Please check the input'", 1, "Please check the input"),
    ] {
        fs::write(
            work.0.join("exit.py"),
            format!("import sys\nsys.exit({argument})\n"),
        )
        .unwrap();
        let result = work.run(&["run", "exit.py"]);
        assert_eq!(result.status.code(), Some(status), "{}", output(&result));
        let text = output(&result);
        assert!(!text.contains("Traceback"), "{text}");
        assert!(text.contains(message), "{text}");
    }
}

#[test]
fn imported_module_errors_point_to_original_source_from_another_directory() {
    let work = Directory::new();
    fs::create_dir(work.0.join("project")).unwrap();
    fs::write(
        work.0.join("project/helper.py"),
        "value = 42\nraise ValueError('helper failure')\n",
    )
    .unwrap();
    fs::write(work.0.join("project/app.py"), "import helper\n").unwrap();
    let result = work.run(&["run", "project/app.py"]);
    assert!(!result.status.success());
    let text = output(&result);
    assert!(text.contains("helper.py\", line 2"), "{text}");
    assert!(text.contains("ValueError: helper failure"), "{text}");
}

#[test]
fn legacy_import_spans_preserve_inline_and_following_statements() {
    let work = Directory::new();
    for source in [
        "from kipferl import success; print('preserved')\n",
        "if True: from kipferl import success; print('preserved')\n",
        "if False: from kipferl import success; print('unexpected')\nprint('preserved')\n",
        "from kipferl import success, \\\n    info\nprint('preserved')\n",
        "from kipferl import (\n    success,\n); print('preserved')\n",
        "if True: from kipferl import (\n    success,\n); print('preserved')\n",
        "from\tkipferl\timport success; from kipferl import info; print('preserved')\n",
    ] {
        fs::write(work.0.join("inline.py"), source).unwrap();
        let result = work.run(&["run", "inline.py"]);
        assert!(
            result.status.success(),
            "source: {source}\n{}",
            output(&result)
        );
        assert_eq!(String::from_utf8_lossy(&result.stdout), "preserved\n");
    }
}

#[test]
fn continued_legacy_imports_preserve_trailing_exception_line_numbers() {
    let work = Directory::new();
    for (source, line) in [
        (
            "from kipferl import (\n    success,\n); raise ValueError('after import')\n",
            3,
        ),
        (
            "if True: from kipferl import (\n    success,\n); raise ValueError('after import')\n",
            3,
        ),
        (
            "from kipferl import success, \\\n    info; raise ValueError('after import')\n",
            2,
        ),
    ] {
        fs::write(work.0.join("continued.py"), source).unwrap();
        let result = work.run(&["run", "continued.py"]);
        assert!(!result.status.success());
        let text = output(&result);
        assert!(
            text.contains(&format!("continued.py\", line {line}")),
            "{text}"
        );
        assert!(text.contains("ValueError: after import"), "{text}");
    }
}

#[test]
fn import_looking_literals_remain_data_next_to_legacy_imports() {
    let work = Directory::new();
    fs::write(work.0.join("literal.py"), "text = \"\"\"if True: from kipferl import success; raise ValueError('data')\nfrom kipferl import (\n\"\"\"\nfrom kipferl import success; assert 'raise ValueError' in text\nassert text.endswith('from kipferl import (\\n')\nprint('literal preserved')\n").unwrap();
    let result = work.run(&["run", "literal.py"]);
    assert!(result.status.success(), "{}", output(&result));
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "literal preserved\n"
    );
}
