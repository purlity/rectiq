#!/usr/bin/env sh
set -eu

# Simple installer for Rectiq CLI from GitHub Releases
# Usage: curl -fsSL https://get.rectiq.sh | sh

OWNER=${OWNER:-purlity}
REPO=${REPO:-rectiq}
BIN_NAME=${BIN_NAME:-rectiq}

# Choose install dir
if [ "${PREFIX:-}" != "" ]; then
  INSTALL_DIR="$PREFIX/bin"
elif [ "$(id -u)" -eq 0 ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.local/bin"
fi
mkdir -p "$INSTALL_DIR"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing required tool: $1" >&2
    exit 1
  }
}

need curl

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux) OS_TAG=unknown-linux-musl ; EXT=tar.xz ;;
  Darwin) OS_TAG=apple-darwin ; EXT=tar.xz ;;
  *) echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_TAG=x86_64 ;;
  aarch64|arm64) ARCH_TAG=aarch64 ;;
  *) echo "error: unsupported ARCH: $ARCH" >&2; exit 1 ;;
esac

# Determine version
if [ -n "${VERSION:-}" ]; then
  TAG="rectiq-cli-v${VERSION}"
else
  API="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
  TAG=$(curl -fsSL "$API" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  if [ -z "$TAG" ]; then
    echo "error: unable to determine latest release tag" >&2
    exit 1
  fi
fi

ASSET="${TAG}-${ARCH_TAG}-${OS_TAG}.${EXT}"
URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG}/${ASSET}"

TMPDIR=${TMPDIR:-/tmp}
WORKDIR="$(mktemp -d "${TMPDIR}/rectiq.XXXXXX")"
trap 'rm -rf "$WORKDIR"' EXIT INT TERM

echo "Downloading $URL" >&2
curl -fL "$URL" -o "$WORKDIR/artifact.$EXT"

case "$EXT" in
  tar.xz)
    need tar
    tar -xJf "$WORKDIR/artifact.$EXT" -C "$WORKDIR"
    ;;
  zip)
    need unzip
    unzip -q "$WORKDIR/artifact.$EXT" -d "$WORKDIR"
    ;;
esac

if [ -x "$WORKDIR/$BIN_NAME" ]; then
  SRC="$WORKDIR/$BIN_NAME"
elif [ -x "$WORKDIR/bin/$BIN_NAME" ]; then
  SRC="$WORKDIR/bin/$BIN_NAME"
else
  echo "error: failed to locate extracted binary" >&2
  exit 1
fi

install -m 0755 "$SRC" "$INSTALL_DIR/$BIN_NAME"
echo "Installed to $INSTALL_DIR/$BIN_NAME"
"$INSTALL_DIR/$BIN_NAME" --version || true

# Initialize default config if missing (zero-touch)
CFG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/rectiq"
CFG="$CFG_DIR/config.toml"
if [ ! -f "$CFG" ]; then
  mkdir -p "$CFG_DIR"
  : "${RECTIQ_API_BASE:=}"
  {
    printf '%s\n' "api_base = \"${RECTIQ_API_BASE:-https://api.rectiq.com}\""
    printf '%s\n' "profile  = \"default\""
  } >"$CFG"
  echo "Wrote default config to $CFG" >&2
fi
