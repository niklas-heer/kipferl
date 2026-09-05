# Real stable-release workflow. Run through scripts/record_demo.sh (mise run demo-gif).
# The wrapper provides an isolated workspace and checks every command's exit status.
Output "website/public/demos/kipferl-0.7.mp4"
Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 22
Set Width 1280
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 28
Set Framerate 24
Set TypingSpeed 35ms
Set CursorBlink false
Hide
Type `source "$DEMO_ROOT/environment"`
Enter
Sleep 150ms
Sleep 500ms
Show
Type "# 01 / Create a project"
Enter
Sleep 150ms
Type "kipferl --version"
Enter
Sleep 150ms
Sleep 1s
Type "kipferl new hello"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 3s
Ctrl+L
Type "cd hello"
Enter
Sleep 150ms
Type "kipferl run -- --name Ada"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 2s
Type "kipferl test"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 4s
Ctrl+L
Type "# 02 / Add a reviewed PyPI package"
Enter
Sleep 150ms
Type "kipferl add tzdata==2025.2"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 2s
Type "kipferl deps check"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 2s
Type "cp ../fixtures/zones.py ."
Enter
Sleep 150ms
Type "cat zones.py"
Enter
Sleep 150ms
Sleep 6s
Ctrl+L
Type "# 03 / Ship one executable"
Enter
Sleep 150ms
Type "kipferl build zones.py --mode universal -o ../dist/zones"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 5s
Ctrl+L
Type "cd ../dist"
Enter
Sleep 150ms
Type "rm -rf ../hello ../state"
Enter
Sleep 150ms
Type "unset TMPDIR KIPFERL_CACHE_DIR"
Enter
Sleep 150ms
Type "# Source, installed packages and caches are gone."
Enter
Sleep 150ms
Sleep 1s
Type "./zones"
Enter
Sleep 150ms
Wait+Line /^\$$/
Sleep 6s
Hide
Type `test ! -e ../hello && test ! -e ../state && touch "$DEMO_ROOT/complete"`
Enter
Sleep 150ms
Sleep 500ms
