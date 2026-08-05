"""
Minimal tarfile module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.
"""

import sys

_passed = 0
_failed = 0
_errors = []
_skipped = 0


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


try:
    import tarfile as tarfile_mod

    HAS_TARFILE = True
except ImportError:
    HAS_TARFILE = False
    print("SKIP: tarfile module not available")

if HAS_TARFILE:
    print("\n=== tarfile.open/is_tarfile ===")
    test("has open", hasattr(tarfile_mod, "open") and callable(tarfile_mod.open))
    test(
        "has is_tarfile",
        hasattr(tarfile_mod, "is_tarfile") and callable(tarfile_mod.is_tarfile),
    )

    def _oct(n):
        if n == 0:
            return "0"
        s = ""
        while n > 0:
            s = chr(ord("0") + (n & 7)) + s
            n //= 8
        return s

    def _sum_bytes(b):
        total = 0
        i = 0
        while i < len(b):
            total += b[i]
            i += 1
        return total

    def _ustar_header(name, size):
        name_b = name.encode()
        if len(name_b) > 100:
            name_b = name_b[:100]
        name_field = name_b + (b"\x00" * (100 - len(name_b)))
        mode_field = b"0000644\x00"
        uid_field = b"0000000\x00"
        gid_field = b"0000000\x00"
        size_o = _oct(size)
        if len(size_o) > 11:
            size_o = size_o[-11:]
        size_field = (("0" * (11 - len(size_o))) + size_o + "\x00").encode()
        mtime_field = b"00000000000\x00"
        chksum_field = b"        "
        typeflag = b"0"
        linkname = b"\x00" * 100
        magic = b"ustar\x00"
        version = b"00"
        uname = b"\x00" * 32
        gname = b"\x00" * 32
        devmajor = b"\x00" * 8
        devminor = b"\x00" * 8
        prefix = b"\x00" * 155
        pad = b"\x00" * 12

        hdr = name_field
        hdr += mode_field
        hdr += uid_field
        hdr += gid_field
        hdr += size_field
        hdr += mtime_field
        hdr += chksum_field
        hdr += typeflag
        hdr += linkname
        hdr += magic
        hdr += version
        hdr += uname
        hdr += gname
        hdr += devmajor
        hdr += devminor
        hdr += prefix
        hdr += pad
        chk = _sum_bytes(hdr)
        chk_o = _oct(chk)
        if len(chk_o) > 6:
            chk_o = chk_o[-6:]
        chk_s = (("0" * (6 - len(chk_o))) + chk_o + "\x00 ").encode()
        hdr = hdr[:148] + chk_s + hdr[156:]
        return hdr

    def _make_tar(files):
        out = b""
        for name, content in files:
            out += _ustar_header(name, len(content)) + content
            pad = (512 - (len(content) % 512)) % 512
            if pad:
                out += b"\x00" * pad
        out += b"\x00" * 1024
        return out

    TAR_BYTES = _make_tar([("a.txt", b"hi"), ("empty.txt", b"")])
    path = "__kipferl_test.tar"
    try:
        with open(path, "wb") as f:
            f.write(TAR_BYTES)
        test("is_tarfile True", tarfile_mod.is_tarfile(path) is True)
        tf = tarfile_mod.open(path, "r")
        names = tf.getnames()
        test("getnames returns list", isinstance(names, list))
        test("contains a.txt", "a.txt" in names)
        fobj = tf.extractfile("a.txt")
        data = fobj.read()
        test("extractfile/read bytes", data == b"hi")
        empty = tf.extractfile("empty.txt").read()
        test("extract empty", empty == b"")
        tf.close()
        try:
            import os

            os.remove(path)
        except Exception:
            pass
    except Exception as e:
        test("tarfile read", False)
        print(f"  ERROR: {e}")

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
