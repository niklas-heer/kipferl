#!/usr/bin/env python3
"""μcharm demo showcasing the current native tui and input modules."""

import time

import input
import tui


def main():
    print()
    tui.box(
        "μcharm v0.6.0-rc.1\nBeautiful CLIs with PocketPy and Rust\nFast startup | Standalone binaries | Python syntax",
        title="Welcome",
        border_color="cyan",
    )
    print()

    tui.rule("Styling", color="magenta")
    print()
    styles = "  "
    styles += tui.style("Bold", bold=True) + "  "
    styles += tui.style("Italic", italic=True) + "  "
    styles += tui.style("Underline", underline=True)
    print(styles)

    colors = "  "
    colors += tui.style("Red", fg="red") + "  "
    colors += tui.style("Green", fg="green") + "  "
    colors += tui.style("Cyan", fg="cyan") + "  "
    colors += tui.style("RGB", fg="#FF6B6B", bold=True)
    print(colors)
    print()

    tui.rule("Status Messages", color="magenta")
    tui.success("Operation completed successfully")
    tui.info("Here is some useful information")
    tui.warning("This might need your attention")
    tui.error("Something went wrong")
    print()

    tui.rule("Progress", color="magenta")
    for current in range(0, 101, 5):
        tui.progress(current, 100, label="Downloading", color="green")
        time.sleep(0.015)
    tui.progress_done()
    print()

    tui.rule("Measured Release", color="magenta")
    tui.table(
        [
            ["Metric", "v0.6.0-rc.1"],
            ["Compatibility", "1,669 / 1,669"],
            ["Median startup", "7.044 ms"],
            ["ARM64 minimal app", "4.3 MB"],
        ],
        headers=True,
        border="rounded",
        border_color="cyan",
    )
    print()

    if input.confirm("Run the interactive demo?", default=True):
        language = input.select(
            "What is your favorite language?",
            ["Python", "Rust", "Go", "JavaScript", "Other"],
        )
        name = input.prompt("What is your name?", default="Developer")
        tui.box(
        f"Hello, {name}!\nGreat choice picking {language}.",
            title="Summary",
            border_color="green",
        )

    tui.success("Demo complete!")


if __name__ == "__main__":
    main()
