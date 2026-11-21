#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_PATH="${REPO_ROOT}/target/release/video-solver"
LINK_PATH="${TARGET_DIR}/video-solver"

echo "==> Building video-solver (release mode)..."
(cd "${REPO_ROOT}" && cargo build --release)

if [ ! -f "${BINARY_PATH}" ]; then
  echo "Error: Expected binary not found at ${BINARY_PATH}" >&2
  exit 1
fi

echo "==> Symlinking ${BINARY_PATH} -> ${LINK_PATH}..."

# Check if destination directory exists
if [ ! -d "${TARGET_DIR}" ]; then
  echo "Creating directory ${TARGET_DIR}..."
  if [ -w "$(dirname "${TARGET_DIR}")" ]; then
    mkdir -p "${TARGET_DIR}"
  else
    sudo mkdir -p "${TARGET_DIR}"
  fi
fi

# Link with or without sudo depending on write access
if [ -w "${TARGET_DIR}" ]; then
  ln -sf "${BINARY_PATH}" "${LINK_PATH}"
else
  echo "Elevating permissions to link in ${TARGET_DIR}..."
  sudo ln -sf "${BINARY_PATH}" "${LINK_PATH}"
fi

echo "==> Successfully installed dev symlink:"
ls -l "${LINK_PATH}"
echo ""
echo "You can now run 'video-solver' from anywhere."
echo "To remove this symlink later, run:"
echo "  sudo rm -f ${LINK_PATH}"
