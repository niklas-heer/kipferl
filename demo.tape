# Kipferl Demo
# Run with: vhs demo.tape

Output demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 720
Set Height 520
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 24
Set TypingSpeed 80ms

# Setup - add kipferl to PATH
Hide
Type "export PATH=$PWD/target/release:$PATH"
Enter
Type "clear"
Enter
Show
Sleep 500ms

# Show the code with syntax highlighting
Type "bat --style=plain --paging=never examples/quick_demo.py"
Enter
Sleep 5s

# Clear and run it
Type "clear"
Enter
Sleep 500ms

Type "kipferl run examples/quick_demo.py"
Enter
Sleep 2s

# Navigate select - take time so people can see
Down
Sleep 600ms
Enter
Sleep 1s

# Confirm
Type "y"
Enter
Sleep 2.5s

# Build it
Type "kipferl build examples/quick_demo.py -o myapp"
Enter
Sleep 3s

# Show size
Type "ls -lh myapp"
Enter
Sleep 3s

# Run built binary
Type "./myapp"
Enter
Sleep 1.5s
Enter
Sleep 800ms
Type "y"
Enter
Sleep 3s

# Cleanup
Hide
Type "rm -f myapp"
Enter
