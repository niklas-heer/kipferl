#!/usr/bin/env python3
"""
Interactive release script for kipferl - built with kipferl!
Creates a new version tag and pushes it to trigger the release workflow.

Run with: kipferl run scripts/release.py
"""

import subprocess
import sys

# Add kipferl library to path (works for both PocketPy and CPython)
# When run from repo root, this adds the kipferl package
sys.path.insert(0, ".")
sys.path.insert(0, "..")

from kipferl import box, confirm, error, info, rule, select, style, success, warning


def run(cmd, capture=True):
    """Run a shell command using native subprocess."""
    # getstatusoutput runs via shell and returns (status, output)
    code, output = subprocess.getstatusoutput(cmd)
    if capture:
        return output, code
    else:
        return "", code


def get_current_version():
    """Get current version from git tags."""
    output, code = run("git describe --tags --abbrev=0 2>/dev/null")
    if code != 0 or not output:
        return "0.0.0"
    return output.lstrip("v")


def parse_version(version):
    """Parse a stable or release-candidate version into components."""
    version_parts = version.split("-")
    if len(version_parts) > 2:
        raise ValueError("version contains more than one prerelease separator")
    parts = version_parts[0].split(".")
    if len(parts) != 3:
        raise ValueError("version must contain major, minor, and patch components")
    prerelease = version_parts[1] if len(version_parts) == 2 else None
    return int(parts[0]), int(parts[1]), int(parts[2]), prerelease


def bump_version(version, bump_type):
    """Bump version based on type."""
    major, minor, patch, _ = parse_version(version)

    if bump_type == "major":
        return f"{major + 1}.0.0"
    elif bump_type == "minor":
        return f"{major}.{minor + 1}.0"
    else:  # patch
        return f"{major}.{minor}.{patch + 1}"


def next_release_candidate(version):
    """Return the next RC, starting a minor RC series from a stable tag."""
    major, minor, patch, prerelease = parse_version(version)
    if prerelease is None:
        return f"{major}.{minor + 1}.0-rc.1"

    prerelease_parts = prerelease.split(".")
    valid_rc = len(prerelease_parts) == 2
    valid_rc = valid_rc and prerelease_parts[0] == "rc"
    valid_rc = valid_rc and prerelease_parts[1].isdigit()
    if not valid_rc:
        raise ValueError("only rc.N prerelease versions are supported")
    return f"{major}.{minor}.{patch}-rc.{int(prerelease_parts[1]) + 1}"


def final_version(version):
    """Remove an RC suffix without changing the release number."""
    major, minor, patch, _ = parse_version(version)
    return f"{major}.{minor}.{patch}"


def workspace_manifest_with_version(contents, new_version):
    """Update only the workspace package version in Cargo.toml."""
    marker = '[workspace.package]\nversion = "'
    start = contents.find(marker)
    if start == -1:
        raise ValueError("Cargo.toml is missing the workspace package version")
    value_start = start + len(marker)
    relative_value_end = contents[value_start:].find('"')
    if relative_value_end == -1:
        raise ValueError("Cargo.toml has an unterminated workspace package version")
    value_end = value_start + relative_value_end
    return contents[:value_start] + new_version + contents[value_end:]


def update_version_files(new_version):
    """Keep the public and Cargo workspace versions synchronized."""
    with open("VERSION", "w") as version_file:
        version_file.write(new_version + "\n")

    with open("Cargo.toml", "r") as manifest_file:
        manifest = manifest_file.read()
    updated_manifest = workspace_manifest_with_version(manifest, new_version)
    with open("Cargo.toml", "w") as manifest_file:
        manifest_file.write(updated_manifest)


def has_uncommitted_changes():
    """Check for uncommitted changes."""
    _, code = run("git diff-index --quiet HEAD --")
    return code != 0


def get_recent_commits(since_tag):
    """Get commits since last tag."""
    if since_tag == "0.0.0":
        cmd = "git log --oneline -10"
    else:
        cmd = f"git log v{since_tag}..HEAD --oneline"
    output, _ = run(cmd)
    return output.split("\n") if output else []


def main():
    # Header
    print()
    box(
        style("Release Manager", bold=True),
        title="kipferl",
        border_color="cyan",
        padding=1,
    )
    print()

    # Check for uncommitted changes
    if has_uncommitted_changes():
        error("You have uncommitted changes")
        print(style("  Please commit or stash them before releasing.", fg="gray"))
        print()
        sys.exit(1)

    # Get current version
    current = get_current_version()
    version_styled = style("v" + current, fg="green", bold=True)
    info("Current version: " + version_styled)
    print()

    # Show recent commits
    commits = get_recent_commits(current)
    if commits and commits[0]:
        print(style("  Recent commits:", fg="gray"))
        for commit in commits[:5]:
            print(style(f"    {commit}", fg="gray"))
        if len(commits) > 5:
            print(style(f"    ... and {len(commits) - 5} more", fg="gray"))
        print()

    # Calculate next versions
    next_patch = bump_version(current, "patch")
    next_minor = bump_version(current, "minor")
    next_major = bump_version(current, "major")
    next_rc = next_release_candidate(current)
    _, _, _, current_prerelease = parse_version(current)

    # Version selection
    if current_prerelease is None:
        choices = [
            f"Release candidate  v{next_rc}  {style('(public bake)', fg='gray')}",
            f"Patch  v{next_patch}  {style('(bug fixes)', fg='gray')}",
            f"Minor  v{next_minor}  {style('(new features)', fg='gray')}",
            f"Major  v{next_major}  {style('(breaking changes)', fg='gray')}",
        ]
    else:
        stable_version = final_version(current)
        choices = [
            f"Release candidate  v{next_rc}  {style('(continue bake)', fg='gray')}",
            f"Final  v{stable_version}  {style('(promote this RC)', fg='gray')}",
        ]

    choice = select("Select version bump:", choices)

    if choice is None:
        print()
        warning("Release cancelled")
        sys.exit(0)

    # Map choice to version
    if "Release candidate" in choice:
        new_version = next_rc
    elif "Final" in choice:
        new_version = final_version(current)
    elif "Patch" in choice:
        new_version = next_patch
    elif "Minor" in choice:
        new_version = next_minor
    else:
        new_version = next_major

    print()
    rule()
    print()

    # Confirm
    release_styled = style("v" + new_version, fg="cyan", bold=True)
    print("  Will create release: " + release_styled)
    print()

    if not confirm("Continue?"):
        print()
        warning("Release cancelled")
        sys.exit(0)

    print()

    # Keep VERSION, Cargo.toml, and Cargo.lock synchronized.
    info("Updating version files...")
    try:
        update_version_files(new_version)
        _, code = run("cargo check --workspace")
        if code != 0:
            raise RuntimeError("cargo check failed while refreshing Cargo.lock")
        success("Updated VERSION, Cargo.toml, and Cargo.lock")
    except Exception as e:
        error(f"Failed to update version files: {e}")
        run("git restore -- VERSION Cargo.toml Cargo.lock")
        sys.exit(1)

    # Commit a version bump when release preparation has not already done so.
    _, diff_code = run("git diff --quiet -- VERSION Cargo.toml Cargo.lock")
    version_commit_created = diff_code != 0
    if version_commit_created:
        info("Committing version bump...")
        _, code = run(
            f'git add VERSION Cargo.toml Cargo.lock && git commit -m "chore: bump version to {new_version}"'
        )
        if code != 0:
            error("Failed to commit version bump")
            run("git restore -- VERSION Cargo.toml Cargo.lock")
            sys.exit(1)
        success("Committed version bump")
    else:
        success(f"Version files are already prepared for {new_version}")

    # Create tag
    info("Creating tag...")
    _, code = run(f'git tag -a "v{new_version}" -m "Release v{new_version}"')
    if code != 0:
        error(f"Failed to create tag v{new_version}")
        if version_commit_created:
            run("git reset --soft HEAD~1")
        sys.exit(1)
    success(f"Created tag v{new_version}")

    # Push commit and tag
    info("Pushing to origin...")
    _, code = run("git push origin main", capture=False)
    if code != 0:
        error("Failed to push commit")
        # Cleanup
        run(f'git tag -d "v{new_version}"')
        if version_commit_created:
            run("git reset --soft HEAD~1")
        sys.exit(1)

    _, code = run(f'git push origin "v{new_version}"', capture=False)
    if code != 0:
        error("Failed to push tag")
        # Cleanup
        run(f'git tag -d "v{new_version}"')
        sys.exit(1)
    success("Pushed to origin")

    print()
    rule()
    print()

    # Success message - build string without multiline f-string (PocketPy limitation)
    msg = style("Release v" + new_version + " initiated!", bold=True) + "\n\n"
    msg = msg + "The workflow will now:\n"
    msg = msg + "  " + style("1.", fg="cyan") + " Build binaries for all platforms\n"
    msg = msg + "  " + style("2.", fg="cyan") + " Publish curated or generated release notes\n"
    msg = msg + "  " + style("3.", fg="cyan") + " Create GitHub release with assets\n"
    msg = msg + "  " + style("4.", fg="cyan") + " Update Homebrew formula"
    box(msg, border_color="green", padding=1)
    print()

    url = "https://github.com/niklas-heer/kipferl/actions"
    print(f"  Watch progress: {style(url, fg='cyan', underline=True)}")
    print()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print()
        warning("Cancelled")
        sys.exit(0)
