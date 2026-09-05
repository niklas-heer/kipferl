use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "kipferl-dotted-imports-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path.canonicalize()?))
    }

    fn write(&self, name: &str, source: &str) -> io::Result<()> {
        let path = self.0.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, source)
    }

    fn cli(&self, args: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_kipferl"))
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("cache"))
            .args(args)
            .output()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dotted_imports_preserve_bindings_initialization_and_failure_in_run_and_standalone()
-> io::Result<()> {
    let fixture = Fixture::new()?;
    fixture.write("src/events.py", "seen = []\n")?;
    fixture.write(
        "src/package/__init__.py",
        "import events\nevents.seen.append('parent')\n__all__ = ['unrequested']\n",
    )?;
    fixture.write(
        "src/package/unrequested.py",
        "raise RuntimeError('ordinary imports must not load __all__ siblings')\n",
    )?;
    fixture.write(
        "src/package/nested/__init__.py",
        "import events\nevents.seen.append('nested')\nSHADOW = 'parent attribute'\n",
    )?;
    fixture.write(
        "src/package/nested/SHADOW.py",
        "raise RuntimeError('from-import must preserve existing parent attributes')\n",
    )?;
    fixture.write(
        "src/package/nested/leaf.py",
        "import events\nevents.seen.append('leaf')\nVALUE = 42\n",
    )?;
    fixture.write(
        "src/observer.py",
        "import sys\nassert sys.modules['__main__'].leaf.VALUE == 42\n",
    )?;
    fixture.write("src/broken/__init__.py", "READY = True\n")?;
    fixture.write(
        "src/broken/child.py",
        "import events\nevents.seen.append('failure')\nraise RuntimeError('child failed')\n",
    )?;
    fixture.write(
        "src/app.py",
        r"import events, sys
if False:
    import package.unrequested
import package.nested.leaf as leaf, observer as observed
assert leaf.VALUE == 42
assert events.seen == ['parent', 'nested', 'leaf']
assert 'package' not in globals()
assert 'package.unrequested' not in sys.modules
import package.nested.leaf
assert package.nested.leaf is leaf
from package.nested import leaf as member
assert member is leaf
from package.nested import SHADOW
assert SHADOW == 'parent attribute'
assert 'package.nested.SHADOW' not in sys.modules
assert __import__('package.nested.leaf') is package
assert __import__('package.nested.leaf', None, None, ['VALUE']) is leaf
assert sys.modules['package.nested.leaf'] is leaf
package.nested.leaf = 'shadowed child'
del package
import package.nested.leaf as shadow
assert shadow == 'shadowed child'
assert 'package' not in globals()
del sys.modules['package.nested'].leaf
import package.nested.leaf as restored
assert restored is leaf
assert 'package' not in globals()
import http.client as client
import http
http.client = 'native shadow'
import http.client as shadow_client
assert shadow_client == 'native shadow'
del http.client
import http.client as cached_client
assert cached_client is client
assert events.seen == ['parent', 'nested', 'leaf']
for attempt in range(2):
    try:
        import broken.child
        assert False
    except RuntimeError as error:
        assert str(error) == 'child failed'
    assert 'broken' not in globals()
    assert 'broken' in sys.modules
    assert 'broken.child' not in sys.modules
    assert not hasattr(sys.modules['broken'], 'child')
assert events.seen == ['parent', 'nested', 'leaf', 'failure', 'failure']
print('dotted imports verified')
",
    )?;
    let run = fixture.cli(&["run", "src/app.py"])?;
    success(&run);
    assert_output(&run);
    success(&fixture.cli(&["build", "src/app.py", "-o", "program"])?);
    fs::remove_dir_all(fixture.0.join("src"))?;
    let standalone = Command::new(fixture.0.join("program"))
        .current_dir(&fixture.0)
        .env("KIPFERL_CACHE_DIR", fixture.0.join("standalone-cache"))
        .output()?;
    success(&standalone);
    assert_output(&standalone);
    Ok(())
}

fn assert_output(output: &Output) {
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "dotted imports verified\n"
    );
}
