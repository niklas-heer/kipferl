//! Native dotted-import behavior against real package trees and a `CPython` oracle.
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> io::Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "kipferl-dotted-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let fixture = Self(root);
        for (name, source) in files {
            let path = fixture.0.join(name);
            let parent = path.parent().ok_or_else(|| io::Error::other("no parent"))?;
            fs::create_dir_all(parent)?;
            fs::write(path, source)?;
        }
        Ok(fixture)
    }

    fn run(&self, program: &str, source: &str) -> io::Result<Output> {
        let mut command = Command::new(program);
        if program == "python3" {
            // Instrument the runtime under test, not the system CPython oracle.
            for variable in [
                "LD_PRELOAD",
                "DYLD_INSERT_LIBRARIES",
                "ASAN_OPTIONS",
                "UBSAN_OPTIONS",
                "LSAN_OPTIONS",
            ] {
                command.env_remove(variable);
            }
        }
        command.args(["-c", source]).current_dir(&self.0).output()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn check(condition: bool, message: impl std::fmt::Display) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message.to_string()))
    }
}
fn matches_cpython(files: &[(&str, &str)], source: &str) -> io::Result<()> {
    let actual = Fixture::new(files)?.run(env!("CARGO_BIN_EXE_pocketpy-kipferl"), source)?;
    let expected = Fixture::new(files)?.run("python3", source)?;
    for (name, output) in [("Kipferl", &actual), ("CPython", &expected)] {
        check(
            output.status.success(),
            format!("{name} failed: {output:?}"),
        )?;
    }
    check(
        actual.stdout == expected.stdout && actual.stderr.is_empty(),
        format!("behavior differs: {actual:?}, {expected:?}"),
    )
}

#[test]
fn dotted_binding_parent_order_metadata_and_cache_match_python() -> io::Result<()> {
    matches_cpython(
        &[
            ("events.py", "seen = []\n"),
            (
                "orchard/__init__.py",
                "import events\nevents.seen.append('root')\n",
            ),
            (
                "orchard/fruit/__init__.py",
                "import events\nevents.seen.append('branch')\n",
            ),
            (
                "orchard/fruit/apple.py",
                "import events\nevents.seen.append('leaf')\nvalue = 42\n",
            ),
            (
                "observer.py",
                "import events\nassert events.seen == ['root', 'branch', 'leaf']\n",
            ),
        ],
        r"
import orchard . fruit . apple as alias, observer
assert 'orchard' not in globals()
assert alias.value == 42
import orchard.fruit.apple
import sys, events
assert events.seen == ['root', 'branch', 'leaf']
assert orchard.fruit.apple is alias
assert alias is sys.modules['orchard.fruit.apple']
assert orchard is sys.modules['orchard']
assert orchard.fruit is sys.modules['orchard.fruit']
assert alias.__name__ == 'orchard.fruit.apple'
assert alias.__package__ == 'orchard.fruit'
assert orchard.fruit.__package__ == 'orchard.fruit'
assert isinstance(orchard.__path__, list)
assert not hasattr(alias, '__path__')
assert __import__('orchard.fruit.apple') is orchard
assert __import__('orchard.fruit.apple', None, None, []) is orchard
assert __import__('orchard.fruit.apple', None, None, ['*']) is alias
print('binding order metadata cache')
",
    )
}

#[test]
fn relative_imports_and_reexports_respect_package_attributes() -> io::Result<()> {
    matches_cpython(
        &[
            (
                "orchard/__init__.py",
                "sentinel = 7\nSHADOW = 'attribute'\nfrom .fruit.apple import value as exported\n",
            ),
            (
                "orchard/fruit/__init__.py",
                "from .. import sentinel\nassert sentinel == 7\n",
            ),
            (
                "orchard/fruit/apple.py",
                "from .. import sentinel\nfrom . import pear\nvalue = sentinel + pear.value\n",
            ),
            ("orchard/fruit/pear.py", "value = 2\n"),
            (
                "orchard/SHADOW.py",
                "raise RuntimeError('shadowed child must not execute')\n",
            ),
        ],
        r"
from orchard.fruit import apple
from orchard import exported, SHADOW
import orchard, sys
assert exported == 9 and apple.value == 9 and SHADOW == 'attribute'
assert 'orchard.SHADOW' not in sys.modules
assert orchard.fruit.apple is apple
print('relative imports and reexports')
",
    )
}

#[test]
fn circular_imports_expose_cached_modules_before_parent_attributes() -> io::Result<()> {
    matches_cpython(
        &[
            ("orchard/__init__.py", "pass\n"),
            (
                "orchard/apple.py",
                "import sys\nassert __name__ in sys.modules\nready = 'partial'\nfrom . import pear\nready = 'complete'\n",
            ),
            (
                "orchard/pear.py",
                "import orchard\nassert not hasattr(orchard, 'apple')\nimport orchard.apple as sibling\nassert sibling.ready == 'partial'\nassert not hasattr(orchard, 'apple')\n",
            ),
        ],
        r"
import orchard.apple, orchard.pear
assert orchard.apple.ready == 'complete'
assert orchard.pear.sibling is orchard.apple
print('circular imports preserve partial identity')
",
    )
}

#[test]
fn failed_child_cleanup_preserves_siblings_attributes_and_earlier_bindings() -> io::Result<()> {
    matches_cpython(
        &[
            ("events.py", "attempts = 0\n"),
            ("orchard/__init__.py", "apple = 'original'\n"),
            ("orchard/side.py", "value = 1\n"),
            (
                "orchard/apple.py",
                "import events\nevents.attempts += 1\nfrom . import side\nraise RuntimeError('child failure')\n",
            ),
        ],
        r"
import orchard, events, sys
for iteration in range(2):
    rejected = False
    try:
        import orchard.side as kept, orchard.apple as missing
    except RuntimeError:
        rejected = True
    assert rejected
    assert kept is sys.modules['orchard.side']
    assert 'missing' not in globals()
    assert 'orchard.apple' not in sys.modules
    assert orchard.apple == 'original'
assert events.attempts == 2
print('failure cleanup and retry')
",
    )
}

#[test]
fn escaped_objects_keep_failed_attempt_globals_across_retry_and_collection() -> io::Result<()> {
    matches_cpython(
        &[
            (
                "keeper.py",
                "attempts = 0\nfunctions = []\nclasses = []\nmodules = []\n",
            ),
            ("orchard/__init__.py", "pass\n"),
            (
                "orchard/apple.py",
                "import keeper, sys\nkeeper.attempts += 1\nvalue = keeper.attempts\ndef read():\n    return value\nclass Saved:\n    def read(self):\n        return value\nkeeper.functions.append(read)\nkeeper.classes.append(Saved)\nkeeper.modules.append(sys.modules[__name__])\nif value == 1:\n    raise RuntimeError('first attempt')\n",
            ),
        ],
        r"
import keeper, gc
try:
    import orchard.apple
except RuntimeError:
    pass
gc.collect()
import orchard.apple
gc.collect()
assert keeper.functions[0]() == 1
assert keeper.functions[1]() == 2
assert keeper.classes[0]().read() == 1
assert keeper.classes[1]().read() == 2
assert keeper.modules[0] is not keeper.modules[1]
assert keeper.modules[1] is orchard.apple
assert keeper.modules[0].value == 1
print('failed module globals stay alive and distinct')
",
    )
}

#[test]
fn cache_hits_preserve_replaced_or_deleted_parent_attributes() -> io::Result<()> {
    matches_cpython(
        &[
            ("orchard/__init__.py", "apple = 'before'\n"),
            ("orchard/apple.py", "value = 42\n"),
        ],
        r"
import orchard.apple as leaf
import orchard
assert orchard.apple is leaf
orchard.apple = 'overridden'
import orchard.apple as changed
assert changed == 'overridden'
del orchard.apple
assert not hasattr(orchard, 'apple')
import orchard.apple as recovered
assert recovered is leaf
assert not hasattr(orchard, 'apple')
print('cached imports preserve parent attribute changes')
",
    )
}

#[test]
fn nonpackages_reject_children_and_packages_win_over_same_named_modules() -> io::Result<()> {
    matches_cpython(
        &[
            ("ordinary.py", "value = 1\n"),
            (
                "ordinary/child.py",
                "raise RuntimeError('must not load child of a module')\n",
            ),
            ("orchard.py", "raise RuntimeError('package must win')\n"),
            ("orchard/__init__.py", "value = 42\n"),
            ("orchard/child.py", "value = 7\n"),
        ],
        r"
rejected = False
try:
    import ordinary.child
except ImportError:
    rejected = True
assert rejected
import orchard.child
assert orchard.value == 42 and orchard.child.value == 7
print('package boundaries and precedence')
",
    )
}

#[test]
fn failed_parent_is_retried_even_when_a_side_effect_child_stays_cached() -> io::Result<()> {
    matches_cpython(
        &[
            ("events.py", "attempts = 0\n"),
            (
                "orchard/__init__.py",
                "import events\nevents.attempts += 1\nfrom . import apple\nraise RuntimeError('parent failure')\n",
            ),
            ("orchard/apple.py", "value = 42\n"),
        ],
        r"
import events, sys
for iteration in range(2):
    rejected = False
    try:
        import orchard.apple
    except RuntimeError:
        rejected = True
    assert rejected
    assert 'orchard' not in sys.modules
    assert 'orchard.apple' in sys.modules
assert events.attempts == 2
print('failed parent retries despite cached child')
",
    )
}

#[test]
fn native_dotted_modules_keep_registered_identity() -> io::Result<()> {
    matches_cpython(
        &[],
        r"
import os.path as path, urllib.parse as parse, xml.etree.ElementTree as etree
import os.path, urllib.parse, xml.etree.ElementTree
from urllib import parse as same
import sys
assert path is os.path
assert parse is urllib.parse and same is parse
assert etree is xml.etree.ElementTree
assert sys.modules['urllib.parse'] is parse
assert sys.modules['xml.etree.ElementTree'] is etree
print('native dotted modules preserve identity')
",
    )
}

#[test]
fn malformed_dotted_import_grammar_is_rejected() -> io::Result<()> {
    for source in [
        "import .orchard",
        "import orchard.",
        "import orchard..apple",
        "import orchard,",
        "import orchard.apple as",
    ] {
        for program in [env!("CARGO_BIN_EXE_pocketpy-kipferl"), "python3"] {
            let output = Fixture::new(&[])?.run(program, source)?;
            check(
                output.status.code() == Some(1),
                format!("{program} accepted {source:?}"),
            )?;
            check(
                String::from_utf8_lossy(&output.stdout).contains("SyntaxError")
                    || String::from_utf8_lossy(&output.stderr).contains("SyntaxError"),
                format!("{program} did not report SyntaxError: {output:?}"),
            )?;
        }
    }
    Ok(())
}

#[test]
fn builtin_fromlists_load_children_and_survive_list_mutation() -> io::Result<()> {
    matches_cpython(
        &[
            ("requests_list.py", "names = ['apple', 'pear']\n"),
            ("orchard/__init__.py", "__all__ = ['plum']\n"),
            (
                "orchard/apple.py",
                "import requests_list\nrequests_list.names.clear()\nvalue = 1\n",
            ),
            (
                "orchard/pear.py",
                "raise RuntimeError('fromlist was cleared before this item')\n",
            ),
            ("orchard/plum.py", "value = 2\n"),
        ],
        r"
import requests_list, sys
root = __import__('orchard', None, None, requests_list.names)
assert root.apple.value == 1
assert 'orchard.pear' not in sys.modules
assert not hasattr(root, 'plum')
assert __import__('orchard', None, None, ['*']) is root
assert root.plum.value == 2
assert __import__('orchard', None, None, ['missing']) is root
print('fromlist loading and mutation')
",
    )
}

#[test]
fn star_reexports_load_all_children_and_class_only_escape_survives_gc() -> io::Result<()> {
    matches_cpython(
        &[
            ("keeper.py", "saved = None\n"),
            ("orchard/__init__.py", "__all__ = ['apple']\n"),
            ("orchard/apple.py", "value = 42\n"),
            (
                "broken.py",
                "import keeper\nclass Escaped:\n    pass\nkeeper.saved = Escaped\nraise RuntimeError('expected')\n",
            ),
        ],
        r"
from orchard import *
assert apple.value == 42
import keeper, gc
try:
    import broken
except RuntimeError:
    pass
gc.collect()
assert keeper.saved.__module__ == 'broken'
print('star reexports and retained class metadata')
",
    )
}

#[test]
fn reloading_a_package_keeps_its_initializer_and_cached_identity() -> io::Result<()> {
    matches_cpython(
        &[
            ("events.py", "attempts = 0\n"),
            (
                "orchard.py",
                "raise RuntimeError('package sibling must not be reloaded')\n",
            ),
            (
                "orchard/__init__.py",
                "import events\nevents.attempts += 1\nvalue = events.attempts\n",
            ),
            ("orchard/apple.py", "value = 42\n"),
        ],
        r"
import orchard.apple, importlib, sys
assert orchard.value == 1
assert importlib.reload(orchard) is orchard
assert orchard.value == 2
assert orchard.apple.value == 42
assert sys.modules['orchard'] is orchard
print('package reload keeps initializer and identity')
",
    )
}
