#!/usr/bin/env python3
"""Check published recipe snippets and run every recipe against a local runtime."""
import argparse
from contextlib import contextmanager
import http.server
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[1]
EXECUTED_RECIPES = {"api_client.py", "csv_summary.py", "generate_report.py", "repository_summary.py"}
SNIPPET = re.compile(
    r"\{/\* recipe:([a-z_]+\.py) \*/\}\n```python\n(.*?)```\n\{/\* endrecipe \*/\}",
    re.DOTALL,
)


class RecipeError(Exception):
    pass


def check_snippets(root=ROOT):
    recipes = {path.name: path.read_text() for path in (root / "examples/recipes").glob("*.py")}
    if not recipes:
        raise RecipeError("No executable recipes found")
    seen = set()
    for page in (root / "website/content/docs").rglob("*.mdx"):
        content = page.read_text()
        matches = list(SNIPPET.finditer(content))
        if content.count("{/* recipe:") != len(matches):
            raise RecipeError(f"Malformed recipe marker in {page}")
        for match in matches:
            name, snippet = match.groups()
            if name not in recipes:
                raise RecipeError(f"Unknown recipe {name} in {page}")
            if snippet != recipes[name]:
                raise RecipeError(f"Recipe snippet drift: {name} in {page}")
            seen.add(name)
    missing = recipes.keys() - seen
    if missing:
        raise RecipeError("Recipes missing from docs: " + ", ".join(sorted(missing)))
    return sorted(recipes)


def run_recipe(runtime, name, args, cwd, expected_code=0, root=ROOT):
    prefix = runtime[name] if isinstance(runtime, dict) else [str(runtime), str(root / "examples/recipes" / name)]
    command = [*prefix, *map(str, args)]
    try:
        result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise RecipeError(f"{name}: could not complete: {error}") from error
    if result.returncode != expected_code:
        raise RecipeError(
            f"{name}: expected exit {expected_code}, got {result.returncode}\n"
            f"stdout: {result.stdout}\nstderr: {result.stderr}"
        )
    return result


def expect_json(result, expected, name):
    try:
        actual = json.loads(result.stdout)
    except ValueError as error:
        raise RecipeError(f"{name}: invalid JSON output: {result.stdout!r}") from error
    if actual != expected:
        raise RecipeError(f"{name}: expected {expected!r}, got {actual!r}")


@contextmanager
def local_api():
    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            status = 200 if self.path in ("/items", "/invalid") else 503
            body = b"not JSON" if self.path == "/invalid" else b'{"items": ["one", "two"]}'
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server.server_port
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def check_execution(runtime, root=ROOT):
    recipes = {path.name for path in (root / "examples/recipes").glob("*.py")}
    if recipes != EXECUTED_RECIPES:
        raise RecipeError("Update recipe execution fixtures to cover exactly: " + ", ".join(sorted(recipes)))
    with tempfile.TemporaryDirectory(prefix="kipferl-recipes-") as temporary:
        work = Path(temporary)
        csv_file = work / "sales.csv"
        csv_file.write_text('category,amount\n"Food, drinks",12.5\nTools,20\n"Food, drinks",7.5\n')
        result = run_recipe(runtime, "csv_summary.py", [csv_file], work, root=root)
        expect_json(result, {"Food, drinks": 20.0, "Tools": 20.0}, "csv_summary.py")
        run_recipe(runtime, "csv_summary.py", [work / "missing.csv"], work, expected_code=1, root=root)

        repository = work / "repository"
        (repository / "src").mkdir(parents=True)
        (repository / ".git").mkdir()
        (repository / ".git/config").write_text("ignored")
        (repository / "src/app.py").write_text("print('hello')\n")
        (repository / "README.md").write_text("# Hello\n")
        (repository / "LICENSE").write_text("MIT\n")
        result = run_recipe(runtime, "repository_summary.py", [repository], work, root=root)
        counts = {".py": 1, ".md": 1, "(no extension)": 1}
        expect_json(result, counts, "repository_summary.py")

        input_file = work / "counts.json"
        input_file.write_text(json.dumps({"Python": 3, "A|B\nC": 2}))
        output = work / "report.md"
        run_recipe(runtime, "generate_report.py", [input_file, output], work, root=root)
        expected = "# Summary\n\n| Item | Value |\n| --- | ---: |\n| A\\|B C | 2 |\n| Python | 3 |\n"
        if not output.exists() or output.read_text() != expected:
            raise RecipeError("generate_report.py: Markdown report differs from expected output")

        with local_api() as port:
            args = ["127.0.0.1", "--port", port, "--path", "/items"]
            result = run_recipe(runtime, "api_client.py", args, work, root=root)
            expect_json(result, {"items": ["one", "two"]}, "api_client.py")
            result = run_recipe(runtime, "api_client.py", args[:-1] + ["/failure"], work, expected_code=1, root=root)
            if "HTTP 503" not in result.stderr:
                raise RecipeError("api_client.py: missing HTTP failure diagnostic")
            result = run_recipe(runtime, "api_client.py", args[:-1] + ["/invalid"], work, expected_code=1, root=root)
            if "Could not fetch JSON" not in result.stderr:
                raise RecipeError("api_client.py: missing invalid JSON diagnostic")


def check_cli(cli, names, root=ROOT):
    commands = {name: [str(cli), "run", str(root / "examples/recipes" / name), "--"] for name in names}
    check_execution(commands, root)
    with tempfile.TemporaryDirectory(prefix="kipferl-recipe-binaries-") as output:
        commands = {}
        with tempfile.TemporaryDirectory(prefix="kipferl-recipe-sources-") as source:
            for name in names:
                script = Path(source) / name
                shutil.copyfile(root / "examples/recipes" / name, script)
                binary = Path(output) / Path(name).stem
                try:
                    result = subprocess.run(
                        [str(cli), "build", str(script), "-o", str(binary)],
                        cwd=source, capture_output=True, text=True, timeout=60,
                    )
                except (OSError, subprocess.TimeoutExpired) as error:
                    raise RecipeError(f"Could not package {name}: {error}") from error
                if result.returncode != 0 or not binary.is_file():
                    raise RecipeError(f"Could not package {name}:\n{result.stdout}\n{result.stderr}")
                commands[name] = [str(binary)]
        # The source directory no longer exists; each executable runs elsewhere.
        check_execution(commands, root)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime", type=Path, default=ROOT / "target/release/pocketpy-kipferl")
    parser.add_argument("--docs-only", action="store_true", help="Check snippet drift without running recipes")
    parser.add_argument("--cli", type=Path, help="Also run through the CLI and verify standalone recipes after deleting build sources")
    args = parser.parse_args()
    try:
        names = check_snippets()
        if not args.docs_only:
            check_execution(args.runtime.resolve())
            if args.cli:
                check_cli(args.cli.resolve(), names)
    except RecipeError as error:
        parser.exit(1, f"Recipe check failed: {error}\n")
    print(f"Verified {len(names)} documented recipes" + ("" if args.docs_only else " and runtime behavior (local HTTP only)") + (" including CLI and isolated standalone binaries" if args.cli and not args.docs_only else ""))


if __name__ == "__main__":
    main()
