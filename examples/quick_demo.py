#!/usr/bin/env python3
"""Quick demo for README GIF"""

import input
import tui

# Welcome box
tui.box("Welcome to Kipferl!", title="Hello", border_color="cyan")
print()

# Interactive select
choice = input.select(
    "What would you like to do?",
    ["Create a new project", "Run tests", "Deploy to production"],
)

if choice:
    print()
    if input.confirm("Are you sure?"):
        print()
        tui.success(f"Selected: {tui.style(choice, bold=True)}")
