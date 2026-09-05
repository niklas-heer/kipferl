# 100 packages worth supporting in Kipferl

Prepared 2026-09-05 against Kipferl 0.7.2's pinned 1,000-project audit. **This is an implementation roadmap, not a list of 100 supported packages.**

We selected packages around the tools people can actually ship: an API client, an interactive operator console, a config editor, a spreadsheet exporter, a file processor or a focused service integration. A popular stub package or Python build backend does not earn a place just because it compiles.

## How to read the ranking

An editorial shortlist of exactly 100 distinct distributions from the pinned 1,000-project audit, chosen for concrete standalone CLI/app workflows. It is not the 100 most downloaded projects, an exhaustive optimum, or a promise of support.

`score = 4 × demand + 6 × usefulness + 3 × reach − 5 × effort`

Usefulness has the largest positive weight because this is a product roadmap. Effort subtracts from the score; larger is harder. A dependency can rank highly when it enables several of the chosen workflows. Ties prefer lower effort, then the original popularity rank, then the name.

| Dimension | Scale |
| --- | --- |
| Demand | 5: PyPI rank 1–50; 4: 51–150; 3: 151–350; 2: 351–650; 1: 651–1000. |
| Usefulness | 5: core app capability; 4: reusable across several workflows; 3: narrow concrete workflow; 2: peripheral; 1: ecosystem plumbing. |
| Reach | Direct dependent count: 5 for ≥50; 4 for 20–49; 3 for 5–19; 2 for 1–4; 1 for zero. |
| Effort | 1: narrow resource check; 2: focused pure API; 3: several runtime/dependency gaps; 4: substantial integration; 5: native/security/large-framework work. |

Usefulness, effort, candidate selection and weights are explicit editorial estimates. Effort scores are relative risk, not days or a guarantee that the first syntax patch will be sufficient. Demand is historical downloads, not observed Kipferl user demand.

Count distinct audited distributions whose recorded Requires-Dist mentions this name. Includes conditional/extra requirements without evaluating markers or version ranges; this is potential direct reach, not a solved dependency closure or a count of newly working packages.

All 100 exact-version project purposes were checked against maintainer-provided PyPI metadata. The linked official documentation supplies additional API context. The scores, scope choices and acceptance tests are our proposed plan, not claims made by those projects.

## Turn prerequisites into shipped capabilities

These are proposed work sequences, not additional compatibility verdicts. A small supported library earns its place by contributing to one of these complete workflows.

- **Ship a useful local data tool:** python-dotenv, platformdirs, pathspec, jmespath, tabulate. Read configuration, select files, query JSON and print a report from a detached executable. Establish scoped import and behavior results for each exact dependency first.
- **Make the shared HTTP stack usable:** certifi, idna, urllib3, requests, httpx. A bundled client talks to a controlled HTTP/TLS fixture with timeouts and certificate failures. Shared prerequisites matter more than importing another SDK.
- **Give the app a dependable command interface:** click, rich, prompt-toolkit, typer. Complete CLI help, errors and exit codes before expanding into rendering, interactive input and typed command frameworks. Test terminal restoration and redirected output.
- **Deliver files people can use elsewhere:** jinja2, xlsxwriter, openpyxl, pypdf. Generate a templated report, a spreadsheet and a bounded PDF transformation, validating each output with an independent reader.

All native implementation work must follow the repository's Rust policy. A native-only wheel is not made installable by this roadmap; any compatible API or packaging strategy needs a separate, explicit design and exact-artifact evidence.

## Ranked shortlist

Each test below is **planned**. The compile audit recorded only the first blocker; fixing it can expose another one. Even compilation-complete entries require import, dependency and behavior evidence before they receive a usable badge. The separate compatibility guide owns verified outcomes.

| Priority | Package / pinned version | PyPI rank | Score (D/U/R/E) | User workflow |
| ---: | --- | ---: | --- | --- |
| 1 | [certifi 2026.7.22](https://pypi.org/project/certifi/2026.7.22/) | 4 | 52 (5/5/4/2) | Bundle the certificate roots expected by Python HTTP clients. |
| 2 | [python-dotenv 1.2.3](https://pypi.org/project/python-dotenv/1.2.3/) | 37 | 49 (5/5/3/2) | Load a local application's configuration from a .env file. |
| 3 | [platformdirs 4.11.7](https://pypi.org/project/platformdirs/4.11.7/) | 38 | 49 (5/5/3/2) | Save settings and caches in the expected user directories. |
| 4 | [pathspec 1.1.1](https://pypi.org/project/pathspec/1.1.1/) | 41 | 49 (5/5/3/2) | Respect .gitignore-style filters in backup and search tools. |
| 5 | [python-dateutil 2.9.0.post0](https://pypi.org/project/python-dateutil/2.9.0.post0/) | 16 | 47 (5/5/4/3) | Interpret user-supplied dates and calendar offsets in reports. |
| 6 | [click 8.5.0](https://pypi.org/project/click/8.5.0/) | 20 | 47 (5/5/4/3) | Build a deploy CLI with subcommands, flags and useful help. |
| 7 | [jmespath 1.1.0](https://pypi.org/project/jmespath/1.1.0/) | 51 | 45 (4/5/3/2) | Filter JSON API responses into useful command output. |
| 8 | [cachetools 7.1.8](https://pypi.org/project/cachetools/7.1.8/) | 133 | 45 (4/5/3/2) | Avoid repeated expensive API lookups in a long-running command. |
| 9 | [requests 2.34.2](https://pypi.org/project/requests/2.34.2/) | 7 | 45 (5/5/5/4) | Build a familiar REST API client or authenticated download tool. |
| 10 | [httpx 0.28.1](https://pypi.org/project/httpx/0.28.1/) | 33 | 45 (5/5/5/4) | Build a modern HTTP client with bounded timeouts and reusable sessions. |
| 11 | [tzdata 2026.3](https://pypi.org/project/tzdata/2026.3/) | 84 | 44 (4/4/3/1) | Ship a versioned timezone database with an offline app. |
| 12 | [filelock 3.32.5](https://pypi.org/project/filelock/3.32.5/) | 43 | 44 (5/5/3/3) | Prevent two copies of a CLI from corrupting the same cache. |
| 13 | [tqdm 4.70.0](https://pypi.org/project/tqdm/4.70.0/) | 50 | 44 (5/5/3/3) | Show progress during a long file conversion or download. |
| 14 | [idna 3.19](https://pypi.org/project/idna/3.19/) | 5 | 43 (5/4/3/2) | Handle internationalized hostnames in API and URL tools. |
| 15 | [jsonschema 4.26.0](https://pypi.org/project/jsonschema/4.26.0/) | 54 | 43 (4/5/4/3) | Give clear validation errors for JSON config and API payloads. |
| 16 | [urllib3 2.7.0](https://pypi.org/project/urllib3/2.7.0/) | 6 | 42 (5/5/4/4) | Provide connection pooling and bounded retries beneath API clients. |
| 17 | [jinja2 3.1.6](https://pypi.org/project/jinja2/3.1.6/) | 40 | 42 (5/5/4/4) | Generate config files and reports from templates and data. |
| 18 | [tabulate 0.10.0](https://pypi.org/project/tabulate/0.10.0/) | 182 | 41 (3/5/3/2) | Print API, CSV and database results as aligned tables. |
| 19 | [tomlkit 0.15.1](https://pypi.org/project/tomlkit/0.15.1/) | 62 | 40 (4/5/3/3) | Edit a user's TOML config while preserving comments. |
| 20 | [tenacity 9.1.4](https://pypi.org/project/tenacity/9.1.4/) | 92 | 40 (4/5/3/3) | Retry temporary service failures without retrying forever. |
| 21 | [beautifulsoup4 4.15.0](https://pypi.org/project/beautifulsoup4/4.15.0/) | 101 | 40 (4/5/3/3) | Extract links or records from saved HTML and web responses. |
| 22 | [pyyaml 6.0.3](https://pypi.org/project/pyyaml/6.0.3/) | 14 | 40 (5/5/5/5) | Read the YAML configuration used by deployment and operations tools. |
| 23 | [pydantic 2.13.5](https://pypi.org/project/pydantic/2.13.5/) | 18 | 40 (5/5/5/5) | Validate typed API and configuration models with useful error messages. |
| 24 | [sortedcontainers 2.4.0](https://pypi.org/project/sortedcontainers/2.4.0/) | 141 | 39 (4/4/3/2) | Maintain ordered queues and indexes in a data-processing tool. |
| 25 | [packaging 26.3](https://pypi.org/project/packaging/26.3/) | 2 | 38 (5/3/5/3) | Interpret application version constraints in an updater or plugin selector. |
| 26 | [charset-normalizer 3.5.1](https://pypi.org/project/charset-normalizer/3.5.1/) | 8 | 38 (5/4/3/3) | Read imported text or HTTP bodies whose encoding is unknown. |
| 27 | [attrs 26.1.0](https://pypi.org/project/attrs/26.1.0/) | 29 | 38 (5/4/3/3) | Define structured application records without repetitive class code. |
| 28 | [rich 15.0.0](https://pypi.org/project/rich/15.0.0/) | 55 | 38 (4/5/4/4) | Make inventory and diagnostic output readable in a terminal. |
| 29 | [boto3 1.43.89](https://pypi.org/project/boto3/1.43.89/) | 1 | 37 (5/5/4/5) | Build an S3 backup, inventory or artifact-download command. |
| 30 | [colorama 0.4.6](https://pypi.org/project/colorama/0.4.6/) | 107 | 36 (4/3/4/2) | Keep colored CLI output compatible with existing Python scripts. |
| 31 | [wcwidth 0.8.3](https://pypi.org/project/wcwidth/0.8.3/) | 132 | 36 (4/4/2/2) | Keep Unicode table columns and terminal cursors aligned. |
| 32 | [isodate 0.7.2](https://pypi.org/project/isodate/0.7.2/) | 171 | 35 (3/4/3/2) | Read ISO 8601 dates and durations from API exports. |
| 33 | [tomli-w 1.2.0](https://pypi.org/project/tomli-w/1.2.0/) | 350 | 35 (3/4/3/2) | Export a generated application configuration as TOML. |
| 34 | [typer 0.27.2](https://pypi.org/project/typer/0.27.2/) | 116 | 35 (4/5/3/4) | Turn typed command functions into a friendly operator CLI. |
| 35 | [openpyxl 3.1.5](https://pypi.org/project/openpyxl/3.1.5/) | 120 | 35 (4/5/3/4) | Read and update business spreadsheets from a standalone CLI. |
| 36 | [more-itertools 11.1.0](https://pypi.org/project/more-itertools/11.1.0/) | 131 | 34 (4/4/3/3) | Process large iterables in bounded chunks without hand-written loops. |
| 37 | [requests-toolbelt 1.0.0](https://pypi.org/project/requests-toolbelt/1.0.0/) | 142 | 34 (4/4/3/3) | Upload large files or inspect HTTP exchanges from an API CLI. |
| 38 | [xlsxwriter 3.2.9](https://pypi.org/project/xlsxwriter/3.2.9/) | 322 | 33 (3/5/2/3) | Generate spreadsheet reports for people who do not use Python. |
| 39 | [fsspec 2026.7.0](https://pypi.org/project/fsspec/2026.7.0/) | 32 | 33 (5/4/3/4) | Use one file interface for local and remote input workflows. |
| 40 | [psutil 7.2.2](https://pypi.org/project/psutil/7.2.2/) | 112 | 33 (4/5/4/5) | Build a portable process and resource diagnostic executable. |
| 41 | [uritemplate 4.2.0](https://pypi.org/project/uritemplate/4.2.0/) | 230 | 32 (3/4/2/2) | Construct API endpoint URLs from templates without ad hoc escaping. |
| 42 | [jsonpointer 3.1.1](https://pypi.org/project/jsonpointer/3.1.1/) | 233 | 32 (3/4/2/2) | Address a single field in JSON config or API responses. |
| 43 | [jsonpatch 1.33](https://pypi.org/project/jsonpatch/1.33/) | 265 | 32 (3/4/2/2) | Apply or generate reviewable changes to JSON configuration. |
| 44 | [websockets 17.1](https://pypi.org/project/websockets/17.1/) | 67 | 32 (4/4/4/4) | Add a live event stream to a terminal application. |
| 45 | [markdown-it-py 4.2.0](https://pypi.org/project/markdown-it-py/4.2.0/) | 56 | 31 (4/4/2/3) | Render CommonMark content for previews and terminal documentation. |
| 46 | [soupsieve 2.9.2](https://pypi.org/project/soupsieve/2.9.2/) | 100 | 31 (4/4/2/3) | Select HTML elements with CSS selectors in a scraping CLI. |
| 47 | [prompt-toolkit 3.0.53](https://pypi.org/project/prompt-toolkit/3.0.53/) | 159 | 31 (3/5/3/4) | Add history and completion to an interactive database or API console. |
| 48 | [ruamel-yaml 0.19.1](https://pypi.org/project/ruamel-yaml/0.19.1/) | 186 | 31 (3/5/3/4) | Update deployment YAML without deleting the user's comments. |
| 49 | [defusedxml 0.7.1](https://pypi.org/project/defusedxml/0.7.1/) | 160 | 30 (3/4/3/3) | Read external XML using parsers designed to reject dangerous XML features. |
| 50 | [structlog 26.1.0](https://pypi.org/project/structlog/26.1.0/) | 306 | 30 (3/5/1/3) | Emit structured machine-readable logs from an operational tool. |
| 51 | [openai 3.8.0](https://pypi.org/project/openai/3.8.0/) | 98 | 30 (4/5/3/5) | Ship an AI-assisted command that calls an external model API. |
| 52 | [dnspython 2.8.0](https://pypi.org/project/dnspython/2.8.0/) | 130 | 29 (4/4/3/4) | Build a bounded DNS troubleshooting or service-discovery command. |
| 53 | [semver 3.0.4](https://pypi.org/project/semver/3.0.4/) | 368 | 28 (2/4/2/2) | Compare application versions and decide whether an update is newer. |
| 54 | [python-slugify 8.0.4](https://pypi.org/project/python-slugify/8.0.4/) | 377 | 28 (2/4/2/2) | Generate stable readable filenames or URL slugs from titles. |
| 55 | [prettytable 3.18.0](https://pypi.org/project/prettytable/3.18.0/) | 512 | 28 (2/4/2/2) | Produce sortable terminal tables for an inventory tool. |
| 56 | [pypdf 6.17.0](https://pypi.org/project/pypdf/6.17.0/) | 258 | 28 (3/5/2/4) | Merge, split or inspect PDFs without a desktop application. |
| 57 | [markupsafe 3.0.3](https://pypi.org/project/markupsafe/3.0.3/) | 34 | 28 (5/4/3/5) | Preserve correct HTML escaping in generated reports. |
| 58 | [pyjwt 2.13.0](https://pypi.org/project/pyjwt/2.13.0/) | 45 | 28 (5/4/3/5) | Verify signed tokens in an API diagnostic or local authentication workflow. |
| 59 | [email-validator 2.3.0](https://pypi.org/project/email-validator/2.3.0/) | 168 | 27 (3/4/2/3) | Validate and normalize email addresses in an import wizard. |
| 60 | [markdown 3.10.3](https://pypi.org/project/markdown/3.10.3/) | 290 | 27 (3/4/2/3) | Convert local Markdown notes into HTML reports. |
| 61 | [xmltodict 1.0.4](https://pypi.org/project/xmltodict/1.0.4/) | 292 | 27 (3/4/2/3) | Turn XML exports into JSON-like records for import pipelines. |
| 62 | [marshmallow 4.3.1](https://pypi.org/project/marshmallow/4.3.1/) | 299 | 27 (3/4/2/3) | Validate and serialize records in a data-import CLI. |
| 63 | [python-json-logger 4.2.0](https://pypi.org/project/python-json-logger/4.2.0/) | 302 | 27 (3/4/2/3) | Integrate a CLI's logs with JSON log ingestion. |
| 64 | [textual 8.2.8](https://pypi.org/project/textual/8.2.8/) | 97 | 27 (4/5/2/5) | Ship a keyboard-driven service or file browser as one executable. |
| 65 | [termcolor 3.3.0](https://pypi.org/project/termcolor/3.3.0/) | 312 | 26 (3/3/2/2) | Add small colored success and failure messages. |
| 66 | [humanize 4.16.0](https://pypi.org/project/humanize/4.16.0/) | 458 | 25 (2/4/1/2) | Show readable file sizes, durations and counts in reports. |
| 67 | [redis 8.1.0](https://pypi.org/project/redis/8.1.0/) | 155 | 25 (3/4/3/4) | Inspect a Redis-backed queue or cache from a portable command. |
| 68 | [gitpython 3.1.61](https://pypi.org/project/gitpython/3.1.61/) | 156 | 25 (3/4/3/4) | Summarize repository state and automate release preparation. |
| 69 | [websocket-client 1.9.2](https://pypi.org/project/websocket-client/1.9.2/) | 179 | 25 (3/4/3/4) | Watch a service's live status over a synchronous WebSocket. |
| 70 | [docker 7.2.0](https://pypi.org/project/docker/7.2.0/) | 214 | 25 (3/4/3/4) | Manage containers through a small operator CLI. |
| 71 | [prometheus-client 0.26.0](https://pypi.org/project/prometheus-client/0.26.0/) | 232 | 25 (3/4/3/4) | Export a local worker's counters and timings for monitoring. |
| 72 | [pymysql 1.2.0](https://pypi.org/project/pymysql/1.2.0/) | 332 | 25 (3/4/3/4) | Export or inspect MySQL records without a CPython database extension. |
| 73 | [tzlocal 5.4.4](https://pypi.org/project/tzlocal/5.4.4/) | 175 | 24 (3/3/3/3) | Display the user's local timezone when scheduling a command. |
| 74 | [croniter 6.2.4](https://pypi.org/project/croniter/6.2.4/) | 345 | 24 (3/4/1/3) | Preview when a scheduled job will run next. |
| 75 | [loguru 0.7.3](https://pypi.org/project/loguru/0.7.3/) | 369 | 23 (2/4/2/3) | Give small CLIs useful logging with less setup code. |
| 76 | [deepdiff 9.1.0](https://pypi.org/project/deepdiff/9.1.0/) | 414 | 23 (2/4/2/3) | Explain what changed between two configuration or inventory snapshots. |
| 77 | [text-unidecode 1.3](https://pypi.org/project/text-unidecode/1.3/) | 367 | 22 (2/3/2/2) | Create ASCII identifiers from non-ASCII labels in imported data. |
| 78 | [requests-file 3.0.1](https://pypi.org/project/requests-file/3.0.1/) | 417 | 22 (2/3/2/2) | Let a data-import client read local fixtures using the same session API as HTTP. |
| 79 | [networkx 3.6.1](https://pypi.org/project/networkx/3.6.1/) | 152 | 22 (3/4/2/4) | Inspect dependency graphs and compute routes in an operations CLI. |
| 80 | [httpx-sse 0.4.3](https://pypi.org/project/httpx-sse/0.4.3/) | 170 | 22 (3/4/2/4) | Consume streamed API progress and text events in a CLI. |
| 81 | [slack-sdk 3.44.1](https://pypi.org/project/slack-sdk/3.44.1/) | 346 | 22 (3/4/2/4) | Send an explicit user-requested notification from an automation tool. |
| 82 | [schema 0.7.8](https://pypi.org/project/schema/0.7.8/) | 836 | 21 (1/4/1/2) | Validate a small config without adopting a larger model framework. |
| 83 | [natsort 8.4.0](https://pypi.org/project/natsort/8.4.0/) | 957 | 21 (1/4/1/2) | Sort filenames and version-like labels the way people expect. |
| 84 | [cattrs 26.1.0](https://pypi.org/project/cattrs/26.1.0/) | 463 | 20 (2/4/1/3) | Convert JSON dictionaries to and from structured application records. |
| 85 | [shellingham 1.5.4](https://pypi.org/project/shellingham/1.5.4/) | 113 | 20 (4/3/2/4) | Suggest the correct completion command for a user's shell. |
| 86 | [google-cloud-storage 3.13.1](https://pypi.org/project/google-cloud-storage/3.13.1/) | 165 | 20 (3/4/3/5) | Move files between a CLI and Google Cloud Storage. |
| 87 | [anthropic 1.4.0](https://pypi.org/project/anthropic/1.4.0/) | 208 | 20 (3/4/3/5) | Build an assistant CLI using an external Messages API. |
| 88 | [azure-storage-blob 12.30.1](https://pypi.org/project/azure-storage-blob/12.30.1/) | 280 | 20 (3/4/3/5) | Build a Blob Storage backup or export utility. |
| 89 | [qrcode 8.2](https://pypi.org/project/qrcode/8.2/) | 748 | 19 (1/4/2/3) | Create shareable QR codes for a URL or device setup flow. |
| 90 | [dulwich 1.2.14](https://pypi.org/project/dulwich/1.2.14/) | 379 | 18 (2/4/2/4) | Read and write Git repositories without requiring the Git executable. |
| 91 | [smart-open 8.0.1](https://pypi.org/project/smart-open/8.0.1/) | 416 | 18 (2/4/2/4) | Stream local or cloud-hosted text through an ETL command. |
| 92 | [pygithub 2.10.0](https://pypi.org/project/pygithub/2.10.0/) | 436 | 18 (2/4/2/4) | Build a GitHub release or repository-reporting CLI. |
| 93 | [pg8000 1.31.5](https://pypi.org/project/pg8000/1.31.5/) | 580 | 18 (2/4/2/4) | Query PostgreSQL from a standalone reporting tool using a pure Python driver. |
| 94 | [watchdog 6.0.0](https://pypi.org/project/watchdog/6.0.0/) | 307 | 17 (3/4/2/5) | Watch a directory and trigger a local automation workflow. |
| 95 | [kubernetes 36.0.3](https://pypi.org/project/kubernetes/36.0.3/) | 317 | 17 (3/4/2/5) | Ship a focused cluster inventory or rollout inspection tool. |
| 96 | [feedparser 6.0.14](https://pypi.org/project/feedparser/6.0.14/) | 847 | 16 (1/4/1/3) | Build an offline-friendly RSS/Atom digest reader. |
| 97 | [argcomplete 3.7.2](https://pypi.org/project/argcomplete/3.7.2/) | 397 | 15 (2/3/3/4) | Offer shell completion for an argparse-based tool. |
| 98 | [pycountry 26.2.16](https://pypi.org/project/pycountry/26.2.16/) | 658 | 13 (1/3/2/3) | Normalize country and language codes in imported business data. |
| 99 | [pyotp 2.10.0](https://pypi.org/project/pyotp/2.10.0/) | 625 | 12 (2/3/2/4) | Generate or verify one-time codes for an explicitly configured authentication workflow. |
| 100 | [questionary 2.1.1](https://pypi.org/project/questionary/2.1.1/) | 865 | 11 (1/4/1/4) | Guide users through a setup wizard instead of requiring flags. |

## Acceptance goals and current blockers

### 1. certifi

**HTTP & APIs.** Bundle the certificate roots expected by Python HTTP clients.

**Planned first acceptance test:** Resolve and read the pinned CA bundle after source and caches are removed, then verify a trusted and an untrusted TLS chain via the client integration.

**Current compile evidence:** All 5 Python sources compile. This audit has not verified imports, dependency closure or the proposed application behavior.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 21 audited distributions.

Package purpose/API references: [certifi 2026.7.22 — maintainer-provided PyPI project description](https://pypi.org/project/certifi/2026.7.22/).

### 2. python-dotenv

**Configuration & validation.** Load a local application's configuration from a .env file.

**Planned first acceptance test:** Parse quotes, comments and variable expansion; preserve existing environment values by default and work from a detached bundle.

**Current compile evidence:** First compile stop in dotenv/cli.py: SyntaxError: expected '@id', got '(' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 18 audited distributions.

Package purpose/API references: [python-dotenv 1.2.3 — maintainer-provided PyPI project description](https://pypi.org/project/python-dotenv/1.2.3/).

### 3. platformdirs

**Configuration & validation.** Save settings and caches in the expected user directories.

**Planned first acceptance test:** Resolve config, data and cache paths under isolated homes on macOS and Linux; assert no writes escape the fixture root.

**Current compile evidence:** First compile stop in platformdirs/__init__.py: SyntaxError: *args should be placed before **kwargs Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 16 audited distributions.

Package purpose/API references: [platformdirs 4.11.7 — maintainer-provided PyPI project description](https://pypi.org/project/platformdirs/4.11.7/).

### 4. pathspec

**Files & documents.** Respect .gitignore-style filters in backup and search tools.

**Planned first acceptance test:** Match nested files against comments, negation and directory patterns, then verify a bundled backup excludes exactly the expected paths.

**Current compile evidence:** First compile stop in pathspec/_backends/agg.py: SyntaxError: expected an expression, got = Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 6 audited distributions.

Package purpose/API references: [pathspec 1.1.1 — maintainer-provided PyPI project description](https://pypi.org/project/pathspec/1.1.1/).

### 5. python-dateutil

**Data & scheduling.** Interpret user-supplied dates and calendar offsets in reports.

**Planned first acceptance test:** Parse ISO timestamps, apply relative month arithmetic and run fixed recurrence cases with explicit time zones and expected outputs.

**Current compile evidence:** First compile stop in dateutil/easter.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 36 audited distributions.

Package purpose/API references: [python-dateutil 2.9.0.post0 — maintainer-provided PyPI project description](https://pypi.org/project/python-dateutil/2.9.0.post0/).

### 6. click

**CLI & terminal.** Build a deploy CLI with subcommands, flags and useful help.

**Planned first acceptance test:** A two-command app handles --help, an invalid integer, default values and exit codes identically before and after bundling.

**Current compile evidence:** First compile stop in click/_compat.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 37 audited distributions.

Package purpose/API references: [click 8.5.0 — maintainer-provided PyPI project description](https://pypi.org/project/click/8.5.0/), [Click documentation](https://click.palletsprojects.com/en/stable/).

### 7. jmespath

**HTTP & APIs.** Filter JSON API responses into useful command output.

**Planned first acceptance test:** Run projections, filters, sorting and missing-key queries over a pinned nested JSON fixture with exact expected values.

**Current compile evidence:** First compile stop in jmespath/exceptions.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 5 audited distributions.

Package purpose/API references: [jmespath 1.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/jmespath/1.1.0/).

### 8. cachetools

**Data & scheduling.** Avoid repeated expensive API lookups in a long-running command.

**Planned first acceptance test:** Exercise LRU eviction and TTL expiry with a controlled clock, plus decorator cache hits, in a bundled app.

**Current compile evidence:** First compile stop in cachetools/__init__.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 9 audited distributions.

Package purpose/API references: [cachetools 7.1.8 — maintainer-provided PyPI project description](https://pypi.org/project/cachetools/7.1.8/).

### 9. requests

**HTTP & APIs.** Build a familiar REST API client or authenticated download tool.

**Planned first acceptance test:** Against a local HTTP/TLS fixture, exercise JSON, redirects, timeouts, streaming and invalid certificates; rerun from a detached executable.

**Current compile evidence:** First compile stop in requests/__init__.py: SyntaxError: expected ')', got '@fstr-begin' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 105 audited distributions.

Package purpose/API references: [requests 2.34.2 — maintainer-provided PyPI project description](https://pypi.org/project/requests/2.34.2/), [Requests quickstart](https://requests.readthedocs.io/en/latest/user/quickstart/).

### 10. httpx

**HTTP & APIs.** Build a modern HTTP client with bounded timeouts and reusable sessions.

**Planned first acceptance test:** A synchronous client performs JSON requests, streams a response, handles timeout and rejects invalid TLS against controlled fixtures; async remains a separate goal.

**Current compile evidence:** First compile stop in httpx/_api.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 55 audited distributions.

Package purpose/API references: [httpx 0.28.1 — maintainer-provided PyPI project description](https://pypi.org/project/httpx/0.28.1/), [HTTPX documentation](https://www.python-httpx.org/).

### 11. tzdata

**Data & scheduling.** Ship a versioned timezone database with an offline app.

**Planned first acceptance test:** Read version constants and known TZif resources from a detached bundle; timezone conversion requires a separately tested consumer such as zoneinfo.

**Current compile evidence:** All 22 Python sources compile. This audit has not verified imports, dependency closure or the proposed application behavior.

**Estimated effort: 1/5.** A narrow resource/API check is the first target; no broad compatibility claim is implied. Potential direct reach: 17 audited distributions.

Package purpose/API references: [tzdata 2026.3 — maintainer-provided PyPI project description](https://pypi.org/project/tzdata/2026.3/).

### 12. filelock

**Files & documents.** Prevent two copies of a CLI from corrupting the same cache.

**Planned first acceptance test:** Two subprocesses contend for one lock; assert exclusion, bounded timeout and lock release after the owner exits.

**Current compile evidence:** First compile stop in filelock/_api.py: SyntaxError: expected '=' after '!' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 13 audited distributions.

Package purpose/API references: [filelock 3.32.5 — maintainer-provided PyPI project description](https://pypi.org/project/filelock/3.32.5/).

### 13. tqdm

**CLI & terminal.** Show progress during a long file conversion or download.

**Planned first acceptance test:** Iterate a bounded workload, capture progress in a pseudo-terminal and ensure redirected output and cancellation behave correctly.

**Current compile evidence:** First compile stop in tqdm/_monitor.py: SyntaxError: expected ')', got 'and' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 18 audited distributions.

Package purpose/API references: [tqdm 4.70.0 — maintainer-provided PyPI project description](https://pypi.org/project/tqdm/4.70.0/).

### 14. idna

**HTTP & APIs.** Handle internationalized hostnames in API and URL tools.

**Planned first acceptance test:** Encode and decode fixed IDNA domain vectors; reject invalid labels and compare strict and UTS 46 modes against pinned CPython.

**Current compile evidence:** First compile stop in idna/cli.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 16 audited distributions.

Package purpose/API references: [idna 3.19 — maintainer-provided PyPI project description](https://pypi.org/project/idna/3.19/).

### 15. jsonschema

**Configuration & validation.** Give clear validation errors for JSON config and API payloads.

**Planned first acceptance test:** Validate and reject Draft 2020-12 fixtures with local references; verify error paths without fetching remote schemas.

**Current compile evidence:** First compile stop in jsonschema/__init__.py: SyntaxError: expected ')', got '@str' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 25 audited distributions.

Package purpose/API references: [jsonschema 4.26.0 — maintainer-provided PyPI project description](https://pypi.org/project/jsonschema/4.26.0/).

### 16. urllib3

**HTTP & APIs.** Provide connection pooling and bounded retries beneath API clients.

**Planned first acceptance test:** Reuse a connection to a local fixture, enforce retry limits, stream a body and reject invalid TLS; verify cleanup after failure.

**Current compile evidence:** First compile stop in urllib3/__init__.py: SyntaxError: expected an expression, got else Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 35 audited distributions.

Package purpose/API references: [urllib3 2.7.0 — maintainer-provided PyPI project description](https://pypi.org/project/urllib3/2.7.0/).

### 17. jinja2

**Files & documents.** Generate config files and reports from templates and data.

**Planned first acceptance test:** Render loops, filters and template includes from bundled resources; verify undefined-variable handling and HTML autoescape in report mode.

**Current compile evidence:** First compile stop in jinja2/async_utils.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 34 audited distributions.

Package purpose/API references: [jinja2 3.1.6 — maintainer-provided PyPI project description](https://pypi.org/project/jinja2/3.1.6/).

### 18. tabulate

**CLI & terminal.** Print API, CSV and database results as aligned tables.

**Planned first acceptance test:** Render fixed rows with headers, Unicode cells, missing values and numeric alignment against exact expected output.

**Current compile evidence:** First compile stop in tabulate/__init__.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 5 audited distributions.

Package purpose/API references: [tabulate 0.10.0 — maintainer-provided PyPI project description](https://pypi.org/project/tabulate/0.10.0/).

### 19. tomlkit

**Configuration & validation.** Edit a user's TOML config while preserving comments.

**Planned first acceptance test:** Change one nested setting, write the document and assert comments, ordering and untouched values survive the round trip.

**Current compile evidence:** First compile stop in tomlkit/_types.py: SyntaxError: expected ')', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 7 audited distributions.

Package purpose/API references: [tomlkit 0.15.1 — maintainer-provided PyPI project description](https://pypi.org/project/tomlkit/0.15.1/).

### 20. tenacity

**Data & scheduling.** Retry temporary service failures without retrying forever.

**Planned first acceptance test:** A deterministic fixture fails twice then succeeds; assert call count, stop conditions and exception filters with an injected no-op sleep.

**Current compile evidence:** First compile stop in tenacity/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 14 audited distributions.

Package purpose/API references: [tenacity 9.1.4 — maintainer-provided PyPI project description](https://pypi.org/project/tenacity/9.1.4/).

### 21. beautifulsoup4

**Files & documents.** Extract links or records from saved HTML and web responses.

**Planned first acceptance test:** Using the standard html.parser backend, extract a table and links from malformed HTML without lxml; compare expected normalized text.

**Current compile evidence:** First compile stop in bs4/__init__.py: SyntaxError: expected ')', got '' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 12 audited distributions.

Package purpose/API references: [beautifulsoup4 4.15.0 — maintainer-provided PyPI project description](https://pypi.org/project/beautifulsoup4/4.15.0/).

### 22. pyyaml

**Configuration & validation.** Read the YAML configuration used by deployment and operations tools.

**Planned first acceptance test:** After an approved pure-wheel or native strategy exists, safe-load a bounded config, reject Python object tags and round-trip plain values.

**Current compile evidence:** The pinned release has no selected generic Python 3 pure wheel; its native/platform artifact is outside the current installer contract.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 64 audited distributions.

Package purpose/API references: [pyyaml 6.0.3 — maintainer-provided PyPI project description](https://pypi.org/project/pyyaml/6.0.3/), [PyYAML parser and safe-load documentation](https://pyyaml.org/wiki/PyYAMLDocumentation).

### 23. pydantic

**Configuration & validation.** Validate typed API and configuration models with useful error messages.

**Planned first acceptance test:** After resolving pydantic-core's native dependency, validate nested models and compare coercions, errors and JSON serialization with pinned CPython.

**Current compile evidence:** First compile stop in pydantic/__init__.py: SyntaxError: expected '@id', got '(' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 71 audited distributions.

Package purpose/API references: [pydantic 2.13.5 — maintainer-provided PyPI project description](https://pypi.org/project/pydantic/2.13.5/).

### 24. sortedcontainers

**Data & scheduling.** Maintain ordered queues and indexes in a data-processing tool.

**Planned first acceptance test:** Insert and delete ordered keys, bisect ranges and compare results with a sorted reference across randomized bounded operations.

**Current compile evidence:** First compile stop in sortedcontainers/sorteddict.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 5 audited distributions.

Package purpose/API references: [sortedcontainers 2.4.0 — maintainer-provided PyPI project description](https://pypi.org/project/sortedcontainers/2.4.0/).

### 25. packaging

**Validation & formats.** Interpret application version constraints in an updater or plugin selector.

**Planned first acceptance test:** Compare normalized versions, prereleases and specifier membership using application-version fixtures; do not run Python package installation tooling.

**Current compile evidence:** First compile stop in packaging/_elffile.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 94 audited distributions.

Package purpose/API references: [packaging 26.3 — maintainer-provided PyPI project description](https://pypi.org/project/packaging/26.3/).

### 26. charset-normalizer

**HTTP & APIs.** Read imported text or HTTP bodies whose encoding is unknown.

**Planned first acceptance test:** Detect bounded UTF-8 and legacy-encoding fixtures and verify decoded text, ambiguity handling and corrupt-byte behavior.

**Current compile evidence:** First compile stop in charset_normalizer/api.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 7 audited distributions.

Package purpose/API references: [charset-normalizer 3.5.1 — maintainer-provided PyPI project description](https://pypi.org/project/charset-normalizer/3.5.1/).

### 27. attrs

**Configuration & validation.** Define structured application records without repetitive class code.

**Planned first acceptance test:** Create an immutable validated record; exercise defaults, converters, equality and serialization in a bundled app.

**Current compile evidence:** First compile stop in attr/_cmp.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 15 audited distributions.

Package purpose/API references: [attrs 26.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/attrs/26.1.0/).

### 28. rich

**CLI & terminal.** Make inventory and diagnostic output readable in a terminal.

**Planned first acceptance test:** Render a Unicode table and progress display; pipe output to a file and verify it contains no terminal control codes.

**Current compile evidence:** First compile stop in rich/__init__.py: SyntaxError: *args should be placed before **kwargs Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 35 audited distributions.

Package purpose/API references: [rich 15.0.0 — maintainer-provided PyPI project description](https://pypi.org/project/rich/15.0.0/), [Rich console behavior](https://rich.readthedocs.io/en/latest/console.html).

### 29. boto3

**Service integrations.** Build an S3 backup, inventory or artifact-download command.

**Planned first acceptance test:** Use pinned service models and a local S3-compatible fixture to list and transfer a bounded object, verifying signing, pagination and resource bundling.

**Current compile evidence:** First compile stop in boto3/__init__.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 33 audited distributions.

Package purpose/API references: [boto3 1.43.89 — maintainer-provided PyPI project description](https://pypi.org/project/boto3/1.43.89/), [Boto3 S3 examples](https://docs.aws.amazon.com/boto3/latest/guide/s3-examples.html).

### 30. colorama

**CLI & terminal.** Keep colored CLI output compatible with existing Python scripts.

**Planned first acceptance test:** Initialize, print ANSI colors and deinitialize twice on all four supported targets without changing redirected plain text.

**Current compile evidence:** First compile stop in colorama/ansitowin32.py: SyntaxError: expected an expression, got else Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 25 audited distributions.

Package purpose/API references: [colorama 0.4.6 — maintainer-provided PyPI project description](https://pypi.org/project/colorama/0.4.6/).

### 31. wcwidth

**CLI & terminal.** Keep Unicode table columns and terminal cursors aligned.

**Planned first acceptance test:** Check combining characters, wide CJK characters and controls against version-pinned width fixtures.

**Current compile evidence:** First compile stop in wcwidth/_clip.py: SyntaxError: expected an expression, got = Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 3 audited distributions.

Package purpose/API references: [wcwidth 0.8.3 — maintainer-provided PyPI project description](https://pypi.org/project/wcwidth/0.8.3/).

### 32. isodate

**Data & scheduling.** Read ISO 8601 dates and durations from API exports.

**Planned first acceptance test:** Round-trip fixed timestamps, offsets and durations and reject malformed inputs against pinned reference fixtures.

**Current compile evidence:** First compile stop in isodate/isodates.py: SyntaxError: expected an expression, got = Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 11 audited distributions.

Package purpose/API references: [isodate 0.7.2 — maintainer-provided PyPI project description](https://pypi.org/project/isodate/0.7.2/).

### 33. tomli-w

**Configuration & validation.** Export a generated application configuration as TOML.

**Planned first acceptance test:** Serialize nested tables, Unicode strings and date values; read them with an independent TOML parser and compare values.

**Current compile evidence:** First compile stop in tomli_w/_writer.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 11 audited distributions.

Package purpose/API references: [tomli-w 1.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/tomli-w/1.2.0/).

### 34. typer

**CLI & terminal.** Turn typed command functions into a friendly operator CLI.

**Planned first acceptance test:** A typed command accepts an integer option, generates help and reports invalid input without a Python installation.

**Current compile evidence:** First compile stop in typer/_click/_compat.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 12 audited distributions.

Package purpose/API references: [typer 0.27.2 — maintainer-provided PyPI project description](https://pypi.org/project/typer/0.27.2/).

### 35. openpyxl

**Files & documents.** Read and update business spreadsheets from a standalone CLI.

**Planned first acceptance test:** Edit cells and styles in a small XLSX fixture, save it and independently verify workbook values, sheet names and ZIP/XML integrity.

**Current compile evidence:** First compile stop in openpyxl/cell/cell.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 7 audited distributions.

Package purpose/API references: [openpyxl 3.1.5 — maintainer-provided PyPI project description](https://pypi.org/project/openpyxl/3.1.5/).

### 36. more-itertools

**Data & scheduling.** Process large iterables in bounded chunks without hand-written loops.

**Planned first acceptance test:** Test chunked, batched and peekable workflows on iterators, including empty and uneven inputs, without materializing a large source.

**Current compile evidence:** First compile stop in more_itertools/more.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 8 audited distributions.

Package purpose/API references: [more-itertools 11.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/more-itertools/11.1.0/).

### 37. requests-toolbelt

**HTTP & APIs.** Upload large files or inspect HTTP exchanges from an API CLI.

**Planned first acceptance test:** Stream a bounded multipart upload to a local server and verify boundary, fields, content length and cleanup without loading the entire file.

**Current compile evidence:** First compile stop in requests_toolbelt/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 7 audited distributions.

Package purpose/API references: [requests-toolbelt 1.0.0 — maintainer-provided PyPI project description](https://pypi.org/project/requests-toolbelt/1.0.0/).

### 38. xlsxwriter

**Files & documents.** Generate spreadsheet reports for people who do not use Python.

**Planned first acceptance test:** Write strings, numbers, formulas and a chart to XLSX; independently inspect the ZIP/XML parts and open the file with a spreadsheet reader.

**Current compile evidence:** First compile stop in xlsxwriter/chart.py: SyntaxError: expected ')', got '@fstr-begin' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 3 audited distributions.

Package purpose/API references: [xlsxwriter 3.2.9 — maintainer-provided PyPI project description](https://pypi.org/project/xlsxwriter/3.2.9/).

### 39. fsspec

**Files & documents.** Use one file interface for local and remote input workflows.

**Planned first acceptance test:** Read and write through local and memory filesystems and verify globbing and context cleanup; remote backends need separate dependency evidence.

**Current compile evidence:** First compile stop in fsspec/__init__.py: SyntaxError: expected an expression, got else Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 16 audited distributions.

Package purpose/API references: [fsspec 2026.7.0 — maintainer-provided PyPI project description](https://pypi.org/project/fsspec/2026.7.0/).

### 40. psutil

**Operations & logging.** Build a portable process and resource diagnostic executable.

**Planned first acceptance test:** After a native API strategy exists, inspect a controlled child process and system memory on all four targets with documented permission errors.

**Current compile evidence:** The pinned release has no selected generic Python 3 pure wheel; its native/platform artifact is outside the current installer contract.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 20 audited distributions.

Package purpose/API references: [psutil 7.2.2 — maintainer-provided PyPI project description](https://pypi.org/project/psutil/7.2.2/).

### 41. uritemplate

**HTTP & APIs.** Construct API endpoint URLs from templates without ad hoc escaping.

**Planned first acceptance test:** Expand RFC 6570 scalar, list and mapping fixtures and assert percent-encoding and omitted-variable behavior.

**Current compile evidence:** First compile stop in uritemplate/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 1 audited distributions.

Package purpose/API references: [uritemplate 4.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/uritemplate/4.2.0/).

### 42. jsonpointer

**HTTP & APIs.** Address a single field in JSON config or API responses.

**Planned first acceptance test:** Resolve and update nested pointers, escaped slashes and array indexes; verify errors for absent paths and invalid indexes.

**Current compile evidence:** First compile stop in jsonpointer.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 2 audited distributions.

Package purpose/API references: [jsonpointer 3.1.1 — maintainer-provided PyPI project description](https://pypi.org/project/jsonpointer/3.1.1/).

### 43. jsonpatch

**HTTP & APIs.** Apply or generate reviewable changes to JSON configuration.

**Planned first acceptance test:** Apply add, remove, replace, move, copy and test operations; compare a generated patch's result and test-failure behavior.

**Current compile evidence:** First compile stop in jsonpatch.py: SyntaxError: expected ')', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 2 audited distributions.

Package purpose/API references: [jsonpatch 1.33 — maintainer-provided PyPI project description](https://pypi.org/project/jsonpatch/1.33/).

### 44. websockets

**HTTP & APIs.** Add a live event stream to a terminal application.

**Planned first acceptance test:** Use the documented synchronous client with a local echo server, test invalid handshakes and shut down cleanly; async support is a separate milestone.

**Current compile evidence:** First compile stop in websockets/asyncio/client.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 21 audited distributions.

Package purpose/API references: [websockets 17.1 — maintainer-provided PyPI project description](https://pypi.org/project/websockets/17.1/).

### 45. markdown-it-py

**Files & documents.** Render CommonMark content for previews and terminal documentation.

**Planned first acceptance test:** Parse CommonMark fixture cases, inspect token nesting and render exact HTML with unsafe raw HTML disabled in the chosen preset.

**Current compile evidence:** First compile stop in markdown_it/_punycode.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 3 audited distributions.

Package purpose/API references: [markdown-it-py 4.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/markdown-it-py/4.2.0/).

### 46. soupsieve

**Files & documents.** Select HTML elements with CSS selectors in a scraping CLI.

**Planned first acceptance test:** Run attribute, descendant and nth-child selectors on a fixed Beautiful Soup document and verify escaped selectors and no-match results.

**Current compile evidence:** First compile stop in soupsieve/__init__.py: SyntaxError: *args should be placed before **kwargs Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 1 audited distributions.

Package purpose/API references: [soupsieve 2.9.2 — maintainer-provided PyPI project description](https://pypi.org/project/soupsieve/2.9.2/).

### 47. prompt-toolkit

**CLI & terminal.** Add history and completion to an interactive database or API console.

**Planned first acceptance test:** A pseudo-terminal test enters text, accepts completion, recalls history and interrupts input without corrupting terminal state.

**Current compile evidence:** First compile stop in prompt_toolkit/application/application.py: SyntaxError: expected '=', got '@eol' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 6 audited distributions.

Package purpose/API references: [prompt-toolkit 3.0.53 — maintainer-provided PyPI project description](https://pypi.org/project/prompt-toolkit/3.0.53/).

### 48. ruamel-yaml

**Configuration & validation.** Update deployment YAML without deleting the user's comments.

**Planned first acceptance test:** Round-trip a commented mapping and anchors, change one scalar and verify preserved comments and documented duplicate-key errors.

**Current compile evidence:** First compile stop in ruamel/yaml/comments.py: SyntaxError: f-string: single '}' is not allowed Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 6 audited distributions.

Package purpose/API references: [ruamel-yaml 0.19.1 — maintainer-provided PyPI project description](https://pypi.org/project/ruamel-yaml/0.19.1/).

### 49. defusedxml

**Files & documents.** Read external XML using parsers designed to reject dangerous XML features.

**Planned first acceptance test:** Parse ordinary XML and reject DTD/entity-expansion fixture cases without external file or network access; compare exception classes.

**Current compile evidence:** First compile stop in defusedxml/ElementTree.py: SyntaxError: finally clause is not supported yet Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 9 audited distributions.

Package purpose/API references: [defusedxml 0.7.1 — maintainer-provided PyPI project description](https://pypi.org/project/defusedxml/0.7.1/).

### 50. structlog

**Operations & logging.** Emit structured machine-readable logs from an operational tool.

**Planned first acceptance test:** Bind request context, log an exception and serialize JSON; verify event keys, context clearing and no ANSI output in a file sink.

**Current compile evidence:** First compile stop in structlog/__init__.py: SyntaxError: expected ')', got '@str' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 0 audited distributions.

Package purpose/API references: [structlog 26.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/structlog/26.1.0/).

### 51. openai

**Service integrations.** Ship an AI-assisted command that calls an external model API.

**Planned first acceptance test:** Against a local protocol fixture, send a Responses request, decode streamed text and propagate API errors; no paid network call is needed.

**Current compile evidence:** First compile stop in openai/__init__.py: SyntaxError: expected ')', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 16 audited distributions.

Package purpose/API references: [openai 3.8.0 — maintainer-provided PyPI project description](https://pypi.org/project/openai/3.8.0/).

### 52. dnspython

**Validation & formats.** Build a bounded DNS troubleshooting or service-discovery command.

**Planned first acceptance test:** Query a controlled DNS fixture for A and TXT records and verify timeout, NXDOMAIN and malformed-response handling.

**Current compile evidence:** First compile stop in dns/_asyncbackend.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 5 audited distributions.

Package purpose/API references: [dnspython 2.8.0 — maintainer-provided PyPI project description](https://pypi.org/project/dnspython/2.8.0/).

### 53. semver

**Data & scheduling.** Compare application versions and decide whether an update is newer.

**Planned first acceptance test:** Parse release, prerelease and build-metadata examples and verify ordering, bumping and invalid-version errors.

**Current compile evidence:** First compile stop in semver/__init__.py: SyntaxError: expected '@id', got 'match' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 1 audited distributions.

Package purpose/API references: [semver 3.0.4 — maintainer-provided PyPI project description](https://pypi.org/project/semver/3.0.4/).

### 54. python-slugify

**Data & scheduling.** Generate stable readable filenames or URL slugs from titles.

**Planned first acceptance test:** Slugify multilingual titles, punctuation and length-limited strings with an explicit transliteration backend and expected outputs.

**Current compile evidence:** First compile stop in slugify/slugify.py: SyntaxError: EOL while scanning string literal Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 1 audited distributions.

Package purpose/API references: [python-slugify 8.0.4 — maintainer-provided PyPI project description](https://pypi.org/project/python-slugify/8.0.4/).

### 55. prettytable

**CLI & terminal.** Produce sortable terminal tables for an inventory tool.

**Planned first acceptance test:** Build a named-column table, sort by one column and compare plain-text and HTML exports with fixture outputs.

**Current compile evidence:** First compile stop in prettytable/colortable.py: SyntaxError: expected ')', got '+' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 1 audited distributions.

Package purpose/API references: [prettytable 3.18.0 — maintainer-provided PyPI project description](https://pypi.org/project/prettytable/3.18.0/).

### 56. pypdf

**Files & documents.** Merge, split or inspect PDFs without a desktop application.

**Planned first acceptance test:** Merge two bounded unencrypted fixture PDFs, select pages and read metadata; verify page count and output with an independent PDF reader.

**Current compile evidence:** First compile stop in pypdf/__init__.py: SyntaxError: expected an expression, got = Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 3 audited distributions.

Package purpose/API references: [pypdf 6.17.0 — maintainer-provided PyPI project description](https://pypi.org/project/pypdf/6.17.0/), [pypdf merge documentation source](https://github.com/py-pdf/pypdf/blob/main/docs/user/merging-pdfs.md).

### 57. markupsafe

**Files & documents.** Preserve correct HTML escaping in generated reports.

**Planned first acceptance test:** After an approved pure-wheel or native strategy exists, test escaping, safe-string concatenation and double-escape avoidance against pinned fixtures.

**Current compile evidence:** The pinned release has no selected generic Python 3 pure wheel; its native/platform artifact is outside the current installer contract.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 8 audited distributions.

Package purpose/API references: [markupsafe 3.0.3 — maintainer-provided PyPI project description](https://pypi.org/project/markupsafe/3.0.3/).

### 58. pyjwt

**Validation & formats.** Verify signed tokens in an API diagnostic or local authentication workflow.

**Planned first acceptance test:** Use fixed HS256 vectors, an explicit algorithm allowlist and key; reject modified, expired and wrong-audience tokens before any scoped approval.

**Current compile evidence:** First compile stop in jwt/algorithms.py: SyntaxError: expected ')', got 'and' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 19 audited distributions.

Package purpose/API references: [pyjwt 2.13.0 — maintainer-provided PyPI project description](https://pypi.org/project/pyjwt/2.13.0/), [PyJWT documentation](https://pyjwt.readthedocs.io/en/stable/).

### 59. email-validator

**Validation & formats.** Validate and normalize email addresses in an import wizard.

**Planned first acceptance test:** Normalize fixed valid addresses and reject invalid syntax with DNS deliverability disabled; DNS-backed checks require separate fixtures.

**Current compile evidence:** First compile stop in email_validator/deliverability.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 2 audited distributions.

Package purpose/API references: [email-validator 2.3.0 — maintainer-provided PyPI project description](https://pypi.org/project/email-validator/2.3.0/).

### 60. markdown

**Files & documents.** Convert local Markdown notes into HTML reports.

**Planned first acceptance test:** Render headings, lists, links and fenced-code fixtures with explicitly selected extensions and compare normalized HTML.

**Current compile evidence:** First compile stop in markdown/blockprocessors.py: SyntaxError: expected newline after line continuation character Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 4 audited distributions.

Package purpose/API references: [markdown 3.10.3 — maintainer-provided PyPI project description](https://pypi.org/project/markdown/3.10.3/).

### 61. xmltodict

**Files & documents.** Turn XML exports into JSON-like records for import pipelines.

**Planned first acceptance test:** Parse repeated elements, attributes and Unicode text from a bounded XML fixture; verify round-trip shape and documented entity behavior.

**Current compile evidence:** First compile stop in xmltodict.py: SyntaxError: expected a literal, got 'lambda' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 3 audited distributions.

Package purpose/API references: [xmltodict 1.0.4 — maintainer-provided PyPI project description](https://pypi.org/project/xmltodict/1.0.4/).

### 62. marshmallow

**Configuration & validation.** Validate and serialize records in a data-import CLI.

**Planned first acceptance test:** Load a nested input, apply a default, reject unknown and invalid fields, then serialize a deterministic JSON-ready mapping.

**Current compile evidence:** First compile stop in marshmallow/class_registry.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 2 audited distributions.

Package purpose/API references: [marshmallow 4.3.1 — maintainer-provided PyPI project description](https://pypi.org/project/marshmallow/4.3.1/).

### 63. python-json-logger

**Operations & logging.** Integrate a CLI's logs with JSON log ingestion.

**Planned first acceptance test:** Format standard logging records and exceptions as JSON; parse each output line and verify configured field names and Unicode content.

**Current compile evidence:** First compile stop in pythonjsonlogger/core.py: SyntaxError: *args should be placed before **kwargs Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 1 audited distributions.

Package purpose/API references: [python-json-logger 4.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/python-json-logger/4.2.0/).

### 64. textual

**CLI & terminal.** Ship a keyboard-driven service or file browser as one executable.

**Planned first acceptance test:** A local two-pane list/detail app accepts arrow keys, resizes, exits cleanly and restores the terminal; no network is needed.

**Current compile evidence:** First compile stop in textual/__init__.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 1 audited distributions.

Package purpose/API references: [textual 8.2.8 — maintainer-provided PyPI project description](https://pypi.org/project/textual/8.2.8/).

### 65. termcolor

**CLI & terminal.** Add small colored success and failure messages.

**Planned first acceptance test:** Exercise forced color, disabled color and attribute combinations with exact escape-sequence assertions.

**Current compile evidence:** First compile stop in termcolor/termcolor.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 2 audited distributions.

Package purpose/API references: [termcolor 3.3.0 — maintainer-provided PyPI project description](https://pypi.org/project/termcolor/3.3.0/).

### 66. humanize

**CLI & terminal.** Show readable file sizes, durations and counts in reports.

**Planned first acceptance test:** Format fixed byte counts and timedeltas in the default locale and compare deterministic expected strings.

**Current compile evidence:** First compile stop in humanize/number.py: SyntaxError: f-string: single '}' is not allowed Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 0 audited distributions.

Package purpose/API references: [humanize 4.16.0 — maintainer-provided PyPI project description](https://pypi.org/project/humanize/4.16.0/).

### 67. redis

**Service integrations.** Inspect a Redis-backed queue or cache from a portable command.

**Planned first acceptance test:** Against an ephemeral local server, set/get expiring keys, decode replies and enforce connection timeouts; document the external server requirement.

**Current compile evidence:** First compile stop in redis/_himport_exec.py: SyntaxError: expected ')', got 'and' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 18 audited distributions.

Package purpose/API references: [redis 8.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/redis/8.1.0/).

### 68. gitpython

**Operations & logging.** Summarize repository state and automate release preparation.

**Planned first acceptance test:** Against a temporary repository and explicitly installed Git, inspect commits and dirty state; missing Git must produce a clear prerequisite error.

**Current compile evidence:** First compile stop in git/__init__.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 7 audited distributions.

Package purpose/API references: [gitpython 3.1.61 — maintainer-provided PyPI project description](https://pypi.org/project/gitpython/3.1.61/).

### 69. websocket-client

**HTTP & APIs.** Watch a service's live status over a synchronous WebSocket.

**Planned first acceptance test:** Exchange text and binary frames with a local server; verify ping/pong, close codes and bounded receive timeouts.

**Current compile evidence:** First compile stop in websocket/_abnf.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 7 audited distributions.

Package purpose/API references: [websocket-client 1.9.2 — maintainer-provided PyPI project description](https://pypi.org/project/websocket-client/1.9.2/).

### 70. docker

**Operations & logging.** Manage containers through a small operator CLI.

**Planned first acceptance test:** Against a local Docker API fixture, list containers and inspect one response; document that a real daemon is an external service requirement.

**Current compile evidence:** First compile stop in docker/api/build.py: SyntaxError: expected an expression, got @eol Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 6 audited distributions.

Package purpose/API references: [docker 7.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/docker/7.2.0/).

### 71. prometheus-client

**Operations & logging.** Export a local worker's counters and timings for monitoring.

**Planned first acceptance test:** Increment counters and histograms, generate exposition text and compare parsed metric samples; a local endpoint is a separately scoped transport test.

**Current compile evidence:** First compile stop in prometheus_client/aiohttp/exposition.py: SyntaxError: expected a literal, got '@id' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 5 audited distributions.

Package purpose/API references: [prometheus-client 0.26.0 — maintainer-provided PyPI project description](https://pypi.org/project/prometheus-client/0.26.0/).

### 72. pymysql

**Service integrations.** Export or inspect MySQL records without a CPython database extension.

**Planned first acceptance test:** Connect to a disposable fixture database, parameterize a query, round-trip Unicode and verify rollback and connection cleanup.

**Current compile evidence:** First compile stop in pymysql/_auth.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 10 audited distributions.

Package purpose/API references: [pymysql 1.2.0 — maintainer-provided PyPI project description](https://pypi.org/project/pymysql/1.2.0/).

### 73. tzlocal

**Data & scheduling.** Display the user's local timezone when scheduling a command.

**Planned first acceptance test:** Discover zone names from controlled OS fixtures on each target and report an explicit failure when local configuration is unavailable.

**Current compile evidence:** First compile stop in tzlocal/unix.py: SyntaxError: EOL while scanning string literal Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 7 audited distributions.

Package purpose/API references: [tzlocal 5.4.4 — maintainer-provided PyPI project description](https://pypi.org/project/tzlocal/5.4.4/).

### 74. croniter

**Data & scheduling.** Preview when a scheduled job will run next.

**Planned first acceptance test:** Compute next and previous times for fixed cron expressions, month boundaries and invalid syntax using explicit timezone fixtures.

**Current compile evidence:** First compile stop in croniter/croniter.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 0 audited distributions.

Package purpose/API references: [croniter 6.2.4 — maintainer-provided PyPI project description](https://pypi.org/project/croniter/6.2.4/).

### 75. loguru

**Operations & logging.** Give small CLIs useful logging with less setup code.

**Planned first acceptance test:** Add a file sink, emit formatted messages and an exception, remove the sink and verify deterministic output and cleanup.

**Current compile evidence:** First compile stop in loguru/_better_exceptions.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 3 audited distributions.

Package purpose/API references: [loguru 0.7.3 — maintainer-provided PyPI project description](https://pypi.org/project/loguru/0.7.3/).

### 76. deepdiff

**Data & scheduling.** Explain what changed between two configuration or inventory snapshots.

**Planned first acceptance test:** Compare nested mappings and lists with type changes and exclusions; verify deterministic change paths and serialization.

**Current compile evidence:** First compile stop in deepdiff/_multiprocessing.py: SyntaxError: expected ')', got '' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 1 audited distributions.

Package purpose/API references: [deepdiff 9.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/deepdiff/9.1.0/).

### 77. text-unidecode

**Data & scheduling.** Create ASCII identifiers from non-ASCII labels in imported data.

**Planned first acceptance test:** Transliterate a fixed multilingual corpus and verify its bundled lookup data remains readable after source and cache removal.

**Current compile evidence:** All 1 Python sources compile. This audit has not verified imports, dependency closure or the proposed application behavior.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 1 audited distributions.

Package purpose/API references: [text-unidecode 1.3 — maintainer-provided PyPI project description](https://pypi.org/project/text-unidecode/1.3/).

### 78. requests-file

**HTTP & APIs.** Let a data-import client read local fixtures using the same session API as HTTP.

**Planned first acceptance test:** Mount the file adapter and read a temporary file URI; verify missing files, encoded paths and access errors within an isolated fixture root.

**Current compile evidence:** First compile stop in requests_file/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 2 audited distributions.

Package purpose/API references: [requests-file 3.0.1 — maintainer-provided PyPI project description](https://pypi.org/project/requests-file/3.0.1/).

### 79. networkx

**Data & scheduling.** Inspect dependency graphs and compute routes in an operations CLI.

**Planned first acceptance test:** Build a directed fixture graph, topologically sort it, find a path and detect a cycle without optional scientific or drawing dependencies.

**Current compile evidence:** First compile stop in networkx/algorithms/approximation/clique.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 4 audited distributions.

Package purpose/API references: [networkx 3.6.1 — maintainer-provided PyPI project description](https://pypi.org/project/networkx/3.6.1/).

### 80. httpx-sse

**HTTP & APIs.** Consume streamed API progress and text events in a CLI.

**Planned first acceptance test:** Parse a local SSE fixture with multiline data, IDs and reconnect hints, and close the connection after cancellation.

**Current compile evidence:** First compile stop in httpx_sse/_api.py: SyntaxError: expected ')', got '@fstr-begin' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 3 audited distributions.

Package purpose/API references: [httpx-sse 0.4.3 — maintainer-provided PyPI project description](https://pypi.org/project/httpx-sse/0.4.3/).

### 81. slack-sdk

**Service integrations.** Send an explicit user-requested notification from an automation tool.

**Planned first acceptance test:** Use a local API fixture to serialize a message and handle rate-limit responses; tests use dummy tokens and never send a real message.

**Current compile evidence:** First compile stop in slack/deprecation.py: SyntaxError: expected ')', got '@str' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 1 audited distributions.

Package purpose/API references: [slack-sdk 3.44.1 — maintainer-provided PyPI project description](https://pypi.org/project/slack-sdk/3.44.1/).

### 82. schema

**Configuration & validation.** Validate a small config without adopting a larger model framework.

**Planned first acceptance test:** Validate a nested mapping with required and optional keys; assert failure messages for missing and incorrect values.

**Current compile evidence:** First compile stop in schema/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 0 audited distributions.

Package purpose/API references: [schema 0.7.8 — maintainer-provided PyPI project description](https://pypi.org/project/schema/0.7.8/).

### 83. natsort

**Data & scheduling.** Sort filenames and version-like labels the way people expect.

**Planned first acceptance test:** Sort mixed numeric filename fixtures, signed numbers and paths with explicit options and compare deterministic ordering.

**Current compile evidence:** First compile stop in natsort/__main__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 2/5.** Start with a bounded pure-Python API slice and its import/resource prerequisites. Potential direct reach: 0 audited distributions.

Package purpose/API references: [natsort 8.4.0 — maintainer-provided PyPI project description](https://pypi.org/project/natsort/8.4.0/).

### 84. cattrs

**Configuration & validation.** Convert JSON dictionaries to and from structured application records.

**Planned first acceptance test:** Round-trip nested attrs records and lists; reject an invalid field with the expected path and type error.

**Current compile evidence:** First compile stop in cattrs/_compat.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 0 audited distributions.

Package purpose/API references: [cattrs 26.1.0 — maintainer-provided PyPI project description](https://pypi.org/project/cattrs/26.1.0/).

### 85. shellingham

**CLI & terminal.** Suggest the correct completion command for a user's shell.

**Planned first acceptance test:** Launch controlled bash and zsh parent processes, identify each shell and return a documented result when detection is unavailable.

**Current compile evidence:** First compile stop in shellingham/_core.py: SyntaxError: expected ')', got '|' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 2 audited distributions.

Package purpose/API references: [shellingham 1.5.4 — maintainer-provided PyPI project description](https://pypi.org/project/shellingham/1.5.4/).

### 86. google-cloud-storage

**Service integrations.** Move files between a CLI and Google Cloud Storage.

**Planned first acceptance test:** With fake credentials and a local HTTP fixture, list blobs and stream a bounded upload/download; document authenticated live service testing separately.

**Current compile evidence:** First compile stop in google/cloud/_storage_v2/__init__.py: SyntaxError: expected ')', got '+' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 12 audited distributions.

Package purpose/API references: [google-cloud-storage 3.13.1 — maintainer-provided PyPI project description](https://pypi.org/project/google-cloud-storage/3.13.1/).

### 87. anthropic

**Service integrations.** Build an assistant CLI using an external Messages API.

**Planned first acceptance test:** Against a local protocol fixture, serialize a Messages request, decode streamed text and handle rate-limit errors with a dummy key.

**Current compile evidence:** First compile stop in anthropic/_base_client.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 13 audited distributions.

Package purpose/API references: [anthropic 1.4.0 — maintainer-provided PyPI project description](https://pypi.org/project/anthropic/1.4.0/).

### 88. azure-storage-blob

**Service integrations.** Build a Blob Storage backup or export utility.

**Planned first acceptance test:** Use a local emulator or HTTP fixture to upload, list and download a bounded blob, including continuation and service error handling.

**Current compile evidence:** First compile stop in azure/storage/blob/_blob_client.py: SyntaxError: expected ')', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 11 audited distributions.

Package purpose/API references: [azure-storage-blob 12.30.1 — maintainer-provided PyPI project description](https://pypi.org/project/azure-storage-blob/12.30.1/).

### 89. qrcode

**Validation & formats.** Create shareable QR codes for a URL or device setup flow.

**Planned first acceptance test:** Generate an SVG QR code using the SVG factory without Pillow; independently decode the fixture payload and verify detached output writing.

**Current compile evidence:** First compile stop in qrcode/__init__.py: SyntaxError: *args should be placed before **kwargs Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 1 audited distributions.

Package purpose/API references: [qrcode 8.2 — maintainer-provided PyPI project description](https://pypi.org/project/qrcode/8.2/).

### 90. dulwich

**Operations & logging.** Read and write Git repositories without requiring the Git executable.

**Planned first acceptance test:** Create a local repository, write a commit and read objects after bundling, with external Git absent and no optional accelerator dependency.

**Current compile evidence:** First compile stop in dulwich/aiohttp/server.py: SyntaxError: expected statement end Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 1 audited distributions.

Package purpose/API references: [dulwich 1.2.14 — maintainer-provided PyPI project description](https://pypi.org/project/dulwich/1.2.14/).

### 91. smart-open

**Files & documents.** Stream local or cloud-hosted text through an ETL command.

**Planned first acceptance test:** Stream plain and compressed local fixtures in text and binary modes; verify encoding and close behavior before adding a separately tested cloud backend.

**Current compile evidence:** First compile stop in smart_open/azure.py: SyntaxError: expected ')', got '|' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 3 audited distributions.

Package purpose/API references: [smart-open 8.0.1 — maintainer-provided PyPI project description](https://pypi.org/project/smart-open/8.0.1/).

### 92. pygithub

**Service integrations.** Build a GitHub release or repository-reporting CLI.

**Planned first acceptance test:** Using a local API fixture and dummy token, paginate repository data and decode rate-limit errors without sending a live mutation.

**Current compile evidence:** First compile stop in github/AccessToken.py: SyntaxError: expected an expression, got @fstr-spec Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 1 audited distributions.

Package purpose/API references: [pygithub 2.10.0 — maintainer-provided PyPI project description](https://pypi.org/project/pygithub/2.10.0/).

### 93. pg8000

**Service integrations.** Query PostgreSQL from a standalone reporting tool using a pure Python driver.

**Planned first acceptance test:** Use a disposable local database to run a parameterized query, round-trip values and test rollback, TLS and timeout behavior.

**Current compile evidence:** First compile stop in pg8000/converters.py: SyntaxError: invalid escape char Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 3 audited distributions.

Package purpose/API references: [pg8000 1.31.5 — maintainer-provided PyPI project description](https://pypi.org/project/pg8000/1.31.5/).

### 94. watchdog

**Operations & logging.** Watch a directory and trigger a local automation workflow.

**Planned first acceptance test:** After a supported backend exists, observe create, modify and rename events in a temporary directory and verify clean observer shutdown.

**Current compile evidence:** The pinned release has no selected generic Python 3 pure wheel; its native/platform artifact is outside the current installer contract.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 4 audited distributions.

Package purpose/API references: [watchdog 6.0.0 — maintainer-provided PyPI project description](https://pypi.org/project/watchdog/6.0.0/).

### 95. kubernetes

**Operations & logging.** Ship a focused cluster inventory or rollout inspection tool.

**Planned first acceptance test:** Load a fixture kubeconfig and list paginated pods against a local API server with TLS and error cases; no live cluster mutation.

**Current compile evidence:** First compile stop in kubernetes/aio/client/api_client.py: SyntaxError: expected newline after line continuation character Later files and runtime behavior may have additional blockers.

**Estimated effort: 5/5.** Requires a native-extension strategy, security-sensitive review, or a large framework/SDK dependency closure. Potential direct reach: 4 audited distributions.

Package purpose/API references: [kubernetes 36.0.3 — maintainer-provided PyPI project description](https://pypi.org/project/kubernetes/36.0.3/).

### 96. feedparser

**Validation & formats.** Build an offline-friendly RSS/Atom digest reader.

**Planned first acceptance test:** Parse local RSS and Atom fixtures with Unicode and malformed entries; verify normalized titles, links and parser error reporting.

**Current compile evidence:** First compile stop in feedparser/api.py: SyntaxError: expected an expression, got else Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 0 audited distributions.

Package purpose/API references: [feedparser 6.0.14 — maintainer-provided PyPI project description](https://pypi.org/project/feedparser/6.0.14/).

### 97. argcomplete

**CLI & terminal.** Offer shell completion for an argparse-based tool.

**Planned first acceptance test:** A controlled shell completion request returns expected subcommands and file candidates without running the app's command body.

**Current compile evidence:** First compile stop in argcomplete/completers.py: SyntaxError: expected '@id', got ',' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 6 audited distributions.

Package purpose/API references: [argcomplete 3.7.2 — maintainer-provided PyPI project description](https://pypi.org/project/argcomplete/3.7.2/).

### 98. pycountry

**Validation & formats.** Normalize country and language codes in imported business data.

**Planned first acceptance test:** Look up countries and languages by fixed ISO codes and verify packaged database resources after source and caches are removed.

**Current compile evidence:** First compile stop in pycountry/__init__.py: SyntaxError: expected '@id', got 'match' Later files and runtime behavior may have additional blockers.

**Estimated effort: 3/5.** Expect several language, standard-library or dependency gaps before the proposed slice can run. Potential direct reach: 1 audited distributions.

Package purpose/API references: [pycountry 26.2.16 — maintainer-provided PyPI project description](https://pypi.org/project/pycountry/26.2.16/).

### 99. pyotp

**Validation & formats.** Generate or verify one-time codes for an explicitly configured authentication workflow.

**Planned first acceptance test:** Run published HOTP/TOTP vectors with a fixed clock; verify invalid codes and boundaries and review the complete cryptographic dependency path before approval.

**Current compile evidence:** First compile stop in pyotp/__init__.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 1 audited distributions.

Package purpose/API references: [pyotp 2.10.0 — maintainer-provided PyPI project description](https://pypi.org/project/pyotp/2.10.0/).

### 100. questionary

**CLI & terminal.** Guide users through a setup wizard instead of requiring flags.

**Planned first acceptance test:** A pseudo-terminal chooses a list item, validates text and cancels a prompt with a predictable exit code.

**Current compile evidence:** First compile stop in questionary/form.py: SyntaxError: expected ')', got 'for' Later files and runtime behavior may have additional blockers.

**Estimated effort: 4/5.** Expect a substantial transport, process, terminal, async or document/dependency integration. Potential direct reach: 0 audited distributions.

Package purpose/API references: [questionary 2.1.1 — maintainer-provided PyPI project description](https://pypi.org/project/questionary/2.1.1/).

## What we deliberately left out

- Type stubs and typing-only support distributions: useful to editors, but they do not deliver an application workflow at runtime.
- Python build backends, installers, virtual-environment tools and release tooling: Kipferl already builds standalone apps; these are not the primary runtime product.
- Test runners, coverage tools and linters: essential developer tooling, but not first-wave functionality inside shipped end-user apps.
- Notebook frontends and widgets, documentation themes, empty compatibility/metapackages: import or compilation success would inflate a support count without a demonstrated application.
- GPU runtimes and large scientific/ML stacks: high downloads do not offset native ABI, distribution size and runtime requirements for this first CLI-focused shortlist.
- Duplicate SDK wrappers and broad orchestration frameworks: prioritize their underlying transport and a few representative service workflows first.

Selected shared libraries such as certifi, idna, wcwidth and MarkupSafe earn places through concrete app integration goals. packaging is scoped to application version constraints. Native-only PyYAML, MarkupSafe, psutil and watchdog remain expensive future work; listing them never implies a wheel can currently install.

## When a package can become usable

The proposed acceptance test is a first milestone, not a full-library certification. A usable compatibility badge additionally needs exact version/wheel/dependency pins, imports, declared scope, error cases and a detached bundled executable on each claimed OS/CPU. Security-sensitive APIs require negative tests and review. Live services, Git and database servers must be disclosed as external requirements.

The source audit is a compile-only screen. Its compilation-complete distributions, source-free wheels and behaviorally reviewed slices must remain separate. No roadmap score changes an installer allowlist, a compatibility verdict or a tested runtime hash.

## Reproducible inputs

- Machine-readable priorities, scores, direct dependents, sources and exact first diagnostics: [support-priorities.json](support-priorities.json).
- Baseline per-package evidence: [popularity-audit.json](popularity-audit.json).
- Original demand snapshot: [popularity.json](popularity.json).

Audit SHA-256: `b2de404bfe036bbc9c93af201a85348a59a60087d31117f484f01fb2bc8160e2`.
Runtime SHA-256: `1f54af5ee829e94d74e928c9317b12ed3be304c0aeab087dfd50283a5d3dbfbd` on `macos-aarch64`.
Popularity snapshot SHA-256: `75c29713bb37b11f719dd03fa7bbcd299bfeebb8bfa73018ca3c184a52fc253d`.

To update this roadmap, rejoin candidates to a fresh canonical audit, recompute demand and direct reach, and explicitly review usefulness, effort and acceptance goals. Keep priorities distinct from observed compatibility results.
