"""
Simplified pathlib module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_pathlib.py
"""

import sys

# Test tracking
_passed = 0
_failed = 0
_errors = []
_skipped = 0


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print("  PASS: " + name)
    else:
        _failed += 1
        _errors.append(name)
        print("  FAIL: " + name)


def skip(name, reason):
    global _skipped
    _skipped += 1
    print("  SKIP: " + name + " (" + reason + ")")


# Try to import pathlib
try:
    from pathlib import Path
except ImportError:
    print("SKIP: pathlib module not available")
    sys.exit(0)

# Create a test Path instance to check available features
_test_path = Path("/test")

# Feature detection
_has_name = hasattr(_test_path, "name")
_has_parent = hasattr(_test_path, "parent")
_has_suffix = hasattr(_test_path, "suffix")
_has_stem = hasattr(_test_path, "stem")
_has_is_absolute = hasattr(_test_path, "is_absolute")
_has_is_file = hasattr(_test_path, "is_file")
_has_is_dir = hasattr(_test_path, "is_dir")
_has_cwd = hasattr(Path, "cwd")
_has_joinpath = hasattr(_test_path, "joinpath")
_has_with_name = hasattr(_test_path, "with_name")
_has_with_suffix = hasattr(_test_path, "with_suffix")
_has_resolve = hasattr(_test_path, "resolve")
_has_truediv = hasattr(_test_path, "__truediv__")

# Check if Path supports multiple constructor args
_has_multi_arg = False
try:
    _test_multi = Path("foo", "bar")
    _has_multi_arg = True
except TypeError:
    pass

# Check if str(Path) returns the path string (not object repr)
_str_returns_path = False
_test_str = Path("/usr/bin")
_str_result = str(_test_str)
if _str_result == "/usr/bin":
    _str_returns_path = True

# Check if Path has a .path attribute for getting the string
_has_path_attr = hasattr(_test_path, "path")

# Check if __file__ is available
_has_file = False
try:
    _dummy = __file__
    _has_file = True
except NameError:
    pass


def get_path_str(p):
    if _has_path_attr:
        return p.path
    return str(p)


print("")
print("=== Path construction ===")

p = Path("/usr/bin")
if _str_returns_path:
    test("Path from string", str(p) == "/usr/bin")
elif _has_path_attr:
    test("Path from string", p.path == "/usr/bin")
else:
    skip("Path from string", "no way to get path string")

if _has_multi_arg:
    p = Path("foo", "bar", "baz")
    test("Path from multiple args", get_path_str(p) == "foo/bar/baz")
else:
    skip("Path from multiple args", "multi-arg constructor not supported")

if _has_truediv:
    p = Path("/usr") / "bin" / "python"
    test("Path with / operator", get_path_str(p) == "/usr/bin/python")
else:
    skip("Path with / operator", "__truediv__ not supported")

p = Path(".")
if _str_returns_path:
    test("Path current dir", str(p) == ".")
elif _has_path_attr:
    test("Path current dir", p.path == ".")
else:
    skip("Path current dir", "no way to get path string")


print("")
print("=== Path parts ===")

p = Path("/usr/bin/python3")

if _has_name:
    test("Path.name", p.name == "python3")
else:
    skip("Path.name", "name attribute not supported")

if _has_parent:
    test("Path.parent", get_path_str(p.parent) == "/usr/bin")
else:
    skip("Path.parent", "parent attribute not supported")

if _has_suffix:
    p = Path("/home/user/file.txt")
    test("Path.suffix", p.suffix == ".txt")
    p = Path("/home/user/file.tar.gz")
    test("Path.suffix multi-ext", p.suffix == ".gz")
    p = Path("/home/user/file")
    test("Path.suffix none", p.suffix == "")
else:
    skip("Path.suffix", "suffix attribute not supported")
    skip("Path.suffix multi-ext", "suffix attribute not supported")
    skip("Path.suffix none", "suffix attribute not supported")

if _has_stem:
    p = Path("/home/user/file.txt")
    test("Path.stem", p.stem == "file")
    p = Path("/home/user/file.tar.gz")
    test("Path.stem multi-ext", p.stem == "file.tar")
else:
    skip("Path.stem", "stem attribute not supported")
    skip("Path.stem multi-ext", "stem attribute not supported")


print("")
print("=== Path properties ===")

if _has_is_absolute:
    test("is_absolute /usr", Path("/usr").is_absolute())
    test("is_absolute relative", not Path("foo/bar").is_absolute())
    test("is_absolute dot", not Path(".").is_absolute())
else:
    skip("is_absolute /usr", "is_absolute not supported")
    skip("is_absolute relative", "is_absolute not supported")
    skip("is_absolute dot", "is_absolute not supported")


print("")
print("=== Path operations ===")

if _has_joinpath:
    p = Path("/usr")
    joined = p.joinpath("bin", "python")
    test("joinpath", get_path_str(joined) == "/usr/bin/python")
else:
    skip("joinpath", "joinpath method not supported")

if _has_with_name:
    p = Path("/usr/bin/python")
    test("with_name", get_path_str(p.with_name("python3")) == "/usr/bin/python3")
else:
    skip("with_name", "with_name method not supported")

if _has_with_suffix:
    p = Path("/home/user/file.txt")
    test("with_suffix", get_path_str(p.with_suffix(".md")) == "/home/user/file.md")
    p2 = Path("/home/user/file")
    result = get_path_str(p2.with_suffix(".txt"))
    test("with_suffix add", result == "/home/user/file.txt")
else:
    skip("with_suffix", "with_suffix method not supported")
    skip("with_suffix add", "with_suffix method not supported")


print("")
print("=== Filesystem operations ===")

test("exists cwd", Path(".").exists())
test("exists nonexistent", not Path("/nonexistent/path/foo").exists())

if _has_is_file and _has_file:
    test("is_file on file", Path(__file__).is_file())
    test("is_file on dir", not Path(".").is_file())
elif _has_is_file:
    skip("is_file on file", "__file__ not available")
    test("is_file on dir", not Path(".").is_file())
else:
    skip("is_file on file", "is_file method not supported")
    skip("is_file on dir", "is_file method not supported")

if _has_is_dir:
    test("is_dir on dir", Path(".").is_dir())
    if _has_file:
        test("is_dir on file", not Path(__file__).is_dir())
    else:
        skip("is_dir on file", "__file__ not available")
else:
    skip("is_dir on dir", "is_dir method not supported")
    skip("is_dir on file", "is_dir method not supported")

if _has_cwd:
    cwd = Path.cwd()
    test("cwd is Path", isinstance(cwd, Path))
    test("cwd exists", cwd.exists())
    if _has_is_dir:
        test("cwd is_dir", cwd.is_dir())
    else:
        skip("cwd is_dir", "is_dir method not supported")
else:
    skip("cwd is Path", "cwd class method not supported")
    skip("cwd exists", "cwd class method not supported")
    skip("cwd is_dir", "cwd class method not supported")


print("")
print("=== Comparison and hashing ===")

p1 = Path("/usr/bin")
p2 = Path("/usr/bin")
p3 = Path("/usr/local")

if _str_returns_path or _has_path_attr:
    path1 = get_path_str(p1)
    path2 = get_path_str(p2)
    path3 = get_path_str(p3)
    test("equality same path strings", path1 == path2)
    test("inequality different path strings", path1 != path3)
else:
    skip("equality same", "cannot compare paths")
    skip("equality different", "cannot compare paths")

p = Path("/usr/bin")
if _str_returns_path:
    test("str()", str(p) == "/usr/bin")
    test("repr contains path", "/usr/bin" in repr(p))
elif _has_path_attr:
    test("path attribute", p.path == "/usr/bin")
    skip("str()", "str() returns object repr, not path")
    skip("repr contains path", "repr returns object repr")
else:
    skip("str()", "no way to get path string")
    skip("repr contains path", "no way to get path string")


print("")
print("=== Edge cases ===")

if _str_returns_path or _has_path_attr:
    p = Path("")
    path_str = get_path_str(p)
    test("empty path str", path_str == "." or path_str == "")
    p = Path("/")
    test("root str", get_path_str(p) == "/")
    if _has_is_absolute:
        test("root is_absolute", p.is_absolute())
    else:
        skip("root is_absolute", "is_absolute not supported")
    if _has_name:
        test("root name", p.name == "")
    else:
        skip("root name", "name attribute not supported")
    if _has_name:
        p = Path("/usr/bin/")
        test("trailing slash name", p.name == "bin")
    else:
        skip("trailing slash name", "name attribute not supported")
    p = Path("./foo/./bar")
    path_str = get_path_str(p)
    test("dot paths", path_str == "./foo/./bar" or path_str == "foo/bar")
    p = Path("/usr/bin/../lib")
    test("double dot preserved", ".." in get_path_str(p))
else:
    skip("empty path str", "no way to get path string")
    skip("root str", "no way to get path string")
    skip("root name", "no way to get path string")
    skip("root is_absolute", "no way to get path string")
    skip("trailing slash name", "no way to get path string")
    skip("dot paths", "no way to get path string")
    skip("double dot preserved", "no way to get path string")


print("")
print("=== resolve ===")

if _has_resolve:
    p = Path(".")
    resolved = p.resolve()
    test("resolve returns Path", isinstance(resolved, Path))
    if _has_is_absolute:
        test("resolve is absolute", resolved.is_absolute())
    else:
        skip("resolve is absolute", "is_absolute not supported")
else:
    skip("resolve returns Path", "resolve method not supported")
    skip("resolve is absolute", "resolve method not supported")


print("")
print("=" * 50)
total = _passed + _failed + _skipped
s = "Results: " + str(_passed) + " passed, " + str(_failed) + " failed, " + str(_skipped) + " skipped"
print(s)
if _errors:
    print("Failed tests:")
    for e in _errors:
        print("  - " + e)
    sys.exit(1)
else:
    print("All tests passed!")
