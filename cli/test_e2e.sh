#!/bin/bash
# End-to-end tests for ucharm CLI
# Run from the cli/ directory: ./test_e2e.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

# Test temp directory
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

# Path to ucharm (absolute path)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MCHARM="$SCRIPT_DIR/zig-out/bin/ucharm"

echo "=== μcharm End-to-End Tests ==="
echo "Test directory: $TEST_DIR"
echo ""

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    echo "  Error: $2"
}

run_test() {
    TESTS_RUN=$((TESTS_RUN + 1))
}

# Check if ucharm is built
if [ ! -f "$MCHARM" ]; then
    echo -e "${YELLOW}Building ucharm...${NC}"
    zig build -Doptimize=ReleaseSmall
fi

# Check if micropython is available
if ! command -v micropython &> /dev/null; then
    echo -e "${RED}Error: micropython not found${NC}"
    echo "Install with: brew install micropython"
    exit 1
fi

echo "--- Test: Version ---"
run_test
# Read expected version from VERSION file
EXPECTED_VERSION=$(cat "$SCRIPT_DIR/src/VERSION" | tr -d '\n\r ')
if $MCHARM --version | grep -q "μcharm.*v$EXPECTED_VERSION"; then
    pass "Version output correct (v$EXPECTED_VERSION)"
else
    fail "Version output incorrect" "Expected 'μcharm v$EXPECTED_VERSION'"
fi

echo ""
echo "--- Test: Help ---"
run_test
if $MCHARM --help | grep -q "COMMANDS"; then
    pass "Help displays commands"
else
    fail "Help missing commands" "Expected 'COMMANDS' in output"
fi

echo ""
echo "--- Test: New Command ---"
run_test
cd "$TEST_DIR"
if $MCHARM new "Test App" 2>&1 | grep -q "Created"; then
    if [ -f "test_app/test_app.py" ]; then
        pass "New command creates test_app/test_app.py"
    else
        fail "New command file missing" "test_app/test_app.py not created"
    fi
else
    fail "New command failed" "No 'Created' in output"
fi

run_test
if grep -q "ucharm" test_app/test_app.py; then
    pass "Generated file contains ucharm reference"
else
    fail "Generated file incorrect" "Missing ucharm reference"
fi

run_test
if [ -x "test_app/test_app.py" ]; then
    pass "Generated file is executable"
else
    fail "Generated file not executable" "Missing execute permission"
fi

echo ""
echo "--- Test: New Command (Duplicate) ---"
run_test
if $MCHARM new "Test App" 2>&1 | grep -q "already exists"; then
    pass "New command detects existing directory"
else
    fail "New command should detect existing directory" "No error for duplicate"
fi

echo ""
echo "--- Test: New Command (Minimal) ---"
run_test
if $MCHARM new "minimal_app" --minimal 2>&1 | grep -q "Created"; then
    if [ -f "minimal_app.py" ]; then
        pass "New --minimal creates file in current dir"
    else
        fail "New --minimal file missing" "minimal_app.py not created"
    fi
else
    fail "New --minimal failed" "No 'Created' in output"
fi

echo ""
echo "--- Test: Build Command (Single Mode) ---"
cd "$OLDPWD"  # Back to cli directory
run_test

# Create a simple test script
cat > "$TEST_DIR/simple.py" << 'EOF'
#!/usr/bin/env micropython
import sys
sys.path.insert(0, ".")
from ucharm import success
success("Hello from simple!")
EOF

if $MCHARM build "$TEST_DIR/simple.py" -o "$TEST_DIR/simple_out.py" --mode single 2>&1 | grep -q "Transformed"; then
    pass "Build single mode creates output"
else
    fail "Build single mode failed" "No 'Transformed' in output"
fi

run_test
if [ -f "$TEST_DIR/simple_out.py" ]; then
    pass "Single mode output file exists"
else
    fail "Single mode output missing" "File not created"
fi

run_test
if grep -q "from tui import\|from input import" "$TEST_DIR/simple_out.py"; then
    pass "Single mode transforms imports"
else
    fail "Single mode doesn't transform imports" "Missing transformed imports"
fi

echo ""
echo "--- Test: Build Command (Executable Mode) ---"
run_test
if $MCHARM build "$TEST_DIR/simple.py" -o "$TEST_DIR/simple_exec" --mode executable 2>&1 | grep -q "Created"; then
    pass "Build executable mode creates output"
else
    fail "Build executable mode failed" "No 'Created' in output"
fi

run_test
if [ -x "$TEST_DIR/simple_exec" ]; then
    pass "Executable mode output is executable"
else
    fail "Executable mode not executable" "Missing execute permission"
fi

run_test
if head -1 "$TEST_DIR/simple_exec" | grep -q "#!/bin/bash"; then
    pass "Executable mode has bash shebang"
else
    fail "Executable mode wrong shebang" "Expected bash shebang"
fi

echo ""
echo "--- Test: Build Command (Universal Mode) ---"
run_test
if $MCHARM build "$TEST_DIR/simple.py" -o "$TEST_DIR/simple_universal" --mode universal 2>&1 | grep -q "universal binary"; then
    pass "Build universal mode creates output"
else
    fail "Build universal mode failed" "No 'universal binary' in output"
fi

run_test
if [ -x "$TEST_DIR/simple_universal" ]; then
    pass "Universal mode output is executable"
else
    fail "Universal mode not executable" "Missing execute permission"
fi

run_test
# Check file size is reasonable (should include micropython ~668KB + overhead)
SIZE=$(stat -f%z "$TEST_DIR/simple_universal" 2>/dev/null || stat -c%s "$TEST_DIR/simple_universal")
if [ "$SIZE" -gt 500000 ]; then
    pass "Universal binary size is reasonable (${SIZE} bytes)"
else
    fail "Universal binary too small" "Expected >500KB, got ${SIZE} bytes"
fi

echo ""
echo "--- Test: Build Command (Missing Script) ---"
run_test
if $MCHARM build nonexistent.py -o out 2>&1 | grep -q "not found"; then
    pass "Build detects missing script"
else
    fail "Build should detect missing script" "No error for missing file"
fi

echo ""
echo "--- Test: Build Command (Missing Output) ---"
run_test
if $MCHARM build "$TEST_DIR/simple.py" 2>&1 | grep -q "No output path"; then
    pass "Build requires -o flag"
else
    fail "Build should require -o flag" "No error for missing -o"
fi

echo ""
echo "--- Test: Unknown Command ---"
run_test
if $MCHARM unknown 2>&1 | grep -q "Unknown command"; then
    pass "Unknown command shows error"
else
    fail "Unknown command should show error" "No error for unknown command"
fi

echo ""
echo "--- Test: Run Command ---"
# Create a simple runnable script
cat > "$TEST_DIR/runme.py" << 'EOF'
print("Hello from runme!")
EOF

run_test
if $MCHARM run "$TEST_DIR/runme.py" 2>&1 | grep -q "Hello from runme"; then
    pass "Run command executes script"
else
    fail "Run command failed" "Script didn't execute"
fi

echo ""
echo "--- Test: UI Components ---"
run_test
cd "$SCRIPT_DIR/.."
if $MCHARM run examples/test_components.py 2>&1 | grep -q "ALL TESTS COMPLETE"; then
    pass "UI components render correctly"
else
    fail "UI components failed" "Components didn't render"
fi

echo ""
echo "--- Test: Interactive Select ---"
run_test
# Test select: down, down, enter -> should select "Blue"
if MCHARM_TEST_KEYS="down,down,enter" $MCHARM run examples/test_select.py 2>&1 | grep -q "SELECTED: Blue"; then
    pass "Select works"
else
    fail "Select failed" "Expected 'SELECTED: Blue'"
fi

echo ""
echo "--- Test: Interactive Confirm (yes) ---"
run_test
if MCHARM_TEST_KEYS="y" $MCHARM run examples/test_confirm.py 2>&1 | grep -q "CONFIRMED: yes"; then
    pass "Confirm 'y' works"
else
    fail "Confirm 'y' failed" "Expected 'CONFIRMED: yes'"
fi

echo ""
echo "--- Test: Interactive Confirm (no) ---"
run_test
if MCHARM_TEST_KEYS="n" $MCHARM run examples/test_confirm.py 2>&1 | grep -q "CONFIRMED: no"; then
    pass "Confirm 'n' works"
else
    fail "Confirm 'n' failed" "Expected 'CONFIRMED: no'"
fi

echo ""
echo "--- Test: Interactive Confirm (default) ---"
run_test
if MCHARM_TEST_KEYS="enter" $MCHARM run examples/test_confirm.py 2>&1 | grep -q "CONFIRMED: yes"; then
    pass "Confirm default (enter) works"
else
    fail "Confirm default failed" "Expected 'CONFIRMED: yes'"
fi

echo ""
echo "--- Test: Interactive Multiselect ---"
run_test
# Select first and third items: space (toggle), down, down, space (toggle), enter
if MCHARM_TEST_KEYS="space,down,down,space,enter" $MCHARM run examples/test_multiselect.py 2>&1 | grep -q "SELECTED: Cheese,Mushrooms"; then
    pass "Multiselect works"
else
    fail "Multiselect failed" "Expected 'SELECTED: Cheese,Mushrooms'"
fi

echo ""
echo "--- Test: Interactive Prompt ---"
run_test
# Type "Alice" then enter using MCHARM_TEST_KEYS (comma-separated chars)
if MCHARM_TEST_KEYS="A,l,i,c,e,enter" $MCHARM run examples/test_prompt.py 2>&1 | grep -q "NAME: Alice"; then
    pass "Prompt text input works"
else
    fail "Prompt failed" "Expected 'NAME: Alice'"
fi

echo ""
echo "--- Test: Universal Binary Interactive ---"
run_test
# Build and test interactive select in universal binary
# Note: MicroPython doesn't support fd 3, so use env var for universal binaries
# Clear any cached loaders to ensure fresh extraction
rm -rf /tmp/ucharm-* 2>/dev/null
$MCHARM build examples/test_select.py -o "$TEST_DIR/select_universal" --mode universal >/dev/null 2>&1
if MCHARM_TEST_KEYS="down,enter" "$TEST_DIR/select_universal" 2>&1 | grep -q "SELECTED: Green"; then
    pass "Universal binary interactive works"
else
    fail "Universal binary interactive failed" "Expected 'SELECTED: Green'"
fi

cd "$SCRIPT_DIR"

echo ""
echo "=== Test Summary ==="
echo -e "Tests run: $TESTS_RUN"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$((TESTS_RUN - TESTS_PASSED))${NC}"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed!${NC}"
    exit 1
fi
