#!/bin/bash
#
# Update Homebrew formula with new version and checksums
# Called by the release workflow after binaries are published
#

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <stable-version>" >&2
    exit 1
fi
VERSION=$1
if [[ ! "$VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo "Homebrew updates require a stable version such as 0.7.0" >&2
    exit 1
fi

TAG="v$VERSION"
REPO="niklas-heer/kipferl"
TAP_REPO="${TAP_REPO:-../homebrew-tap}"
BASE_URL="https://github.com/$REPO/releases/download/$TAG"
DOWNLOAD_DIRECTORY=$(mktemp -d)
FORMULA_TEMPORARY=""
cleanup() {
    rm -rf "$DOWNLOAD_DIRECTORY"
    if [[ -n "$FORMULA_TEMPORARY" ]]; then
        rm -f "$FORMULA_TEMPORARY"
    fi
}
trap cleanup EXIT

echo "Updating Homebrew formula to version $VERSION..."

# Command substitutions do not reliably inherit errexit in Bash. Explicitly
# propagate every download/read failure before returning a verified hash.
verified_sha256() {
    local asset=$1
    local binary="$DOWNLOAD_DIRECTORY/$asset"
    local sidecar="$binary.sha256"
    local actual published hash
    curl -fsSL --connect-timeout 30 --max-time 120 \
        --output "$binary" "$BASE_URL/$asset" || return 1
    curl -fsSL --connect-timeout 30 --max-time 120 \
        --output "$sidecar" "$BASE_URL/$asset.sha256" || return 1
    if [[ ! -s "$binary" ]]; then
        echo "Release binary is empty: $asset" >&2
        return 1
    fi
    actual=$(shasum -a 256 "$binary") || return 1
    hash=${actual%% *}
    published=$(cat "$sidecar") || return 1
    if [[ ! "$hash" =~ ^[0-9a-f]{64}$ || "$published" != "$hash  $asset" ]]; then
        echo "Release checksum mismatch or invalid sidecar: $asset" >&2
        return 1
    fi
    printf '%s\n' "$hash"
}

echo "Downloading and verifying release checksums..."
SHA_MACOS_ARM64=$(verified_sha256 "kipferl-macos-aarch64")
SHA_MACOS_X86=$(verified_sha256 "kipferl-macos-x86_64")
SHA_LINUX_X86=$(verified_sha256 "kipferl-linux-x86_64")
SHA_LINUX_ARM64=$(verified_sha256 "kipferl-linux-aarch64")

echo "  macOS ARM64: $SHA_MACOS_ARM64"
echo "  macOS x86:   $SHA_MACOS_X86"
echo "  Linux x86:   $SHA_LINUX_X86"
echo "  Linux ARM64: $SHA_LINUX_ARM64"

# Stage beside the destination so the final rename is atomic. No existing
# formula is touched unless every published binary and checksum agrees.
FORMULA_PATH="$TAP_REPO/Formula/kipferl.rb"
mkdir -p "$(dirname "$FORMULA_PATH")"
FORMULA_TEMPORARY=$(mktemp "$TAP_REPO/Formula/.kipferl.rb.XXXXXX")

cat > "$FORMULA_TEMPORARY" << EOF
class Kipferl < Formula
  desc "Bake Python CLI apps into fast standalone binaries"
  homepage "https://kipferl.dev"
  version "$VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/niklas-heer/kipferl/releases/download/v#{version}/kipferl-macos-aarch64"
      sha256 "$SHA_MACOS_ARM64"
    else
      url "https://github.com/niklas-heer/kipferl/releases/download/v#{version}/kipferl-macos-x86_64"
      sha256 "$SHA_MACOS_X86"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/niklas-heer/kipferl/releases/download/v#{version}/kipferl-linux-aarch64"
      sha256 "$SHA_LINUX_ARM64"
    else
      url "https://github.com/niklas-heer/kipferl/releases/download/v#{version}/kipferl-linux-x86_64"
      sha256 "$SHA_LINUX_X86"
    end
  end

  def install
    binary_name = if OS.mac?
      Hardware::CPU.arm? ? "kipferl-macos-aarch64" : "kipferl-macos-x86_64"
    else
      Hardware::CPU.arm? ? "kipferl-linux-aarch64" : "kipferl-linux-x86_64"
    end

    bin.install binary_name => "kipferl"
    bin.install_symlink "kipferl" => "ucharm"
  end

  test do
    assert_match "Kipferl", shell_output("#{bin}/kipferl --version")
    assert_match "renamed to", shell_output("#{bin}/ucharm --version 2>&1")
  end
end
EOF

chmod 644 "$FORMULA_TEMPORARY"
mv -f "$FORMULA_TEMPORARY" "$FORMULA_PATH"
FORMULA_TEMPORARY=""
echo "Formula written to: $FORMULA_PATH"
