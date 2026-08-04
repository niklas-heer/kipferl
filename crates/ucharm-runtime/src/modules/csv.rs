use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
import io


class _RustStringIO:
    def __init__(self, initial_value=""):
        self._value = initial_value

    def write(self, value):
        self._value += value
        return len(value)

    def getvalue(self):
        return self._value


io.StringIO = _RustStringIO


QUOTE_MINIMAL = 0
QUOTE_ALL = 1
QUOTE_NONNUMERIC = 2
QUOTE_NONE = 3


def _rust_csv_row(line):
    fields = []
    field = ""
    quoted = False
    index = 0
    while index < len(line):
        char = line[index]
        if quoted:
            if char == '"':
                if index + 1 < len(line) and line[index + 1] == '"':
                    field += '"'
                    index += 2
                    continue
                quoted = False
            else:
                field += char
        else:
            if char == '"' and len(field) == 0:
                quoted = True
            elif char == ',':
                fields.append(field)
                field = ""
            else:
                field += char
        index += 1
    fields.append(field)
    return fields


def reader(csvfile):
    rows = []
    for line in csvfile:
        rows.append(_rust_csv_row(line))
    return rows


def _rust_csv_field(value):
    value = str(value)
    if ',' in value or '"' in value or '\n' in value or '\r' in value:
        return '"' + value.replace('"', '""') + '"'
    return value


class writer:
    def __init__(self, output):
        self._output = output

    def writerow(self, row):
        fields = []
        for value in row:
            fields.append(_rust_csv_field(value))
        return self._output.write(",".join(fields) + "\r\n")


class DictReader:
    def __init__(self, data, fieldnames=None):
        rows = reader(data)
        if fieldnames is None:
            if len(rows) == 0:
                self._rows = []
                return
            fieldnames = rows[0]
            rows = rows[1:]
        self._rows = []
        for row in rows:
            value = {}
            index = 0
            while index < len(fieldnames) and index < len(row):
                value[fieldnames[index]] = row[index]
                index += 1
            self._rows.append(value)

    def __iter__(self):
        return iter(self._rows)


class DictWriter:
    def __init__(self, output, fieldnames):
        self._writer = writer(output)
        self.fieldnames = fieldnames

    def writeheader(self):
        return self._writer.writerow(self.fieldnames)

    def writerow(self, row):
        values = []
        for field in self.fieldnames:
            if field in row:
                values.append(row[field])
            else:
                values.append("")
        return self._writer.writerow(values)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"csv",
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
        "embedded csv compatibility layer failed"
    );
}
