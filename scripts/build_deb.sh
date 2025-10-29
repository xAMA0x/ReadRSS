#!/usr/bin/env bash
set -euo pipefail

# ===
#
# Construit un paquet .deb pour rss-gui via cargo-deb
#
# ===

if ! command -v cargo-deb >/dev/null 2>&1; then
  echo "[Setup] Installation de cargo-deb (cargo install cargo-deb)"
  cargo install cargo-deb
fi

echo "[Build] cargo deb -p rss-gui -- --release"
cargo deb -p rss-gui -- --release

TARGET_DIR=${CARGO_TARGET_DIR:-}
if [[ -z "${TARGET_DIR}" && -f .cargo/config.toml ]]; then
  CFG_DIR=$(sed -n 's/^target-dir = "\(.*\)"/\1/p' .cargo/config.toml | head -n1 || true)
  TARGET_DIR=${CFG_DIR:-target}
fi
TARGET_DIR=${TARGET_DIR:-target}

echo "[OK] Paquets générés dans $TARGET_DIR/debian/"
