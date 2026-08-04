#!/bin/bash
# Check if a newer version of PocketPy is available
# Usage: ./scripts/check-pocketpy-version.sh [--update]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION_FILE="$PROJECT_ROOT/pocketpy/POCKETPY_VERSION"
VENDOR_DIR="$PROJECT_ROOT/pocketpy/vendor"

# Read current version
if [ -f "$VERSION_FILE" ]; then
    CURRENT_VERSION=$(cat "$VERSION_FILE" | tr -d '[:space:]')
else
    # Try to extract from header
    CURRENT_VERSION=$(grep '#define PK_VERSION' "$VENDOR_DIR/pocketpy.h" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

echo "Current PocketPy version: $CURRENT_VERSION"

# Fetch latest release from GitHub API
LATEST_RELEASE=$(curl -s https://api.github.com/repos/pocketpy/pocketpy/releases/latest)
LATEST_VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/')

echo "Latest PocketPy version: $LATEST_VERSION"

if [ "$CURRENT_VERSION" = "$LATEST_VERSION" ]; then
    echo "Already on the latest version."
    exit 0
else
    echo "New version available: $LATEST_VERSION (current: $CURRENT_VERSION)"

	    if [ "$1" = "--update" ]; then
	        echo "Downloading PocketPy v$LATEST_VERSION..."

	        curl -sL "https://github.com/pocketpy/pocketpy/releases/download/v$LATEST_VERSION/pocketpy.c" -o "$VENDOR_DIR/pocketpy.c"
	        curl -sL "https://github.com/pocketpy/pocketpy/releases/download/v$LATEST_VERSION/pocketpy.h" -o "$VENDOR_DIR/pocketpy.h"

	        echo "$LATEST_VERSION" > "$VERSION_FILE"

	        echo "Applying μcharm PocketPy patchset..."
	        "$PROJECT_ROOT/scripts/apply-pocketpy-patches.sh"

	        echo "Updated to v$LATEST_VERSION"
	        echo ""
	        echo "Rebuild and test:"
	        echo "  cargo build --release -p ucharm-runtime"
	        echo "  python3 tests/compat_runner.py --runtime target/release/pocketpy-ucharm --report"
	    else
	        echo "Run with --update to download the new version."
	        exit 1
    fi
fi
