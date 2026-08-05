use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
import random


class UUID:
    def __init__(self, hex):
        if not isinstance(hex, str):
            raise TypeError("UUID() argument must be a string")
        compact = hex.replace("-", "")
        if len(compact) != 32:
            raise ValueError("invalid UUID string length")
        values = []
        index = 0
        while index < 32:
            try:
                values.append(int(compact[index:index + 2], 16))
            except Exception:
                raise ValueError("invalid UUID string")
            index += 2
        self._hex = compact.lower()
        self._bytes = bytes(values)

    def __str__(self):
        value = self._hex
        return value[:8] + "-" + value[8:12] + "-" + value[12:16] + "-" + value[16:20] + "-" + value[20:]

    def __repr__(self):
        return "UUID('" + str(self) + "')"

    def __eq__(self, other):
        return isinstance(other, UUID) and self._hex == other._hex

    def __ne__(self, other):
        return not self == other

    @property
    def version(self):
        return int(self._hex[12], 16)

    @property
    def hex(self):
        return self._hex

    @property
    def bytes(self):
        return self._bytes

    @property
    def int(self):
        return int(self._hex[:15], 16)


def uuid4():
    values = []
    for _ in range(16):
        values.append(random.getrandbits(8))
    values[6] = (values[6] & 15) | 64
    values[8] = (values[8] & 63) | 128
    digits = "0123456789abcdef"
    encoded = ""
    for value in values:
        encoded += digits[value >> 4] + digits[value & 15]
    return UUID(encoded)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"uuid",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded uuid compatibility layer failed"
    );
}
