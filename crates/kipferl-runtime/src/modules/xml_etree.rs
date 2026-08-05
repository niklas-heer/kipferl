use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r#"
class Element:
    def __init__(self, tag, attrib=None, **extra):
        self.tag = tag
        self.attrib = {} if attrib is None else attrib
        for key, value in extra.items():
            self.attrib[key] = value
        self.text = None
        self._children = []

    def append(self, element):
        self._children.append(element)

    def __iter__(self):
        return iter(self._children)

def SubElement(parent, tag, attrib=None, **extra):
    child = Element(tag, attrib, **extra)
    parent.append(child)
    return child

def _escape(value):
    return value.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')

def _unescape(value):
    return value.replace('&lt;', '<').replace('&gt;', '>').replace('&quot;', '"').replace('&apos;', "'").replace('&amp;', '&')

def _serialize(element):
    attributes = ''
    for key, value in element.attrib.items():
        attributes += ' ' + key + '=\"' + _escape(str(value)) + '\"'
    body = '' if element.text is None else _escape(element.text)
    for child in element._children:
        body += _serialize(child)
    return '<' + element.tag + attributes + '>' + body + '</' + element.tag + '>'

def tostring(element, encoding='us-ascii', method='xml'):
    value = _serialize(element)
    return value

def _attributes(header):
    parts = header.split()
    attributes = {}
    for part in parts[1:]:
        if '=' in part:
            index = part.index('=')
            key = part[:index]
            value = part[index + 1:]
            if len(value) >= 2 and value[0] in "'\"" and value[-1] == value[0]:
                value = value[1:-1]
            attributes[key] = _unescape(value)
    return parts[0], attributes

def _parse_at(source, position):
    while position < len(source) and source[position] in ' \t\r\n':
        position += 1
    if position >= len(source) or source[position] != '<':
        raise ValueError('expected element')
    close = source.index('>', position)
    header = source[position + 1:close].strip()
    self_closing = header.endswith('/')
    if self_closing:
        header = header[:-1].strip()
    tag, attributes = _attributes(header)
    element = Element(tag, attributes)
    position = close + 1
    if self_closing:
        return element, position
    text = ''
    while position < len(source):
        if source[position:].startswith('</' + tag):
            end = source.index('>', position)
            if text:
                element.text = _unescape(text)
            return element, end + 1
        if source[position] == '<':
            child, position = _parse_at(source, position)
            element.append(child)
        else:
            start = position
            while position < len(source) and source[position] != '<':
                position += 1
            text += source[start:position]
    raise ValueError('unclosed element')

def fromstring(text, parser=None):
    if not isinstance(text, str):
        text = text.decode()
    element, position = _parse_at(text, 0)
    return element

XML = fromstring

class ElementTree:
    def __init__(self, element=None, file=None):
        if file is not None:
            element = parse(file).getroot()
        self._root = element

    def getroot(self):
        return self._root

    def write(self, file, encoding='us-ascii', xml_declaration=None, default_namespace=None, method='xml'):
        value = tostring(self._root, encoding, method)
        if hasattr(file, 'write'):
            file.write(value)
            return None
        stream = open(file, 'w')
        stream.write(value)
        stream.close()
        return None

def parse(source, parser=None):
    if hasattr(source, 'read'):
        text = source.read()
    else:
        stream = open(source, 'r')
        text = stream.read()
        stream.close()
    return ElementTree(fromstring(text, parser))
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"xml.etree.ElementTree",
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
        "embedded ElementTree module"
    );
}
