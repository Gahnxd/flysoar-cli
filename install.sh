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
    platform_label="macOS (Apple Silicon)"
    ;;
  Darwin/x86_64)
    target="x86_64-apple-darwin"
    platform_label="macOS (Intel)"
    ;;
  Linux/x86_64)
    target="x86_64-unknown-linux-gnu"
    platform_label="Linux"
    ;;
  *)
    echo "Sorry, flysoar doesn't have a ready-to-use build for your computer yet." >&2
    echo "You can find manual download options at https://github.com/$REPOSITORY/releases" >&2
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

# Spinner + step runner: shows an animated spinner (when attached to a
# terminal) while a step runs in the background, then prints a plain
# success/failure line. On failure, the captured output is shown so the
# user has something useful to share if they need help.
run_step() {
  step_message="$1"
  shift
  step_log="$temporary_directory/step.log"

  ("$@") >"$step_log" 2>&1 &
  step_pid=$!

  if [ -t 1 ]; then
    frame_index=0
    while kill -0 "$step_pid" 2>/dev/null; do
      case $frame_index in
        0) frame="⠋" ;;
        1) frame="⠙" ;;
        2) frame="⠹" ;;
        3) frame="⠸" ;;
        4) frame="⠼" ;;
        5) frame="⠴" ;;
        6) frame="⠦" ;;
        7) frame="⠧" ;;
        8) frame="⠇" ;;
        *) frame="⠏" ;;
      esac
      frame_index=$(( (frame_index + 1) % 10 ))
      printf '\r%s %s' "$frame" "$step_message"
      sleep 0.1
    done
  fi

  if wait "$step_pid"; then
    printf '\r✓ %s\n' "$step_message"
  else
    printf '\r✗ %s\n' "$step_message" >&2
    echo "" >&2
    echo "Something went wrong. Details:" >&2
    cat "$step_log" >&2
    echo "" >&2
    echo "Please try again, or open an issue at https://github.com/$REPOSITORY/issues" >&2
    exit 1
  fi
}

download_and_verify() {
  curl --fail --location --silent --show-error "$release_url/$archive" --output "$temporary_directory/$archive"
  curl --fail --location --silent --show-error "$release_url/SHA256SUMS" --output "$temporary_directory/SHA256SUMS"

  expected_checksum="$(awk -v asset="$archive" '$2 == asset { print $1 }' "$temporary_directory/SHA256SUMS")"
  if [ -z "$expected_checksum" ]; then
    echo "Could not verify this download (missing checksum entry)."
    exit 1
  fi

  if command -v shasum >/dev/null 2>&1; then
    actual_checksum="$(shasum -a 256 "$temporary_directory/$archive" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "$temporary_directory/$archive" | awk '{print $1}')"
  else
    echo "A SHA-256 utility (shasum or sha256sum) is required to verify this download."
    exit 1
  fi

  if [ "$actual_checksum" != "$expected_checksum" ]; then
    echo "This download did not match its expected checksum and was discarded."
    exit 1
  fi
}

install_binary() {
  tar -xzf "$temporary_directory/$archive" -C "$temporary_directory"
  mkdir -p "$INSTALL_DIR"
  install -m 755 "$temporary_directory/flysoar" "$INSTALL_DIR/flysoar"
}

echo "Install Flysoar CLI, a command-line flight search tool."
echo ""

run_step "Downloading flysoar for $platform_label" download_and_verify
run_step "Installing flysoar" install_binary

echo ""
echo "flysoar is installed!"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "Run 'flysoar --help' to get started."
    ;;
  *)
    if [ -n "${SHELL:-}" ] && [ -x "$SHELL" ] && [ -r /dev/tty ] && [ -w /dev/tty ]; then
      echo "Finishing setup..."
      export PATH="$INSTALL_DIR:$PATH"
      exec "$SHELL" -l </dev/tty >/dev/tty 2>&1
    fi
    echo "Restart your terminal, then run 'flysoar --help' to get started."
    ;;
esac
