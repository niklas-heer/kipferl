#!/usr/bin/env python3
"""
Generate AI-powered release notes using OpenRouter.

This script analyzes git commits and uses AI to generate polished,
user-friendly release notes for GitHub releases.
"""

import os
import subprocess
import sys
from typing import Optional


def get_commits(prev_tag: Optional[str], current_tag: str) -> str:
    """Fetch commit messages with their full bodies between two tags."""
    if prev_tag:
        range_spec = f"{prev_tag}..{current_tag}"
    else:
        range_spec = current_tag

    cmd = ["git", "log", "--pretty=format:%h|%s|%b|||", range_spec]

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return result.stdout


def parse_commits(raw_commits: str) -> list[dict]:
    """Parse raw git log output into structured commit data."""
    commits = []

    for commit_block in raw_commits.split("|||"):
        commit_block = commit_block.strip()
        if not commit_block:
            continue

        parts = commit_block.split("|", 2)
        if len(parts) >= 2:
            commit_hash = parts[0].strip()
            subject = parts[1].strip()
            body = parts[2].strip() if len(parts) > 2 else ""

            if subject.startswith("Merge"):
                continue

            commits.append({"hash": commit_hash, "subject": subject, "body": body})

    return commits


def installation_section(current_tag: str, repo: str) -> str:
    """Return deterministic installation instructions for a release."""
    if "-" in current_tag:
        return f"""### Installation

This is a prerelease. Download the matching CLI and adjacent `.sha256` file
from the [{current_tag} GitHub release](https://github.com/{repo}/releases/tag/{current_tag}):

- `kipferl-macos-aarch64` — macOS Apple Silicon
- `kipferl-macos-x86_64` — macOS Intel
- `kipferl-linux-aarch64` — Linux ARM64, static musl
- `kipferl-linux-x86_64` — Linux x86_64, static musl"""
    return """### Installation

```bash
brew install niklas-heer/tap/kipferl
brew upgrade kipferl
```"""


def assemble_release_notes(
    generated: str,
    current_tag: str,
    prev_tag: Optional[str],
    repo: str,
) -> str:
    """Add factual installation and changelog sections outside the AI output."""
    sections = [generated.strip(), installation_section(current_tag, repo)]
    if prev_tag:
        sections.append(
            f"---\n\n**Full changelog:** "
            f"https://github.com/{repo}/compare/{prev_tag}...{current_tag}"
        )
    return "\n\n".join(sections)


def validate_generated_summary(generated: str) -> None:
    """Reject AI-owned metadata so the deterministic fallback takes over."""
    normalized = generated.casefold()
    forbidden = (
        "http://",
        "https://",
        "### installation",
        "## installation",
        "brew install",
        "brew upgrade",
        "kipferl-macos-",
        "kipferl-linux-",
        "full changelog",
    )
    if any(value in normalized for value in forbidden):
        raise ValueError("generated summary contains release metadata")


def generate_release_notes_with_ai(
    commits: list[dict],
    current_tag: str,
    prev_tag: Optional[str],
    repo: str,
    api_key: str,
    model: str = "anthropic/claude-haiku-4.5",
) -> str:
    """Generate release notes using OpenRouter AI."""
    import requests

    commits_context = []
    for commit in commits:
        commit_text = f"**{commit['subject']}** ({commit['hash']})"
        if commit["body"]:
            commit_text += f"\n{commit['body']}"
        commits_context.append(commit_text)

    commits_text = "\n\n".join(commits_context)

    prompt = f"""You are writing release notes for "kipferl" (Kipferl), a CLI toolkit for building beautiful, fast command-line applications with Python syntax. The production CLI, universal loader, native modules, and PocketPy host are implemented in Rust. PocketPy itself is vendored C. Supported release targets are macOS ARM64, macOS x86_64, Linux ARM64 musl, and Linux x86_64 musl. Profile-based tree shaking chooses a 1.13-1.35 MB core runtime or a 4.50-5.45 MB full runtime without requiring a Rust toolchain. The measured minimal Apple Silicon app is 1.451 MB and starts in 7.679 ms median.

# Commits:

{commits_text}

# Task:

Generate polished, engaging release notes in markdown. Follow the style of polished developer tools like Bun and Deno.

## Structure:

1. **Opening** (1-2 lines max):
   - Start with a short, friendly tagline that captures the release theme
   - Can be playful but not forced (e.g., "This release brings interactive prompts to your CLI apps" or "Faster builds, smaller binaries")

2. **Sections** (use emoji prefixes, only include sections with content):
   - ✨ **What's New** — New features and capabilities
   - ⚡ **Improvements** — Performance gains, enhancements
   - 🐛 **Bug Fixes** — Corrected issues
   - 📚 **Documentation** — Docs and examples (only if significant)

## Style Guidelines:

- **Tone**: Friendly and approachable, like talking to a fellow developer
- **Bullets**: Use `-` with concise descriptions (one line each, ~10-20 words max)
- **Emphasis**: Use `**bold**` for module names, commands, and key terms
- **Metrics**: Include performance numbers when available (e.g., "6.6x faster than CPython")
- **Accuracy**: Do not invent targets, metrics, compatibility counts, size claims, or installation methods. Never claim Windows support. Only use numbers present in the commits or the product facts above.
- **No commit hashes** in the output
- **Present tense**: "Add" not "Added"
- **Scope**: Do not write installation commands, download links, asset names, or a full changelog link. The generator appends those facts deterministically.

## Example Output:

Interactive prompts have arrived! Build beautiful CLI experiences with select menus, confirmations, and more.

### ✨ What's New

- Add **input** module with `select()`, `confirm()`, `prompt()`, and `password()`
- Add **tui.spinner_frame()** for animated loading indicators
- New `kipferl init --ai` command generates AI assistant instructions

### ⚡ Improvements

- **signal** module now 6.6x faster than CPython equivalent
- Universal binaries start 30% faster on macOS with improved caching

### 🐛 Bug Fixes

- Fix box rendering when content contains ANSI color codes
- Correct cursor positioning after multiselect prompts

Generate the release notes now, starting with the opening tagline."""

    response = requests.post(
        "https://openrouter.ai/api/v1/chat/completions",
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": f"https://github.com/{repo}",
            "X-Title": "kipferl Release Notes Generator",
        },
        json={
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.6,
            "max_tokens": 1500,
        },
        timeout=60,
    )

    response.raise_for_status()
    result = response.json()

    generated = result["choices"][0]["message"]["content"].strip()
    validate_generated_summary(generated)
    return assemble_release_notes(generated, current_tag, prev_tag, repo)


def main():
    """Main entry point."""
    current_tag = os.environ.get("CURRENT_TAG")
    prev_tag = os.environ.get("PREV_TAG", "").strip()
    repo = os.environ.get("GITHUB_REPOSITORY")
    api_key = os.environ.get("OPENROUTER_API_KEY")
    model = os.environ.get("AI_MODEL", "anthropic/claude-haiku-4.5")

    if not current_tag:
        print("Error: CURRENT_TAG environment variable is required", file=sys.stderr)
        sys.exit(1)

    if not repo:
        print(
            "Error: GITHUB_REPOSITORY environment variable is required", file=sys.stderr
        )
        sys.exit(1)

    if not api_key:
        print(
            "Error: OPENROUTER_API_KEY environment variable is required",
            file=sys.stderr,
        )
        sys.exit(1)

    prev_tag = prev_tag if prev_tag else None

    print(f"Generating release notes for {current_tag}", file=sys.stderr)
    if prev_tag:
        print(f"Previous tag: {prev_tag}", file=sys.stderr)
    else:
        print("First release (no previous tag)", file=sys.stderr)

    raw_commits = get_commits(prev_tag, current_tag)
    commits = parse_commits(raw_commits)

    print(f"Found {len(commits)} commits to analyze", file=sys.stderr)

    if not commits:
        print("No commits found. Generating minimal release notes.", file=sys.stderr)
        release_notes = assemble_release_notes(
            f"Release {current_tag}", current_tag, prev_tag, repo
        )
    else:
        try:
            release_notes = generate_release_notes_with_ai(
                commits=commits,
                current_tag=current_tag,
                prev_tag=prev_tag,
                repo=repo,
                api_key=api_key,
                model=model,
            )
        except Exception as e:
            print(f"Error calling OpenRouter API: {e}", file=sys.stderr)
            print("Falling back to basic release notes", file=sys.stderr)

            changes = "## Changes\n\n" + "\n".join(
                f"- {commit['subject']}" for commit in commits
            )
            release_notes = assemble_release_notes(
                changes, current_tag, prev_tag, repo
            )

    print(release_notes)


if __name__ == "__main__":
    main()
