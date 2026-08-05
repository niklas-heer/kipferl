#!/bin/bash
#
# Update Homebrew formula with new version and checksums
# Called by the release workflow after binaries are published
#

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

TAG="v$VERSION"
REPO="niklas-heer/kipferl"
TAP_REPO="${TAP_REPO:-../homebrew-tap}"

echo "Updating Homebrew formula to version $VERSION..."

# Download binaries and calculate checksums
calc_sha256() {
    local url=$1
    curl -fsSL "$url" | shasum -a 256 | cut -d' ' -f1
}

BASE_URL="https://github.com/$REPO/releases/download/$TAG"

echo "Calculating checksums..."
SHA_MACOS_ARM64=$(calc_sha256 "$BASE_URL/kipferl-macos-aarch64")
SHA_MACOS_X86=$(calc_sha256 "$BASE_URL/kipferl-macos-x86_64")
SHA_LINUX_X86=$(calc_sha256 "$BASE_URL/kipferl-linux-x86_64")
SHA_LINUX_ARM64=$(calc_sha256 "$BASE_URL/kipferl-linux-aarch64")

echo "  macOS ARM64: $SHA_MACOS_ARM64"
echo "  macOS x86:   $SHA_MACOS_X86"
echo "  Linux x86:   $SHA_LINUX_X86"
echo "  Linux ARM64: $SHA_LINUX_ARM64"

# Generate formula
FORMULA_PATH="$TAP_REPO/Formula/kipferl.rb"
mkdir -p "$(dirname "$FORMULA_PATH")"

cat > "$FORMULA_PATH" << EOF
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

echo "Formula written to: $FORMULA_PATH"
