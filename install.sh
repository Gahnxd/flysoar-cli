#!/usr/bin/env sh
# Install flysoar from a GitHub Release.
# Optional environment variables:
#   FLYSOAR_VERSION=v0.1.0   Install a specific tag (default: latest)
#   FLYSOAR_INSTALL_DIR=...  Installation directory (default: ~/.local/bin)

set -eu

REPOSITORY="Gahnxd/flysoar-cli"
VERSION="${FLYSOAR_VERSION:-latest}"
INSTALL_DIR="${FLYSOAR_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
architecture="$(uname -m)"

case "$os/$architecture" in
  Darwin/arm64)
    target="aarch64-apple-darwin"
    ;;
  Darwin/x86_64)
    target="x86_64-apple-darwin"
    ;;
  Linux/x86_64)
    target="x86_64-unknown-linux-gnu"
    ;;
  *)
    echo "Unsupported platform: $os/$architecture" >&2
    echo "Download a matching release asset from https://github.com/$REPOSITORY/releases" >&2
    exit 1
    ;;
esac

archive="flysoar-${target}.tar.gz"
if [ "$VERSION" = "latest" ]; then
  release_url="https://github.com/$REPOSITORY/releases/latest/download"
else
  release_url="https://github.com/$REPOSITORY/releases/download/$VERSION"
fi

temporary_directory="$(mktemp -d)"
cleanup() {
  rm -rf "$temporary_directory"
}
trap cleanup 0 HUP INT TERM

download() {
  curl --fail --location --silent --show-error "$1" --output "$2"
}

echo "Downloading flysoar for $target..."
download "$release_url/$archive" "$temporary_directory/$archive"
download "$release_url/SHA256SUMS" "$temporary_directory/SHA256SUMS"

expected_checksum="$(awk -v asset="$archive" '$2 == asset { print $1 }' "$temporary_directory/SHA256SUMS")"
if [ -z "$expected_checksum" ]; then
  echo "Checksum for $archive was not found in SHA256SUMS" >&2
  exit 1
fi

actual_checksum=""
if command -v shasum >/dev/null 2>&1; then
  actual_checksum="$(shasum -a 256 "$temporary_directory/$archive" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  actual_checksum="$(sha256sum "$temporary_directory/$archive" | awk '{print $1}')"
else
  echo "A SHA-256 utility (shasum or sha256sum) is required" >&2
  exit 1
fi

if [ "$actual_checksum" != "$expected_checksum" ]; then
  echo "Checksum verification failed for $archive" >&2
  exit 1
fi

tar -xzf "$temporary_directory/$archive" -C "$temporary_directory"
mkdir -p "$INSTALL_DIR"
install -m 755 "$temporary_directory/flysoar" "$INSTALL_DIR/flysoar"

echo "Installed flysoar to $INSTALL_DIR/flysoar"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add $INSTALL_DIR to your PATH to run 'flysoar' from any directory." ;;
esac
