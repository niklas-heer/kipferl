use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r#"
class ConfigParser:
    def __init__(self):
        self._sections = {}
        self._order = []

    def read_string(self, text):
        self._sections = {}
        self._order = []
        self._read_text(text)

    def _read_text(self, text):
        current = None
        for raw in text.split('\n'):
            line = raw.strip()
            if not line or line.startswith('#') or line.startswith(';'):
                continue
            if line.startswith('[') and line.endswith(']'):
                current = line[1:-1].strip()
                if current not in self._sections:
                    self._sections[current] = {}
                    self._order.append(current)
                continue
            if '=' not in line or current is None:
                raise ValueError('invalid configuration line')
            index = line.index('=')
            key = line[:index].strip()
            value = line[index + 1:].strip()
            self._sections[current][key] = value

    def read(self, filenames):
        if isinstance(filenames, str):
            filenames = [filenames]
        loaded = []
        for filename in filenames:
            try:
                stream = open(filename, 'r')
                text = stream.read()
                stream.close()
                self._read_text(text)
                loaded.append(filename)
            except OSError:
                pass
        return loaded

    def write(self, stream):
        for section in self._order:
            stream.write('[' + section + ']\n')
            for key, value in self._sections[section].items():
                stream.write(key + ' = ' + value + '\n')
            stream.write('\n')

    def sections(self):
        return list(self._order)

    def get(self, section, option):
        return self._sections[section][option]

    def getint(self, section, option):
        return int(self.get(section, option))

    def getfloat(self, section, option):
        return float(self.get(section, option))

    def getboolean(self, section, option):
        value = self.get(section, option).lower()
        if value in ('1', 'yes', 'true', 'on'):
            return True
        if value in ('0', 'no', 'false', 'off'):
            return False
        raise ValueError('invalid boolean')

    def has_section(self, section):
        return section in self._sections

    def has_option(self, section, option):
        return section in self._sections and option in self._sections[section]
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"configparser",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, SOURCE),
        "embedded configparser module"
    );
}
