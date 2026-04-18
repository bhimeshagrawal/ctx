#!/usr/bin/env bash

set -euo pipefail

REPO="${CTX_REPO:-bhimeshagrawal/ctx}"
VERSION="${CTX_VERSION:-latest}"
BIN_NAME="ctx"
DEFAULT_BIN_DIR="${HOME}/.local/bin"
BIN_DIR="${CTX_INSTALL_DIR:-$DEFAULT_BIN_DIR}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log() {
  printf '%s\n' "$*"
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

detect_os() {
  case "$(uname -s)" in
    Darwin) printf 'darwin' ;;
    Linux) printf 'linux' ;;
    *) fail "unsupported operating system: $(uname -s)" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    arm64|aarch64) printf 'arm64' ;;
    x86_64|amd64) printf 'x64' ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
  esac
}

release_base_url() {
  if [ "$VERSION" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download' "$REPO"
  else
    printf 'https://github.com/%s/releases/download/%s' "$REPO" "$VERSION"
  fi
}

verify_checksum() {
  local asset_name="$1"
  local archive_path="$2"
  local checksums_path="$3"

  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    log "checksum verification skipped: no sha256 tool found"
    return 0
  fi

  if ! grep -q " ${asset_name}\$" "$checksums_path"; then
    log "checksum verification skipped: ${asset_name} not present in checksums file"
    return 0
  fi

  if command -v shasum >/dev/null 2>&1; then
    (cd "$(dirname "$archive_path")" && shasum -a 256 -c "$checksums_path" --ignore-missing >/dev/null)
  else
    (cd "$(dirname "$archive_path")" && sha256sum -c "$checksums_path" --ignore-missing >/dev/null)
  fi
  log "checksum verified for ${asset_name}"
}

need_cmd curl
need_cmd tar

OS="$(detect_os)"
ARCH="$(detect_arch)"
ASSET_NAME="${BIN_NAME}-${OS}-${ARCH}.tar.gz"
BASE_URL="$(release_base_url)"
ARCHIVE_URL="${BASE_URL}/${ASSET_NAME}"
CHECKSUMS_URL="${BASE_URL}/checksums.txt"
ARCHIVE_PATH="${TMP_DIR}/${ASSET_NAME}"
CHECKSUMS_PATH="${TMP_DIR}/checksums.txt"

log "installing ${BIN_NAME} for ${OS}-${ARCH}"
log "source: ${ARCHIVE_URL}"

mkdir -p "$BIN_DIR"

curl -fsSL "$ARCHIVE_URL" -o "$ARCHIVE_PATH"
if curl -fsSL "$CHECKSUMS_URL" -o "$CHECKSUMS_PATH"; then
  verify_checksum "$ASSET_NAME" "$ARCHIVE_PATH" "$CHECKSUMS_PATH"
else
  log "checksums.txt not found for this release; skipping checksum verification"
fi

tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

if [ -f "${TMP_DIR}/${BIN_NAME}" ]; then
  SRC_DIR="${TMP_DIR}"
else
  fail "could not find ${BIN_NAME} in extracted archive"
fi

install -m 0755 "${SRC_DIR}/${BIN_NAME}" "${BIN_DIR}/${BIN_NAME}"

log "installed to ${BIN_DIR}/${BIN_NAME}"

case ":$PATH:" in
  *":${BIN_DIR}:"*) ;;
  *)
    log ""
    log "add this to your shell profile if needed:"
    log "  export PATH=\"${BIN_DIR}:\$PATH\""
    ;;
esac

log ""
log "next steps:"
log "  ${BIN_NAME} setup"

if [ "${CTX_RUN_SETUP:-0}" = "1" ]; then
  log ""
  log "running ${BIN_NAME} setup"
  CTX_DATA_DIR="${CTX_DATA_DIR:-}" CTX_CACHE_DIR="${CTX_CACHE_DIR:-}" "${BIN_DIR}/${BIN_NAME}" setup
fi
