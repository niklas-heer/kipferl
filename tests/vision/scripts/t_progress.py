# Test tui.progress() and tui.spinner() functionality
import tui

# Basic progress bar
print("Progress bar tests:")
tui.progress(5, 10, label="Loading", width=20)
tui.progress_done()

# Progress with elapsed time
tui.progress(7, 10, label="Building", width=25, elapsed=3.5)
tui.progress_done()

# Full progress
tui.progress(10, 10, label="Complete", width=20)
tui.progress_done()

# Empty progress
tui.progress(0, 10, label="Starting", width=20)
tui.progress_done()

# Spinner test
print("\nSpinner tests:")
tui.spinner(0, "Frame 0")
tui.progress_done()
tui.spinner(3, "Frame 3")
tui.progress_done()

# Spinner frame function
print("\nSpinner frames:")
for i in range(10):
    frame = tui.spinner_frame(i)
    print(frame, end=" ")
print()
