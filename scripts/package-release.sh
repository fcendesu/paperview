#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
VERSION=${PAPERVIEW_VERSION:-$(awk '
  /^\[workspace.package\]/ { in_workspace_package = 1; next }
  /^\[/ { in_workspace_package = 0 }
  in_workspace_package && /^version = / {
    gsub(/"/, "", $3)
    print $3
    exit
  }
' "$ROOT_DIR/Cargo.toml")}
TARGET_TRIPLE=$(rustc -vV | awk '/host:/ { print $2 }')
PACKAGE_NAME="paperview-v${VERSION}-${TARGET_TRIPLE}"
STAGING_DIR="${ROOT_DIR}/target/dist/${PACKAGE_NAME}"
ARCHIVE_PATH="${ROOT_DIR}/target/dist/${PACKAGE_NAME}.tar.gz"

cd "$ROOT_DIR"
cargo build --release --workspace

rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"

cp target/release/paperview-gui "$STAGING_DIR/"
cp target/release/paperview-tui "$STAGING_DIR/"
cp README.md "$STAGING_DIR/"
cp LICENSE.md "$STAGING_DIR/"

tar -C "$ROOT_DIR/target/dist" -czf "$ARCHIVE_PATH" "$PACKAGE_NAME"

printf '%s\n' "$ARCHIVE_PATH"
