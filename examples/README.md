# Examples and executable recipes

Run examples from the repository root after the
[development setup](../docs/development.md). For the interactive tour:

```console
mise run demo
```

`demo.py` presents the native `tui` and `input` APIs. `quick_demo.py` is the short
prompt demonstration used for recording, while `simple_cli.py` shows a small
command dispatcher with greetings and status output. The `test_*.py` files are
focused UI fixtures; prompt and selection examples need a terminal for their
interactive behavior.

## Start from a practical recipe

| Script | Arguments | Result |
| --- | --- | --- |
| [csv_summary.py](recipes/csv_summary.py) | CSV file with `category` and `amount` columns | JSON totals by category |
| [repository_summary.py](recipes/repository_summary.py) | Directory | JSON file counts by extension, excluding common generated directories |
| [generate_report.py](recipes/generate_report.py) | Input JSON object and output Markdown path | A sorted Markdown table |
| [api_client.py](recipes/api_client.py) | Host, optional `--path`, `--port`, and `--https` | JSON response or an error with exit status 1 |

The CSV recipe is a compact example for ordinary one-line records, not a complete
streaming CSV application; it splits physical lines before parsing. The report
recipe writes the output path supplied by the caller. Review and adapt these
small scripts to your data and application requirements.

For example, run the repository summary with the freshly built runtime:

```console
mise run build-runtime
target/release/pocketpy-kipferl examples/recipes/repository_summary.py examples
```

To package it as a standalone application:

```console
mise run build
target/release/kipferl build examples/recipes/repository_summary.py \
  -o target/repository-summary
target/repository-summary examples
```

The raw-runtime command exercises current Rust source. The packaged command uses
the CLI's embedded release runtime assets; see the
[validation boundary](../docs/development.md#know-which-runtime-you-tested).

## Keep the examples and published recipes aligned

```console
mise run recipes
```

This checks that the website's marked recipe snippets match their source files,
then executes all four recipes against the raw runtime and through standalone
packaging, including running the copied binaries after removing their source
files. Fixtures cover successful output and error paths; the API fixture uses a
local HTTP server, so this check does not establish public HTTPS connectivity.
When adding a recipe, add its published snippet and corresponding execution
fixture in `scripts/check_recipes.py` so it receives the same coverage.
