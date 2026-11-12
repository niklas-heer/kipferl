//! Everyday builtins and methods that the embedded interpreter lacks or
//! implements partially, written in Python against the existing primitives.
use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, execute_module,
    return_value, type_error,
};

const SOURCE: &str = r#"
def pow(base, exp, mod=None):
    if mod is None:
        return base ** exp
    if not (isinstance(base, (int, bool)) and isinstance(exp, (int, bool))
            and isinstance(mod, (int, bool))):
        raise TypeError("pow() 3rd argument not allowed unless all arguments are integers")
    base, exp, mod = int(base), int(exp), int(mod)
    if mod == 0:
        raise ValueError("pow() 3rd argument cannot be 0")
    if exp < 0:
        raise ValueError("pow() negative exponent is not supported with a modulus")
    return _kipferl_powmod(base, exp, mod)


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
        if ndigits <= -19:
            # 10**19 exceeds 64 bits: only a magnitude above half of it rounds away from zero
            if ndigits == -19 and (number > 5 * 10**18 or number < -5 * 10**18):
                raise OverflowError("integer overflow: the result does not fit in a 64-bit int")
            return 0
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


# Augmented assignment mutates mutable containers in place, as in CPython.
def _kipferl_list_iadd(self, other):
    self.extend(other)
    return self


def _kipferl_list_imul(self, count):
    if not isinstance(count, (int, bool)):
        return NotImplemented
    # the native repetition accepts an int but not a bool
    product = self * int(count)
    self.clear()
    self.extend(product)
    return self


def _kipferl_dict_ior(self, other):
    self.update(other)
    return self


def _kipferl_set_inplace(update):
    def method(self, other):
        if not isinstance(other, set):
            return NotImplemented
        update(self, other)
        return self
    return method


list.__iadd__ = _kipferl_list_iadd
list.__imul__ = _kipferl_list_imul
dict.__ior__ = _kipferl_dict_ior
set.__ior__ = _kipferl_set_inplace(set.update)
set.__iand__ = _kipferl_set_inplace(_kipferl_set_intersection_update)
set.__isub__ = _kipferl_set_inplace(_kipferl_set_difference_update)
set.__ixor__ = _kipferl_set_inplace(_kipferl_set_symmetric_difference_update)


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

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_kipferl_powmod",
    callback: powmod,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"builtins",
    kind: NativeModuleKind::Extend,
    functions: FUNCTIONS,
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

/// `pow(base, exponent, modulus)` for a nonzero modulus. The result takes the
/// sign of the modulus, as `%` does. Every product of two residues below
/// 2^63 fits in 128 bits, so the minimum modulus needs no special case.
fn modular_power(base: i64, exponent: u64, modulus: i64) -> Option<i64> {
    let magnitude = i128::from(modulus.unsigned_abs());
    let mut square = i128::from(base).checked_rem_euclid(magnitude)?;
    let mut result = 1_i128.checked_rem(magnitude)?;
    let mut remaining = exponent;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = result.checked_mul(square)?.checked_rem(magnitude)?;
        }
        remaining >>= 1;
        if remaining > 0 {
            square = square.checked_mul(square)?.checked_rem(magnitude)?;
        }
    }
    if modulus < 0 && result != 0 {
        result = result.checked_sub(magnitude)?;
    }
    i64::try_from(result).ok()
}

unsafe extern "C" fn powmod(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(3, 3) {
        return false;
    }
    let integer = |index| arguments.get(index).and_then(Value::integer);
    let (Some(base), Some(exponent), Some(modulus)) = (integer(0), integer(1), integer(2)) else {
        return type_error(c"pow() arguments must be integers");
    };
    let Ok(exponent) = u64::try_from(exponent) else {
        return type_error(c"pow() exponent must not be negative");
    };
    let Some(result) = modular_power(base, exponent, modulus) else {
        return type_error(c"pow() modulus must not be zero");
    };
    let mut roots = RootFrame::new();
    return_value(roots.integer(result))
}

#[cfg(test)]
mod tests {
    use super::modular_power;

    #[test]
    fn modular_power_takes_the_sign_of_the_modulus() {
        assert_eq!(modular_power(2, 3, 5), Some(3));
        assert_eq!(modular_power(-2, 3, 5), Some(2));
        assert_eq!(modular_power(2, 3, -5), Some(-2));
        assert_eq!(modular_power(10, 3, -5), Some(0));
        assert_eq!(modular_power(7, 0, 1), Some(0));
        assert_eq!(modular_power(7, 0, -1), Some(0));
        assert_eq!(modular_power(1, 1, 0), None);
    }

    #[test]
    fn modular_power_handles_the_64_bit_extremes() {
        assert_eq!(
            modular_power(2, 3, i64::MIN),
            Some(-9_223_372_036_854_775_800)
        );
        assert_eq!(modular_power(i64::MIN, 1, i64::MIN), Some(0));
        assert_eq!(modular_power(-1, 1, i64::MIN), Some(-1));
        assert_eq!(modular_power(i64::MAX, 2, i64::MAX), Some(0));
        assert_eq!(
            modular_power(i64::MAX - 1, u64::MAX, i64::MAX),
            Some(i64::MAX - 1)
        );
        assert_eq!(
            modular_power(3, 200, i64::MIN),
            Some(-2_595_481_728_043_143_007)
        );
    }
}
