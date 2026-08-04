#!/usr/bin/env python3
"""
Generate Python type stubs (.pyi) from runtime legacy C module source files.

Note: μcharm's production runtime modules are implemented in Rust. This script
only supports the historical C declaration format; if no matching legacy
sources exist, it emits no stubs and the checked-in stubs must be maintained
from the Rust API surface.

Parses:
- Function signatures from mp_arg_t allowed_args[] arrays
- Docstrings from structured comments:
    /// @brief Short description
    /// @param name Description of parameter
    /// @return Description of return value
- Simple functions from MPY_FUNC_* macros
- Signature hints from // module.func(args) -> type comments

Usage:
    python scripts/generate_stubs.py [--output stubs/]
"""

import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


@dataclass
class Argument:
    name: str
    type: str
    required: bool = False
    default: Optional[str] = None
    doc: str = ""


@dataclass
class Function:
    name: str
    args: list[Argument] = field(default_factory=list)
    returns: str = "None"
    doc: str = ""
    brief: str = ""
    is_method: bool = False


@dataclass
class Constant:
    name: str
    type: str
    value: Optional[str] = None
    doc: str = ""


@dataclass
class Module:
    name: str
    doc: str = ""
    functions: list[Function] = field(default_factory=list)
    constants: list[Constant] = field(default_factory=list)


def parse_mp_arg_type(type_str: str) -> tuple[str, bool, Optional[str]]:
    """Parse MP_ARG_* type to Python type, required flag, and default."""
    required = "MP_ARG_REQUIRED" in type_str

    if "MP_ARG_BOOL" in type_str:
        py_type = "bool"
        default = "False" if not required else None
    elif "MP_ARG_INT" in type_str:
        py_type = "int"
        default = "0" if not required else None
    elif "MP_ARG_OBJ" in type_str:
        py_type = "Optional[str]"  # Most OBJ args are strings or None
        default = "None" if not required else None
    else:
        py_type = "Any"
        default = None

    return py_type, required, default


def parse_allowed_args(content: str, func_name: str) -> list[Argument]:
    """Parse mp_arg_t allowed_args[] array for a function."""
    # Find the function and its allowed_args
    pattern = rf"{func_name}\s*\([^)]*\)\s*\{{\s*static\s+const\s+mp_arg_t\s+allowed_args\[\]\s*=\s*\{{([^;]+)\}};"
    match = re.search(pattern, content, re.DOTALL)

    if not match:
        return []

    args_block = match.group(1)
    args = []

    # Parse each argument: { MP_QSTR_name, MP_ARG_TYPE, {.u_xxx = default} }
    arg_pattern = r"\{\s*MP_QSTR_(\w+)\s*,\s*([^,]+)\s*,\s*\{[^}]*\}\s*\}"

    for m in re.finditer(arg_pattern, args_block):
        name = m.group(1)
        type_flags = m.group(2)
        py_type, required, default = parse_mp_arg_type(type_flags)

        # Adjust type based on common patterns
        if name in ("fg", "bg", "color", "border_color"):
            py_type = "Optional[str]"
        elif name in ("text", "content", "message", "msg", "title", "label", "prompt"):
            py_type = "str" if required else "Optional[str]"
        elif name in ("width", "height", "padding", "current", "total", "index"):
            py_type = "int"
        elif name in ("bold", "dim", "italic", "underline", "strikethrough"):
            py_type = "bool"
        elif name == "border":
            py_type = "str"  # Border style name
        elif name == "options":
            py_type = "list[str]"
        elif name == "default":
            py_type = "Optional[Any]"

        args.append(
            Argument(name=name, type=py_type, required=required, default=default)
        )

    return args


def parse_function_comment(
    content: str, func_name: str, module_name: str
) -> tuple[str, str, str]:
    """Extract signature hint, docstring and return type from comment above function.

    Looks for patterns like:
        // ============================================================================
        // module.func(args) -> return_type
        // Brief description of the function.
        // @param name Description of parameter
        // @return Description of return value
        // ============================================================================

    Returns: (brief, doc, returns)
    """
    # Find the comment block before this function
    # Allow any amount of whitespace and additional comment lines between the doc block and function
    func_pattern = rf"((?://[^\n]*\n)+)(?:\s*(?://[^\n]*\n)*\s*)?(?:static\s+)?mp_obj_t\s+{func_name}"
    match = re.search(func_pattern, content)

    brief = ""
    doc = ""
    returns = "None"

    if match:
        comment_block = match.group(1)

        # Extract signature line: // module.func(...) -> type
        sig_match = re.search(
            r"//\s*\w+\.(\w+)\(([^)]*)\)(?:\s*->\s*(\w+))?", comment_block
        )
        if sig_match:
            returns = sig_match.group(3) or "None"

        # Extract brief description (first non-signature, non-separator comment line)
        for line in comment_block.split("\n"):
            line = line.strip()
            if line.startswith("//"):
                text = line[2:].strip()
                # Skip separator lines and signature lines
                if (
                    text.startswith("=")
                    or text.startswith(module_name + ".")
                    or text.startswith("@")
                ):
                    continue
                if text and not text.startswith("Using "):  # Skip implementation notes
                    brief = text
                    break

    return brief, doc, returns


def parse_function_comment_for_macro(
    content: str, module_name: str, func_name: str
) -> tuple[str, str, str]:
    """Extract signature hint, docstring and return type from comment above MPY_FUNC_* macro.

    Looks for patterns like:
        // ============================================================================
        // module.func(arg) -> return_type
        // Brief description of the function.
        // ============================================================================
        MPY_FUNC_1(module, func) {

    Returns: (brief, doc, returns)
    """
    # Find the comment block before this MPY_FUNC_* macro
    func_pattern = (
        rf"((?://[^\n]*\n)+)\s*MPY_FUNC_[01]\(\s*{module_name}\s*,\s*{func_name}\s*\)"
    )
    match = re.search(func_pattern, content)

    brief = ""
    doc = ""
    returns = "None"

    if match:
        comment_block = match.group(1)

        # Extract signature line: // module.func(...) -> type
        sig_match = re.search(
            rf"//\s*{module_name}\.{func_name}\([^)]*\)(?:\s*->\s*(\w+))?",
            comment_block,
        )
        if sig_match:
            returns = sig_match.group(1) or "None"

        # Extract brief description (first non-signature, non-separator comment line)
        for line in comment_block.split("\n"):
            line = line.strip()
            if line.startswith("//"):
                text = line[2:].strip()
                # Skip separator lines and signature lines
                if (
                    text.startswith("=")
                    or text.startswith(module_name + ".")
                    or text.startswith("@")
                ):
                    continue
                if text and not text.startswith("Using "):  # Skip implementation notes
                    brief = text
                    break

    return brief, doc, returns


def parse_simple_func(content: str, macro_match: re.Match) -> Optional[Function]:
    """Parse MPY_FUNC_1 or MPY_FUNC_0 style functions."""
    macro = macro_match.group(0)

    if "MPY_FUNC_0" in macro:
        # No args function
        module = macro_match.group(1)
        name = macro_match.group(2)
        return Function(name=name, args=[], returns="Any")
    elif "MPY_FUNC_1" in macro:
        # Single arg function
        module = macro_match.group(1)
        name = macro_match.group(2)
        return Function(
            name=name,
            args=[Argument(name="arg", type="Any", required=True)],
            returns="Any",
        )

    return None


def parse_module_constants(content: str) -> list[Constant]:
    """Parse MPY_MODULE_INT and other constant definitions."""
    constants = []

    # MPY_MODULE_INT(NAME, value)
    for m in re.finditer(r"MPY_MODULE_INT\(\s*(\w+)\s*,\s*(\d+)\s*\)", content):
        constants.append(Constant(name=m.group(1), type="int", value=m.group(2)))

    # MPY_MODULE_STR(NAME, "value")
    for m in re.finditer(r'MPY_MODULE_STR\(\s*(\w+)\s*,\s*"([^"]*)"\s*\)', content):
        constants.append(Constant(name=m.group(1), type="str", value=f'"{m.group(2)}"'))

    return constants


def parse_c_module(filepath: Path) -> Optional[Module]:
    """Parse a C module file and extract function/constant definitions."""
    content = filepath.read_text()

    # Extract module name from MPY_MODULE_BEGIN(name) or filename
    mod_match = re.search(r"MPY_MODULE_BEGIN\(\s*(\w+)\s*\)", content)
    if not mod_match:
        return None

    module_name = mod_match.group(1)
    module = Module(name=module_name)

    # Extract module docstring from file header comment
    header_match = re.search(r"/\*\s*\n\s*\*\s*(\w+\.c)\s*-\s*([^\n]+)", content)
    if header_match:
        module.doc = header_match.group(2).strip()

    # Parse constants
    module.constants = parse_module_constants(content)

    # Find all function definitions in the module table
    # Look for entries like: { MP_ROM_QSTR(MP_QSTR_func), MP_ROM_PTR(&module_func_obj) }
    # or MPY_MODULE_FUNC(module, func)

    func_entries = re.findall(
        r"MP_ROM_QSTR\(MP_QSTR_(\w+)\)\s*,\s*MP_ROM_PTR\(&(\w+)_obj\)", content
    )
    func_entries += [
        (m.group(2), f"{m.group(1)}_{m.group(2)}")
        for m in re.finditer(r"MPY_MODULE_FUNC\(\s*(\w+)\s*,\s*(\w+)\s*\)", content)
    ]

    for py_name, c_func_base in func_entries:
        func = Function(name=py_name)

        # Try to find the actual function implementation
        c_func_name = f"{module_name}_{py_name}_func"

        # Parse arguments from allowed_args if it's a KW function
        func.args = parse_allowed_args(content, c_func_name)

        # Get brief description and return type hint from comment
        brief, _, returns = parse_function_comment(content, c_func_name, module_name)
        func.brief = brief
        func.returns = returns

        # If no args found, check if it's a simple function
        if not func.args:
            # Check for MPY_FUNC_1 pattern
            simple_match = re.search(
                rf"MPY_FUNC_1\(\s*{module_name}\s*,\s*{py_name}\s*\)", content
            )
            if simple_match:
                func.args = [Argument(name="value", type="Any", required=True)]
                # Also try to get comment from MPY_FUNC_1 declaration
                if not func.brief:
                    brief, _, returns = parse_function_comment_for_macro(
                        content, module_name, py_name
                    )
                    func.brief = brief
                    if returns != "None":
                        func.returns = returns

            simple_match = re.search(
                rf"MPY_FUNC_0\(\s*{module_name}\s*,\s*{py_name}\s*\)", content
            )
            if simple_match:
                func.args = []
                # Also try to get comment from MPY_FUNC_0 declaration
                if not func.brief:
                    brief, _, returns = parse_function_comment_for_macro(
                        content, module_name, py_name
                    )
                    func.brief = brief
                    if returns != "None":
                        func.returns = returns

        module.functions.append(func)

    return module


def generate_stub(module: Module) -> str:
    """Generate .pyi stub content for a module."""
    lines = ['"""']
    lines.append(f"{module.name} - {module.doc or 'Native module'}")
    lines.append('"""')
    lines.append("")
    lines.append(
        "from typing import Optional, Any, List, Callable, TypeVar, Iterable, Iterator"
    )
    lines.append("")

    # Constants
    if module.constants:
        for const in module.constants:
            if const.doc:
                lines.append(f"# {const.doc}")
            lines.append(f"{const.name}: {const.type}")
        lines.append("")

    # Functions
    for func in module.functions:
        # Build argument string
        arg_parts = []
        kwarg_parts = []

        for arg in func.args:
            if arg.required:
                arg_parts.append(f"{arg.name}: {arg.type}")
            else:
                default = arg.default or "None"
                kwarg_parts.append(f"{arg.name}: {arg.type} = {default}")

        # Add * separator for keyword-only args if we have both positional and kwargs
        if arg_parts and kwarg_parts:
            all_parts = arg_parts + ["*"] + kwarg_parts
        else:
            all_parts = arg_parts + kwarg_parts

        args_str = ", ".join(all_parts)
        lines.append(f"def {func.name}({args_str}) -> {func.returns}:")

        # Build docstring
        if func.brief or func.doc or any(arg.doc for arg in func.args):
            lines.append('    """')
            if func.brief:
                lines.append(f"    {func.brief}")
            elif func.doc:
                lines.append(f"    {func.doc}")

            # Add args documentation
            args_with_docs = [arg for arg in func.args if arg.doc]
            if args_with_docs:
                lines.append("")
                lines.append("    Args:")
                for arg in args_with_docs:
                    lines.append(f"        {arg.name}: {arg.doc}")

            lines.append('    """')

        lines.append("    ...")
        lines.append("")

    return "\n".join(lines)


def main():
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    native_dir = project_root / "runtime"
    output_dir = project_root / "stubs"

    # Parse command line args
    if "--output" in sys.argv:
        idx = sys.argv.index("--output")
        if idx + 1 < len(sys.argv):
            output_dir = Path(sys.argv[idx + 1])

    output_dir.mkdir(exist_ok=True)

    # Find all mod*.c files
    modules = []
    for mod_file in native_dir.glob("*/legacy/mod*.c"):
        print(f"Parsing {mod_file.relative_to(project_root)}...")
        module = parse_c_module(mod_file)
        if module:
            modules.append(module)
            print(f"  Found module: {module.name}")
            print(f"    Functions: {[f.name for f in module.functions]}")
            print(f"    Constants: {[c.name for c in module.constants]}")

    # Generate stubs
    print(f"\nGenerating stubs in {output_dir}/")
    for module in modules:
        stub_content = generate_stub(module)
        stub_file = output_dir / f"{module.name}.pyi"
        stub_file.write_text(stub_content)
        print(f"  Generated {stub_file.name}")

    print(f"\nGenerated {len(modules)} stub files.")


if __name__ == "__main__":
    main()
