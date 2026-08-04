# Test tui.table() functionality
import tui

# Basic table without headers
print("Basic table:")
data = [["Alice", "25", "Engineer"], ["Bob", "30", "Designer"]]
tui.table(data)

# Table with headers
print("\nWith headers:")
data = [["Name", "Age", "Role"], ["Alice", "25", "Engineer"], ["Bob", "30", "Designer"]]
tui.table(data, headers=True)

# Different border styles
print("\nRounded border:")
tui.table([["A", "B"], ["1", "2"]], border="rounded")

print("\nDouble border:")
tui.table([["X", "Y"], ["3", "4"]], border="double")

print("\nHeavy border:")
tui.table([["P", "Q"], ["5", "6"]], border="heavy")

# Single row with headers
print("\nSingle data row:")
tui.table([["Status", "Count"], ["OK", "42"]], headers=True)
