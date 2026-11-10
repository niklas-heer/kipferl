//! Remove Kipferl's own bootstrap frames from uncaught-exception tracebacks.
//!
//! `kipferl run`, `kipferl dev`, and built applications execute a generated
//! bootstrap that compiles the user's source under its original filename and
//! runs it with `exec(__kipferl_code)`. Bundled applications also load local
//! modules through a generated helper module. Those frames describe Kipferl,
//! not the program, so they are dropped; every remaining frame keeps the exact
//! bytes `PocketPy` produced, including file names and line numbers.

/// The only statement the generated bootstraps use to run user code.
const BOOTSTRAP_STATEMENT: &str = "exec(__kipferl_code)";
/// The generated module that loads bundled local modules.
const BUNDLE_HELPER_FILE: &str = "__kipferl_bundle_runtime.py";

const TRACEBACK_HEADER: &str = "Traceback (most recent call last):";
const FRAME_PREFIX: &str = "  File \"";
const SOURCE_PREFIX: &str = "    ";

/// Return `traceback` without frames that belong to Kipferl's bootstraps.
///
/// Only frame lines directly following a `Traceback` header are considered,
/// so exception messages are never altered, even when they resemble frames.
pub fn hide_bootstrap_frames(traceback: &str) -> String {
    let mut output = String::with_capacity(traceback.len());
    let mut lines = traceback.split_inclusive('\n').peekable();
    let mut in_frames = false;
    while let Some(line) = lines.next() {
        let Some(location) = line.strip_prefix(FRAME_PREFIX).filter(|_| in_frames) else {
            in_frames = line.trim_end_matches('\n') == TRACEBACK_HEADER;
            output.push_str(line);
            continue;
        };
        let source = lines.next_if(|next| next.starts_with(SOURCE_PREFIX));
        if is_bootstrap_frame(location, source) {
            continue;
        }
        output.push_str(line);
        if let Some(source) = source {
            output.push_str(source);
        }
    }
    output
}

fn is_bootstrap_frame(location: &str, source: Option<&str>) -> bool {
    let file = location
        .split_once("\", line ")
        .map_or(location, |(file, _)| file);
    let helper = file == BUNDLE_HELPER_FILE
        || file
            .strip_suffix(BUNDLE_HELPER_FILE)
            .is_some_and(|directory| directory.ends_with('/'));
    helper || source.is_some_and(|source| source.trim() == BOOTSTRAP_STATEMENT)
}

#[cfg(test)]
mod tests {
    use super::hide_bootstrap_frames;

    #[test]
    fn removes_bootstrap_and_helper_frames_and_keeps_user_frames_verbatim() {
        let traceback = "Traceback (most recent call last):\n  File \"/tmp/kipferl-run/script-1.py\", line 36\n    exec(__kipferl_code)\n  File \"importer.py\", line 2\n    import broken\n  File \"__kipferl_bundle_runtime.py\", line 23, in load\n    module = __import__(name, None, None, ['__name__'])\n  File \"/tmp/root/__kipferl_bundle_runtime.py\", line 21, in load\n    load('.'.join(parts[:-1]))\n  File \"broken.py\", line 7\n        exec(__kipferl_code)\n  File \"broken.py\", line 2, in fail\n    raise RuntimeError('at import')\nRuntimeError: at import";
        assert_eq!(
            hide_bootstrap_frames(traceback),
            "Traceback (most recent call last):\n  File \"importer.py\", line 2\n    import broken\n  File \"broken.py\", line 2, in fail\n    raise RuntimeError('at import')\nRuntimeError: at import"
        );
    }

    #[test]
    fn handles_chained_tracebacks_and_frames_without_source() {
        let traceback = "Traceback (most recent call last):\n  File \"app.py\", line 2\n    {}['k']\nKeyError: 'k'\n\nDuring handling of the above exception, another exception occurred:\n\nTraceback (most recent call last):\n  File \"a.py\", line 9\n    exec(__kipferl_code)\n  File \"<generated>\", line 4\n  File \"app.py\", line 4\n    raise ValueError('second')\nValueError: second";
        assert_eq!(
            hide_bootstrap_frames(traceback),
            "Traceback (most recent call last):\n  File \"app.py\", line 2\n    {}['k']\nKeyError: 'k'\n\nDuring handling of the above exception, another exception occurred:\n\nTraceback (most recent call last):\n  File \"<generated>\", line 4\n  File \"app.py\", line 4\n    raise ValueError('second')\nValueError: second"
        );
    }

    #[test]
    fn leaves_user_code_and_exception_messages_untouched() {
        for traceback in [
            "Traceback (most recent call last):\n  File \"my__kipferl_bundle_runtime.py\", line 1\n    exec(code)\nValueError: x",
            "Traceback (most recent call last):\n  File \"app.py\", line 1\n    raise ValueError(message)\nValueError: first line\n  File \"x.py\", line 1\n    exec(__kipferl_code)",
            "no traceback here\n  File \"a.py\", line 1\n    exec(__kipferl_code)\n",
        ] {
            assert_eq!(hide_bootstrap_frames(traceback), traceback);
        }
    }
}
