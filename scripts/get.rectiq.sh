#!/usr/bin/env sh
set -eu

# Rectiq CLI installer with signed checksum verification (minisign)
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

# Map to build targets and archive ext
case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="x86_64-unknown-linux-musl" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "error: unsupported ARCH for Linux: $ARCH" >&2; exit 1 ;;
    esac
    EXT=tar.gz
    ;;
  Darwin)
    case "$ARCH" in
      x86_64|amd64) TARGET="x86_64-apple-darwin" ;;
      aarch64|arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "error: unsupported ARCH for macOS: $ARCH" >&2; exit 1 ;;
    esac
    EXT=tar.gz
    ;;
  *) echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

# Determine version tag
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
VER=${TAG#rectiq-cli-v}

ARCHIVE="rectiq-cli-${VER}-${TARGET}.${EXT}"
BASE="https://github.com/${OWNER}/${REPO}/releases/download/${TAG}"
ARCHIVE_URL="${BASE}/${ARCHIVE}"
SUMS_URL="${BASE}/SHA256SUMS.txt"
SIG_URL="${SUMS_URL}.minisig"

TMPDIR=${TMPDIR:-/tmp}
WORKDIR="$(mktemp -d "${TMPDIR}/rectiq.XXXXXX")"
trap 'rm -rf "$WORKDIR"' EXIT INT TERM

# Download and verify SHA256SUMS.txt (unless bypassed)
if [ -z "${RECTIQ_INSECURE_SKIP_VERIFY:-}" ]; then
  if ! command -v minisign >/dev/null 2>&1; then
    echo "minisign is required to verify; set RECTIQ_INSECURE_SKIP_VERIFY=1 to bypass (not recommended)." >&2
    echo "macOS: brew install minisign | Debian/Ubuntu: sudo apt-get install -y minisign" >&2
    exit 1
  fi
  PUBKEY_FILE="$WORKDIR/rectiq-minisign.pub"
  if [ -n "${RECTIQ_MINISIGN_PUBKEY:-}" ]; then
    printf '%s\n' "$RECTIQ_MINISIGN_PUBKEY" > "$PUBKEY_FILE"
  else
    PUBKEY_URL=${RECTIQ_MINISIGN_PUBKEY_URL:-https://raw.githubusercontent.com/purlity/rectiq/main/SECURITY/rectiq-minisign.pub}
    echo "Fetching minisign public key from $PUBKEY_URL" >&2
    curl -fsSL "$PUBKEY_URL" -o "$PUBKEY_FILE"
  fi
  echo "Downloading $SUMS_URL and signature" >&2
  curl -fsSL "$SUMS_URL" -o "$WORKDIR/SHA256SUMS.txt"
  curl -fsSL "$SIG_URL"  -o "$WORKDIR/SHA256SUMS.txt.minisig"
  echo "Verifying signature..." >&2
  minisign -Vm "$WORKDIR/SHA256SUMS.txt" -P "$PUBKEY_FILE"
else
  echo "WARNING: RECTIQ_INSECURE_SKIP_VERIFY set; skipping signature verification" >&2
  curl -fsSL "$SUMS_URL" -o "$WORKDIR/SHA256SUMS.txt"
fi

echo "Downloading $ARCHIVE_URL" >&2
curl -fL "$ARCHIVE_URL" -o "$WORKDIR/artifact.$EXT"

# Verify archive checksum against verified list
EXPECTED=$(awk -v f="$ARCHIVE" '$2==f {print $1}' "$WORKDIR/SHA256SUMS.txt" || true)
if [ -z "$EXPECTED" ]; then
  echo "error: could not find checksum for $ARCHIVE in SHA256SUMS.txt" >&2
  exit 1
fi
if [ "$OS" = "Darwin" ]; then
  ACTUAL=$(shasum -a 256 "$WORKDIR/artifact.$EXT" | awk '{print $1}')
else
  need sha256sum
  ACTUAL=$(sha256sum "$WORKDIR/artifact.$EXT" | awk '{print $1}')
fi
if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "error: checksum mismatch for $ARCHIVE" >&2
  echo "expected: $EXPECTED" >&2
  echo "actual:   $ACTUAL" >&2
  exit 1
fi

# Extract
need tar
tar -xzf "$WORKDIR/artifact.$EXT" -C "$WORKDIR"

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

