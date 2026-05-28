# MM-01

Browser-based clone of the Roland SH-101. Rust→WASM DSP in an AudioWorklet,
TypeScript main thread for UI / MIDI / state.

## Read these first

- [`docs/spec.md`](docs/spec.md) — product spec, **organised by iteration**.
  Each iteration section defines the scope for that step. New iterations are
  appended; the current target is always the last section.
- [`docs/architecture.md`](docs/architecture.md) — long-term system shape:
  language split, project layout, message protocol, build/tooling, and an
  "Iteration mapping" section that says what's in scope for the current step.

When picking up work, the pair to consult is **the latest iteration in
`spec.md` + the "Iteration mapping" in `architecture.md`**. Earlier iterations
in `spec.md` describe behaviour that should still hold.

## Layout

See `architecture.md` → "Project layout". In short: `crates/mm01-dsp/` (Rust,
the WASM engine), `web/` (Vite + TS app, AudioWorklet bridge, UI).

## Build

```bash
cd web
npm install
npm run build:wasm    # wasm-pack → web/wasm/
npm run dev           # Vite dev server
```

## Conventions

- DSP code is ported from VCV Rack (`Fundamental`, `Rack`) 1:1 where possible.
  See `architecture.md` → "DSP Code".
- No allocation, no panics, no blocking in the audio callback.
- Message protocol is the only main↔worklet contract — defined in Rust,
  mirrored in TS.
