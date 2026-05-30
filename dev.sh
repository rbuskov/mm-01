#!/usr/bin/env bash
#
# dev.sh — build and run the ENTIRE MM-01 project.
#
# "Build everything" means all four artifacts, in order, every run:
#   1. Rust → WASM DSP engine          (wasm-pack)
#   2. AudioWorklet bundle             (esbuild → web/public/mm01-worklet.js)
#   3. TypeScript typecheck            (tsc -b)
#   4. Vite                            (dev server, or production build)
#
# Steps 1–3 always run so nothing is ever stale or missing — a missing/stale
# worklet bundle is exactly what breaks audio start ("node name 'mm01' is not
# defined"). The dev server then serves the freshly-built artifacts.
#
# Usage:
#   ./dev.sh            # build everything, then start the Vite dev server
#   ./dev.sh build      # build everything, then produce a production build (web/dist)
#   ./dev.sh preview    # build everything, then serve the production build
#
set -euo pipefail

# Always operate from the repo root (this script's directory).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT/web"

# --- Tooling checks -------------------------------------------------------
for tool in npm npx wasm-pack cargo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: '$tool' not found on PATH." >&2
    case "$tool" in
      wasm-pack) echo "  install: cargo install wasm-pack" >&2 ;;
      cargo)     echo "  install: https://rustup.rs" >&2 ;;
      npm|npx)   echo "  install: https://nodejs.org" >&2 ;;
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

# --- Build everything -----------------------------------------------------
# Each artifact is built explicitly here (rather than hidden inside an npm
# script) so a run of dev.sh visibly and guaranteed-ly builds the whole project.
build_all() {
  echo "==> [1/3] Building Rust → WASM DSP engine"
  npm run build:wasm

  echo "==> [2/3] Building AudioWorklet bundle"
  npm run build:worklet

  echo "==> [3/3] Typechecking TypeScript"
  npx tsc -b
}

# --- Run ------------------------------------------------------------------
case "${1:-dev}" in
  dev)
    build_all
    echo "==> Starting Vite dev server"
    exec npx vite
    ;;
  build)
    build_all
    echo "==> Producing production build (web/dist)"
    exec npx vite build
    ;;
  preview)
    build_all
    echo "==> Producing production build, then serving it"
    npx vite build
    exec npx vite preview
    ;;
  *)
    echo "usage: ./dev.sh [dev|build|preview]" >&2
    exit 2
    ;;
esac
