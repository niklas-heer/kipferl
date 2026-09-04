use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r"
class Namespace:
    def __init__(self, **kwargs):
        for key, value in kwargs.items():
            setattr(self, key, value)

class _MutuallyExclusiveGroup:
    def __init__(self, parser):
        self.parser = parser
        self.dests = []

    def add_argument(self, *names, **kwargs):
        spec = self.parser.add_argument(*names, **kwargs)
        self.dests.append(spec['dest'])
        return spec

class _SubParsersAction:
    def __init__(self, parser, dest):
        self.parser = parser
        self.dest = dest

    def add_parser(self, name, **kwargs):
        child = ArgumentParser(prog=name, **kwargs)
        self.parser._subparsers[name] = child
        return child

class ArgumentParser:
    def __init__(self, prog=None, description=None, **kwargs):
        self.prog = prog
        self.description = description
        self._options = {}
        self._option_order = []
        self._positionals = []
        self._groups = []
        self._subparsers = {}
        self._subparser_dest = None

    def add_argument(self, *names, **kwargs):
        if len(names) == 0:
            raise TypeError('add_argument() requires a name')
        first = names[0]
        is_option = first.startswith('-')
        dest = kwargs.get('dest')
        if dest is None:
            chosen = first
            for name in names:
                if name.startswith('--'):
                    chosen = name
                    break
            dest = chosen.lstrip('-').replace('-', '_')
        spec = {
            'names': names,
            'dest': dest,
            'type': kwargs.get('type'),
            'default': kwargs.get('default'),
            'has_default': 'default' in kwargs,
            'action': kwargs.get('action'),
            'nargs': kwargs.get('nargs'),
            'const': kwargs.get('const'),
            'choices': kwargs.get('choices'),
            'required': kwargs.get('required', False),
            'help': kwargs.get('help'),
        }
        if is_option:
            self._option_order.append(spec)
            for name in names:
                self._options[name] = spec
        else:
            self._positionals.append(spec)
        return spec

    def add_mutually_exclusive_group(self, **kwargs):
        group = _MutuallyExclusiveGroup(self)
        self._groups.append(group.dests)
        return group

    def add_subparsers(self, dest=None, **kwargs):
        self._subparser_dest = dest
        return _SubParsersAction(self, dest)

    def _convert(self, spec, value):
        converter = spec['type']
        result = converter(value) if converter is not None else value
        choices = spec['choices']
        if choices is not None and result not in choices:
            raise SystemExit('invalid choice')
        return result

    def parse_args(self, args=None):
        if args is None:
            import sys
            args = sys.argv[1:]
        args = list(args)
        if len(args) > 0 and args[0] in self._subparsers:
            command = args[0]
            namespace = self._subparsers[command].parse_args(args[1:])
            if self._subparser_dest is not None:
                setattr(namespace, self._subparser_dest, command)
            return namespace

        namespace = Namespace()
        seen = {}
        for spec in self._option_order:
            action = spec['action']
            if spec['has_default']:
                value = spec['default']
            elif action == 'store_true':
                value = False
            elif action == 'store_false':
                value = True
            else:
                value = None
            setattr(namespace, spec['dest'], value)

        positional_values = []
        index = 0
        while index < len(args):
            token = args[index]
            if token.startswith('-'):
                if token not in self._options:
                    raise SystemExit('unknown option')
                spec = self._options[token]
                dest = spec['dest']
                for group in self._groups:
                    if dest in group:
                        for other in group:
                            if other != dest and other in seen:
                                raise SystemExit('mutually exclusive options')
                action = spec['action']
                nargs = spec['nargs']
                if action == 'store_true':
                    value = True
                    index += 1
                elif action == 'store_false':
                    value = False
                    index += 1
                elif nargs == '*':
                    value = []
                    index += 1
                    while index < len(args) and not args[index].startswith('-'):
                        value.append(self._convert(spec, args[index]))
                        index += 1
                elif nargs == '?':
                    index += 1
                    if index >= len(args) or args[index].startswith('-'):
                        value = spec['const']
                    else:
                        value = self._convert(spec, args[index])
                        index += 1
                else:
                    if index + 1 >= len(args):
                        raise SystemExit('expected one argument')
                    value = self._convert(spec, args[index + 1])
                    index += 2
                setattr(namespace, dest, value)
                seen[dest] = True
            else:
                positional_values.append(token)
                index += 1

        position = 0
        for spec in self._positionals:
            nargs = spec['nargs']
            if nargs == '*' or nargs == '+':
                values = []
                while position < len(positional_values):
                    values.append(self._convert(spec, positional_values[position]))
                    position += 1
                if nargs == '+' and len(values) == 0:
                    raise SystemExit('required positional missing')
                setattr(namespace, spec['dest'], values)
                seen[spec['dest']] = True
            else:
                if position >= len(positional_values):
                    raise SystemExit('required positional missing')
                setattr(namespace, spec['dest'], self._convert(spec, positional_values[position]))
                seen[spec['dest']] = True
                position += 1
        if position != len(positional_values):
            raise SystemExit('unrecognized arguments')

        for spec in self._option_order:
            if spec['required'] and spec['dest'] not in seen:
                raise SystemExit('required option missing')
        return namespace
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"argparse",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded argparse module");
}
