#!/usr/bin/env python3
"""Test script for multiselect component - used by e2e tests."""

import input

result = input.multiselect(
    "Select toppings:", ["Cheese", "Pepperoni", "Mushrooms", "Olives"]
)
if result:
    print(f"SELECTED: {','.join(result)}")
else:
    print("NONE")
