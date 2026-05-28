#!/usr/bin/env bash
#
# dev.sh — build and run the entire MM-01 project.
#
# Builds the Rust→WASM DSP engine (wasm-pack) and starts the Vite dev server,
# which serves the TypeScript main-thread app + AudioWorklet bridge.
#
# Usage:
#   ./dev.sh            # install deps if needed, build WASM, run dev server
#   ./dev.sh build      # production build (wasm --release, tsc, vite build)
#   ./dev.sh preview    # serve the production build from web/dist
#
set -euo pipefail

# Always operate from the repo root (this script's directory).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT/web"

# --- Tooling checks -------------------------------------------------------
for tool in npm wasm-pack cargo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: '$tool' not found on PATH." >&2
    case "$tool" in
      wasm-pack) echo "  install: cargo install wasm-pack" >&2 ;;
      cargo)     echo "  install: https://rustup.rs" >&2 ;;
      npm)       echo "  install: https://nodejs.org" >&2 ;;
    esac
    exit 1
  fi
done

# --- Dependencies ---------------------------------------------------------
# Install node deps when missing or out of date relative to the lockfile.
if [ ! -d node_modules ] || [ package-lock.json -nt node_modules ]; then
  echo "==> Installing npm dependencies"
  npm install
fi

# --- Run ------------------------------------------------------------------
case "${1:-dev}" in
  dev)
    echo "==> Building WASM + starting Vite dev server"
    exec npm run dev
    ;;
  build)
    echo "==> Production build (WASM release + tsc + vite build)"
    exec npm run build
    ;;
  preview)
    echo "==> Building then serving production preview"
    npm run build
    exec npm run preview
    ;;
  *)
    echo "usage: ./dev.sh [dev|build|preview]" >&2
    exit 2
    ;;
esac
