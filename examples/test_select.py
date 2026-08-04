#!/usr/bin/env python3
"""Test script for select component - used by e2e tests."""

import input

result = input.select("Choose a color:", ["Red", "Green", "Blue"])
if result:
    print(f"SELECTED: {result}")
else:
    print("CANCELLED")
