use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
def _rust_date_isoformat(self):
    return str(self)


def _rust_date_weekday(self):
    offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]
    year = self.year
    if self.month < 3:
        year -= 1
    sunday_based = (year + year // 4 - year // 100 + year // 400 + offsets[self.month - 1] + self.day) % 7
    return (sunday_based + 6) % 7


def _rust_timedelta_total_seconds(self):
    return float(self.days * 86400 + self.seconds)


date.isoformat = _rust_date_isoformat
date.weekday = _rust_date_weekday
timedelta.total_seconds = _rust_timedelta_total_seconds
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"datetime",
    kind: NativeModuleKind::ImportAndExtend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded datetime compatibility layer failed"
    );
}
