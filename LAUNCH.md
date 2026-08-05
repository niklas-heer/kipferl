# Kipferl Launch Plan

This document is a lightweight go-to-market plan focused on developer
adoption for CLI tools.

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

- CLI essentials: subcommands, help, completions.
- Robust subprocess (argv fidelity, large output handling).
- Config parsing: `configparser` plus `toml` or a tiny `fetch` module.
- Unicode width handling for aligned tables/boxes.

### Docs

- Quickstart (hello + build + run).
- Limitations (no pip) and module tiers.
- CLI cookbook (common patterns: config, logging, subprocess, progress).

### Examples

- "deploy" CLI with progress + subprocess.
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
