use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parses_and_serializes_supported_configuration_formats() {
    let output = run(concat!(
        "import configparser, csv, io, json, kdl, toml, tomllib, yaml\n",
        "from xml.etree.ElementTree import ElementTree, fromstring, parse, tostring\n",
        "data = {'name': 'kipferl', 'enabled': True, 'ports': [80, 443], 'nested': {'greeting': 'Grüß dich'}}\n",
        "assert json.loads(json.dumps(data)) == data\n",
        "assert yaml.loads(yaml.dumps(data)) == data\n",
        "toml_data = toml.loads(toml.dumps(data))\n",
        "assert toml_data == data and tomllib.loads(toml.dumps(data)) == data\n",
        "rows = list(csv.reader(['name,value', 'kipferl,6']))\n",
        "assert rows == [['name', 'value'], ['kipferl', '6']]\n",
        "config = configparser.ConfigParser()\n",
        "config.read_string('[server]\\nenabled=yes\\nport=443\\n')\n",
        "assert config.getboolean('server', 'enabled') and config.getint('server', 'port') == 443\n",
        "root = fromstring('<config enabled=\"true\"><name>kipferl</name></config>')\n",
        "assert root.attrib['enabled'] == 'true' and list(root)[0].text == 'kipferl'\n",
        "assert 'kipferl' in tostring(root, 'unicode') and ElementTree(root).getroot() is root\n",
        "document = kdl.loads('(config)package (name)kipferl version=(major)6 {\\n  feature yaml enabled=#true\\n}\\n')\n",
        "node = document[0]\n",
        "assert node['name'] == 'package' and node['type'] == 'config'\n",
        "assert node['entries'][0] == {'name': None, 'type': 'name', 'value': 'kipferl'}\n",
        "assert node['entries'][1] == {'name': 'version', 'type': 'major', 'value': 6}\n",
        "assert node['children'][0]['entries'][1]['value'] is True\n",
        "assert kdl.loads(kdl.dumps(document)) == document\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn reads_and_writes_yaml_toml_and_kdl_files() {
    let directory = TemporaryDirectory::new("files");
    let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .current_dir(&directory.path)
        .args([
            "-c",
            concat!(
                "import configparser, json, kdl, toml, yaml\n",
                "from xml.etree.ElementTree import ElementTree, fromstring, parse\n",
                "data = {'name': 'kipferl', 'count': 3}\n",
                "json.dump(data, 'settings.json')\n",
                "assert json.load('settings.json') == data\n",
                "yaml.dump(data, 'settings.yaml')\n",
                "assert yaml.load('settings.yaml') == data\n",
                "toml.dump(data, 'settings.toml')\n",
                "assert toml.load('settings.toml') == data\n",
                "document = [kdl.node('package', [kdl.argument('kipferl'), kdl.property('count', 3)])]\n",
                "kdl.dump(document, 'settings.kdl')\n",
                "assert kdl.load('settings.kdl') == document\n",
                "ElementTree(fromstring('<config><name>kipferl</name></config>')).write('settings.xml')\n",
                "assert parse('settings.xml').getroot().tag == 'config'\n",
                "with open('settings.ini', 'w') as stream:\n",
                "    stream.write('[app]\\nname = kipferl\\n')\n",
                "config = configparser.ConfigParser()\n",
                "assert config.read('settings.ini') == ['settings.ini'] and config.get('app', 'name') == 'kipferl'\n",
                "with open('stream.yaml', 'w') as stream:\n",
                "    yaml.safe_dump(data, stream)\n",
                "with open('stream.yaml', 'r') as stream:\n",
                "    assert yaml.safe_load(stream.read()) == data\n",
            ),
        ])
        .output()
        .expect("run config format file I/O script");
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
    for name in [
        "settings.json",
        "settings.yaml",
        "settings.toml",
        "settings.kdl",
        "settings.xml",
        "settings.ini",
        "stream.yaml",
    ] {
        assert!(directory.path.join(name).is_file(), "missing {name}");
    }
}

#[test]
fn reports_invalid_or_unsupported_configuration_data() {
    let output = run(concat!(
        "import kdl, toml, yaml\n",
        "def must_fail(operation):\n",
        "    try:\n",
        "        operation()\n",
        "    except Exception:\n",
        "        return\n",
        "    raise AssertionError('operation unexpectedly succeeded')\n",
        "must_fail(lambda: yaml.loads('items: [1, 2'))\n",
        "must_fail(lambda: toml.loads('name ='))\n",
        "must_fail(lambda: toml.dumps({'missing': None}))\n",
        "must_fail(lambda: kdl.loads('node {'))\n",
        "must_fail(lambda: kdl.dumps({'name': 'not-a-document'}))\n",
        "must_fail(lambda: kdl.dumps([{'name': 'node', 'entries': [{'value': []}]}]))\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "kipferl-config-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create temporary directory");
        Self { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
