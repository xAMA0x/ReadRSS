#!/usr/bin/env bash
set -euo pipefail

# ===
#
# Build release de l’application (binaire rss-gui)
#
# ===

echo "[Build] cargo build --release -p rss-gui"
cargo build --release -p rss-gui

TARGET_DIR=${CARGO_TARGET_DIR:-}
if [[ -z "${TARGET_DIR}" ]]; then
  if [[ -f .cargo/config.toml ]]; then
    CFG_DIR=$(sed -n 's/^target-dir = "\(.*\)"/\1/p' .cargo/config.toml | head -n1 || true)
    if [[ -n "${CFG_DIR:-}" ]]; then
      TARGET_DIR="$CFG_DIR"
    fi
  fi
fi
TARGET_DIR=${TARGET_DIR:-target}
BIN1="$TARGET_DIR/release/rss-gui"
BIN2="$TARGET_DIR/release/rss_gui"

if [[ -x "$BIN1" ]]; then
  echo "[OK] Binaire: $BIN1"
  exit 0
elif [[ -x "$BIN2" ]]; then
  echo "[OK] Binaire: $BIN2"
  exit 0
else
  echo "[ERREUR] Binaire non trouvé dans $TARGET_DIR/release/ (recherché: rss-gui/rss_gui)" >&2
  ls -la "$TARGET_DIR" || true
  ls -la "$TARGET_DIR/release" || true
  exit 1
fi
