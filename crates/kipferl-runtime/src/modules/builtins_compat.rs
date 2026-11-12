//! Everyday builtins and methods that the embedded interpreter lacks or
//! implements partially, written in Python against the existing primitives.
use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r#"
def pow(base, exp, mod=None):
    if mod is None:
        return base ** exp
    if not (isinstance(base, int) and isinstance(exp, int) and isinstance(mod, int)):
        raise TypeError("pow() 3rd argument not allowed unless all arguments are integers")
    if mod == 0:
        raise ValueError("pow() 3rd argument cannot be 0")
    if exp < 0:
        raise ValueError("pow() negative exponent is not supported with a modulus")
    result = 1 % mod
    base = base % mod
    while exp:
        if exp & 1:
            result = result * base % mod
        exp >>= 1
        base = base * base % mod
    return result


def sum(iterable, start=0):
    total = start
    for item in iterable:
        total = total + item
    return total


_kipferl_missing = object()


def _kipferl_extreme(name, better, args, key, default):
    if len(args) == 1:
        iterator = iter(args[0])
    elif len(args) == 0:
        raise TypeError(name + " expected at least 1 argument, got 0")
    else:
        if default is not _kipferl_missing:
            raise TypeError("Cannot specify a default for " + name + "() with multiple positional arguments")
        iterator = iter(args)
    best = _kipferl_missing
    best_key = None
    for item in iterator:
        item_key = item if key is None else key(item)
        if best is _kipferl_missing or better(item_key, best_key):
            best = item
            best_key = item_key
    if best is _kipferl_missing:
        if default is _kipferl_missing:
            raise ValueError(name + "() iterable argument is empty")
        return default
    return best


def _kipferl_default(name, options):
    default = options.pop("default", _kipferl_missing)
    for option in options:
        raise TypeError("'" + option + "' is an invalid keyword argument for " + name + "()")
    return default


def min(*args, key=None, **options):
    default = _kipferl_default("min", options)
    return _kipferl_extreme("min", lambda a, b: a < b, args, key, default)


def max(*args, key=None, **options):
    default = _kipferl_default("max", options)
    return _kipferl_extreme("max", lambda a, b: a > b, args, key, default)


def zip(*iterables, strict=False):
    iterators = [iter(it) for it in iterables]
    if not iterators:
        return
    while True:
        values = []
        for index in range(len(iterators)):
            try:
                values.append(next(iterators[index]))
            except StopIteration:
                if strict:
                    if index > 0:
                        raise ValueError("zip() argument %d is shorter than argument%s 1%s"
                                         % (index + 1, "s" if index > 1 else "",
                                            "-%d" % index if index > 1 else ""))
                    for later in range(1, len(iterators)):
                        try:
                            next(iterators[later])
                        except StopIteration:
                            continue
                        raise ValueError("zip() argument %d is longer than argument%s 1%s"
                                         % (later + 1, "s" if later > 1 else "",
                                            "-%d" % later if later > 1 else ""))
                return
        yield tuple(values)


_kipferl_round = round


def _kipferl_half_even(value):
    floor = int(value // 1)
    remainder = value - floor
    if remainder > 0.5 or (remainder == 0.5 and floor % 2 == 1):
        floor += 1
    return floor


def round(number, ndigits=None):
    if isinstance(number, bool) or not isinstance(number, (int, float)):
        return _kipferl_round(number) if ndigits is None else _kipferl_round(number, ndigits)
    if isinstance(number, int):
        if ndigits is None or ndigits >= 0:
            return number
        scale = 10 ** -ndigits
        quotient, remainder = divmod(number, scale)
        if remainder * 2 > scale or (remainder * 2 == scale and quotient % 2 == 1):
            quotient += 1
        return quotient * scale
    if number != number or number in (float("inf"), float("-inf")):
        return _kipferl_round(number) if ndigits is None else number
    if ndigits is None:
        return _kipferl_half_even(number)
    if ndigits > 300:
        return number
    if ndigits >= 0:
        # printf rounds the exact binary value correctly, ties to even
        return float(format(number, "." + str(ndigits) + "f"))
    scale = 10.0 ** -ndigits
    return float(_kipferl_half_even(number / scale)) * scale


def _kipferl_str_removeprefix(self, prefix):
    return self[len(prefix):] if prefix and self.startswith(prefix) else self


def _kipferl_str_removesuffix(self, suffix):
    return self[:-len(suffix)] if suffix and self.endswith(suffix) else self


def _kipferl_str_partition(self, sep):
    if not sep:
        raise ValueError("empty separator")
    index = self.find(sep)
    if index < 0:
        return (self, "", "")
    return (self[:index], sep, self[index + len(sep):])


def _kipferl_str_rpartition(self, sep):
    if not sep:
        raise ValueError("empty separator")
    parts = self.rsplit(sep, 1)
    if len(parts) == 1:
        return ("", "", self)
    return (parts[0], sep, parts[1])


def _kipferl_str_capitalize(self):
    return self[:1].upper() + self[1:].lower()


def _kipferl_str_title(self):
    out = []
    previous_cased = False
    for char in self:
        if char.isalpha():
            out.append(char.lower() if previous_cased else char.upper())
            previous_cased = True
        else:
            out.append(char)
            previous_cased = False
    return "".join(out)


def _kipferl_str_swapcase(self):
    return "".join(c.lower() if c.isupper() else c.upper() for c in self)


def _kipferl_str_center(self, width, fillchar=" "):
    padding = width - len(self)
    if padding <= 0:
        return self
    left = padding // 2 + (padding & width & 1)
    return fillchar * left + self + fillchar * (padding - left)


def _kipferl_str_expandtabs(self, tabsize=8):
    out = []
    column = 0
    for char in self:
        if char == "\t":
            if tabsize > 0:
                spaces = tabsize - column % tabsize
                out.append(" " * spaces)
                column += spaces
        elif char in "\n\r":
            out.append(char)
            column = 0
        else:
            out.append(char)
            column += 1
    return "".join(out)


str.removeprefix = _kipferl_str_removeprefix
str.removesuffix = _kipferl_str_removesuffix
str.partition = _kipferl_str_partition
str.rpartition = _kipferl_str_rpartition
str.capitalize = _kipferl_str_capitalize
str.title = _kipferl_str_title
str.swapcase = _kipferl_str_swapcase
str.casefold = str.lower
str.center = _kipferl_str_center
str.expandtabs = _kipferl_str_expandtabs


set.__le__ = lambda self, other: self.issubset(other) if isinstance(other, set) else NotImplemented
set.__lt__ = lambda self, other: (len(self) < len(other) and self.issubset(other)) if isinstance(other, set) else NotImplemented
set.__ge__ = lambda self, other: self.issuperset(other) if isinstance(other, set) else NotImplemented
set.__gt__ = lambda self, other: (len(self) > len(other) and self.issuperset(other)) if isinstance(other, set) else NotImplemented


def _kipferl_set_pop(self):
    for item in self:
        self.discard(item)
        return item
    raise KeyError("pop from an empty set")


def _kipferl_set_difference_update(self, *others):
    for other in others:
        for item in other:
            self.discard(item)


def _kipferl_set_intersection_update(self, *others):
    for other in others:
        keep = set(other)
        for item in list(self):
            if item not in keep:
                self.discard(item)


def _kipferl_set_symmetric_difference_update(self, other):
    for item in set(other):
        if item in self:
            self.discard(item)
        else:
            self.add(item)


set.pop = _kipferl_set_pop
set.difference_update = _kipferl_set_difference_update
set.intersection_update = _kipferl_set_intersection_update
set.symmetric_difference_update = _kipferl_set_symmetric_difference_update


def _kipferl_float_is_integer(self):
    return self == self and self not in (float("inf"), float("-inf")) and self == self // 1


float.is_integer = _kipferl_float_is_integer


def _kipferl_int_to_bytes(self, length=1, byteorder="big", signed=False):
    if byteorder not in ("big", "little"):
        raise ValueError("byteorder must be either 'little' or 'big'")
    if self < 0 and not signed:
        raise OverflowError("can't convert negative int to unsigned")
    bits = 8 * length
    if bits < 64:
        low = -(1 << (bits - 1)) if signed else 0
        high = (1 << (bits - 1)) if signed else (1 << bits)
        if self < low or self >= high:
            raise OverflowError("int too big to convert")
    out = []
    value = self
    for index in range(length):
        out.append(value & 0xFF)
        value >>= 8
    if byteorder == "big":
        out.reverse()
    return bytes(out)


def _kipferl_int_from_bytes(data, byteorder="big", signed=False):
    if byteorder not in ("big", "little"):
        raise ValueError("byteorder must be either 'little' or 'big'")
    data = list(data)
    if byteorder == "little":
        data.reverse()
    value = -1 if signed and data and data[0] & 0x80 else 0
    for byte in data:
        value = (value << 8) | byte
    return value


def _kipferl_int_bit_count(self):
    return bin(self).count("1")


int.to_bytes = _kipferl_int_to_bytes
int.from_bytes = staticmethod(_kipferl_int_from_bytes)
int.bit_count = _kipferl_int_bit_count


def _kipferl_bytes_hex(self, sep=None):
    digits = [format(byte, "02x") for byte in self]
    return "".join(digits) if sep is None else sep.join(digits)


bytes.hex = _kipferl_bytes_hex
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"builtins",
    kind: NativeModuleKind::Extend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, SOURCE),
        "embedded builtins compatibility layer failed"
    );
}
