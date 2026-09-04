# Kipferl Launch Plan

This document is a lightweight go-to-market plan focused on developer
adoption for CLI tools. Stable v0.6.0 is published. The project workflow and
safety improvements listed as implemented below are on main and still need a
new release with rebuilt components before promotion to stable users.

## Positioning

- Python ergonomics with Go-style shipping.
- Tiny, fast, single-file binaries.
- Beautiful terminal UX out of the box.
- Clear compatibility boundaries (not a pip replacement).

## Target Users

- Internal tooling teams (DevOps, Platform, SRE).
- OSS CLI authors who want Python DX without Python packaging.
- Teams that ship binaries into CI/CD, containers, or air-gapped environments.

## v1 Readiness Checklist

### Product

- Implemented on main: subcommands, help, Bash/Zsh/Fish completions,
  cli/api/interactive templates, and project test/configuration workflows.
- Implemented on main: argument fidelity, bounded captured subprocess output,
  signal exit statuses, and source-aware errors.
- Available configuration APIs: JSON, YAML, TOML, KDL, XML, CSV, and INI/CFG;
  use `http.client` for networking. There is no `fetch` module.
- Remaining: terminal-cell-aware Unicode widths for presentation APIs that
  intentionally retain the historical byte-width behavior.
- Fast local feedback through `kipferl dev` watch/restart mode.

### Docs

- Updated quickstart: distinguish stable downloads from the unreleased project
  workflow, then create, run, develop, test, build, and verify an isolated app.
- Limitations (no pip) and module tiers.
- Four executable recipes cover CSV summaries, HTTP clients, repository
  inspection, and report generation; CI checks source snippets and behavior.

### Examples

- Four implemented recipes: `examples/recipes/`, documented on the website.
- Future showcase: "deploy" CLI with progress + subprocess.
- "log viewer" CLI with filtering + table output.
- "scaffold" CLI with prompts + config.

### Distribution

- Homebrew formula.
- GitHub Releases with platform binaries.
- One-liner install and "verify" steps.

## Launch Assets

- 60-90s demo video (build -> run -> binary size -> startup time).
- Comparison table vs Python+Rich, Go+Cobra, Rust+Ratatui.
- "Built with Kipferl" gallery section.

## Channels

- HN / r/commandline / r/devops
- Terminal UI community channels (Bubble Tea / Lip Gloss audiences)
- GitHub discussions and showcases

## Success Metrics

- Time to first successful `kipferl build` < 5 minutes.
- > 30% of visitors complete the Quickstart.
- 3-5 showcase apps in the first month.
- Sustained weekly downloads from releases.
