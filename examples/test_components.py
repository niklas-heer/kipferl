#!/usr/bin/env python3
"""Test script for UI components - used by e2e tests."""

import tui

# Test box
print("=== BOX TEST ===")
tui.box("Hello World", title="Test Box")

# Test rule
print("\n=== RULE TEST ===")
tui.rule("Section Title")

# Test status messages
print("\n=== STATUS TEST ===")
tui.success("This is success")
tui.error("This is error")
tui.warning("This is warning")
tui.info("This is info")

# Test styled text
print("\n=== STYLE TEST ===")
print(tui.style("Bold text", bold=True))
print(tui.style("Red text", fg="red"))
print(tui.style("Combined", fg="cyan", bold=True))

# Test table
print("\n=== TABLE TEST ===")
tui.table(
    [["Name", "Age"], ["Alice", "25"], ["Bob", "30"]], headers=True
)

# Test progress (static)
print("\n=== PROGRESS TEST ===")
tui.progress(50, 100, width=20, label="Loading")
print()  # newline after progress

print("\n=== ALL TESTS COMPLETE ===")
