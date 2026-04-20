#!/usr/bin/env bash
set -e

REPO="haphuongmoethptthanglong-dev/contriblat"
VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$VERSION" ]; then
  echo "Failed to fetch latest version"; exit 1
fi

# Detect OS and arch
OS=$(uname -s | tr "[:upper:]" "[:lower:]")
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      arm64|aarch64) BINARY="contribai-${VERSION}-linux-aarch64" ;;
      *)             BINARY="contribai-${VERSION}-linux-x86_64" ;;
    esac ;;
  darwin)
    case "$ARCH" in
      arm64|aarch64) BINARY="contribai-${VERSION}-macos-aarch64" ;;
      *)             BINARY="contribai-${VERSION}-macos-x86_64" ;;
    esac ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY"

echo "Installing ContribAI $VERSION..."
echo "  OS: $OS | Arch: $ARCH"
echo "  Binary: $BINARY"
echo "  Downloading from: $URL"
echo ""

curl -fsSL "$URL" -o contribai
chmod +x contribai

# Try /usr/local/bin first, fall back to ~/.local/bin
INSTALL_DIR="/usr/local/bin"
if [ -w "$INSTALL_DIR" ]; then
  mv contribai "$INSTALL_DIR/contribai"
elif command -v sudo >/dev/null 2>&1; then
  echo "Need sudo to install to $INSTALL_DIR"
  sudo mv contribai "$INSTALL_DIR/contribai"
else
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  mv contribai "$INSTALL_DIR/contribai"
  if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "  Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
    echo "  (add this to your ~/.bashrc or ~/.profile to make it permanent)"
  fi
fi

echo ""
echo "ContribAI installed to: $INSTALL_DIR/contribai"
echo "Run: contribai init"
