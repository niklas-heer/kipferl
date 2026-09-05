"""Shared CPython oracle for null-prefixed and three-digit regex octal escapes."""
import re

cases = [
    (r"\0", "\x00"),
    (r"\00", "\x00"),
    (r"\000", "\x00"),
    (r"\001", "\x01"),
    (r"\07", "\x07"),
    (r"\077", "?"),
    (r"\08", "\x008"),
    (r"\078", "\x078"),
    (r"\0123", "\n3"),
    (r"\101", "A"),
    (r"\141", "a"),
    (r"\377", "ÿ"),
    (r"\1234", "S4"),
    (r"[\000-\007]", "\x03"),
    (r"[\141-\143]", "b"),
    (r"\\0", "\\0"),
    (r"\\141", "\\141"),
    (r"\\\000", "\\\x00"),
    (r"(?x)\040", " "),
    (r"(?x)\043", "#"),
    (r"(?x)# ignored \777\n" + "\nA", "A"),
    (r"\u3000", "　"),
    (r"\U00003000", "　"),
]
for pattern, text in cases:
    compiled = re.compile(pattern)
    assert compiled.pattern == pattern
    assert compiled.search(text) is not None
print("23 octal, escape, verbose, and Unicode cases passed")

assert re.findall(r"\141", "banana") == ["a", "a", "a"]
assert re.sub(r"\141", "o", "banana") == "bonono"
assert re.split(r"\040", "one two three", 1) == ["one", "two three"]
print("all regex entry points passed")

# Exact Cc and Z expressions from the reviewed uc-micro-py 2.0.0 wheel.
control = r"[\0-\x1F\x7F-\x9F]"
separator = r"[ \xA0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000]"
assert re.search(control, "\n") is not None
assert re.search(control, "A") is None
assert re.search(separator, "　") is not None
assert re.search(separator, "A") is None
print("uc-micro Cc and Z examples passed")

for pattern in [r"\400", r"\777", r"[\400]", r"[\777]", r"\uZZZZ", r"\U00110000"]:
    failed = False
    try:
        re.compile(pattern)
    except Exception:
        failed = True
    assert failed
print("invalid escapes rejected")
