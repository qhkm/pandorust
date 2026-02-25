#!/bin/sh
# pandorust installer — downloads the latest release binary
# Usage: curl -fsSL https://raw.githubusercontent.com/qhkm/pandorust/main/install.sh | sh

set -e

REPO="qhkm/pandorust"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS="linux" ;;
    Darwin) OS="darwin" ;;
    *)      echo "Error: unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *)             echo "Error: unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="pandorust-${OS}-${ARCH}"

# Get latest release tag
echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: could not determine latest release"
    exit 1
fi

echo "Latest version: $LATEST"

# Download binary
URL="https://github.com/${REPO}/releases/download/${LATEST}/${TARGET}.tar.gz"
echo "Downloading ${URL}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "${TMPDIR}/pandorust.tar.gz"
tar -xzf "${TMPDIR}/pandorust.tar.gz" -C "$TMPDIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/pandorust" "${INSTALL_DIR}/pandorust"
else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "${TMPDIR}/pandorust" "${INSTALL_DIR}/pandorust"
fi

chmod +x "${INSTALL_DIR}/pandorust"

echo ""
echo "pandorust ${LATEST} installed to ${INSTALL_DIR}/pandorust"
echo ""
pandorust --version
