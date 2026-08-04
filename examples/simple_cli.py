#!/usr/bin/env python3
"""A small command-line application built with μcharm."""

import sys
import time

import input
import tui


def cmd_greet(name=None):
    if not name:
        name = input.prompt("What is your name?", default="World")
    tui.box(
        f"Hello, {name}!\nWelcome to μcharm.",
        title="Greeting",
        border_color="cyan",
    )


def cmd_status():
    tui.info("Checking system status...")
    checks = [
        ("Database", True),
        ("Cache", True),
        ("API", True),
        ("Queue", False),
    ]
    for name, ok in checks:
        time.sleep(0.1)
        if ok:
            tui.success(name + ": Connected")
        else:
            tui.error(name + ": Disconnected")

    tui.table(
        [
            ["Metric", "Value"],
            ["Uptime", "3 days, 14 hours"],
            ["Memory", "245 MB / 512 MB"],
            ["CPU", "12%"],
        ],
        headers=True,
        border="rounded",
    )


def cmd_process(count=50):
    tui.info("Processing " + str(count) + " files...")
    for current in range(count + 1):
        tui.progress(current, count, label="Progress", color="green")
        time.sleep(0.01)
    tui.progress_done()
    tui.success("Processed " + str(count) + " files")


def show_help():
    tui.box(
        "Commands:\n  greet [name]    Greet someone\n  status          Show system status\n  process [n]     Process n files\n  help            Show this help",
        title="Simple CLI — Help",
        border_color="cyan",
    )


def interactive_mode():
    tui.box("Simple CLI Example\nBuilt with μcharm", title="Welcome")
    command = input.select(
        "What would you like to do?",
        ["Greet someone", "Check status", "Process files", "Exit"],
    )
    if command == "Greet someone":
        cmd_greet()
    elif command == "Check status":
        cmd_status()
    elif command == "Process files":
        cmd_process()
    else:
        tui.info("Goodbye!")


def main():
    args = sys.argv[1:]
    if not args:
        interactive_mode()
        return

    command = args[0]
    if command == "greet":
        cmd_greet(args[1] if len(args) > 1 else None)
    elif command == "status":
        cmd_status()
    elif command == "process":
        cmd_process(int(args[1]) if len(args) > 1 else 50)
    elif command in ("help", "--help", "-h"):
        show_help()
    else:
        tui.error("Unknown command: " + command)
        show_help()


if __name__ == "__main__":
    main()
