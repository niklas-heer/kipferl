#!/usr/bin/env python3
"""Test script for confirm component - used by e2e tests."""

import input

result = input.confirm("Do you agree?")
if result is True:
    print("CONFIRMED: yes")
elif result is False:
    print("CONFIRMED: no")
else:
    print("CANCELLED")
