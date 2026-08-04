#!/usr/bin/env python3
"""Test script for prompt component - used by e2e tests."""

import input

result = input.prompt("Enter your name:")
if result:
    print(f"NAME: {result}")
else:
    print("CANCELLED")
