# MM-01 Architecture

MM-01 is a browser-only application. There is no backend: the synth, sequencer, MIDI handling, and UI all run client-side. The system is split into two languages crossing a single boundary:

- **Rust**, compiled to **WebAssembly**, implements all real-time audio code (DSP, voice state, sequencer, clock) and runs inside an **AudioWorklet**.
- **TypeScript** implements the UI, control surface, MIDI I/O, and asset loading, running on the main thread.

The two halves communicate through a small, typed message protocol over the worklet's `MessagePort`.

## High-level layout

```
┌────────────────────────────── Main thread (TypeScript) ──────────────────────────────┐
│                                                                                      │
│   ┌──────────────┐    ┌──────────────┐    ┌────────────────┐    ┌────────────────┐   │
│   │  Panel UI    │    │  Keyboard    │    │  MIDI I/O      │    │  App state     │   │
│   │  (React/…)   │    │  (mouse/kbd) │    │  (Web MIDI)    │    │  (patch, seq)  │   │
│   └──────┬───────┘    └──────┬───────┘    └───────┬────────┘    └────────┬───────┘   │
│          │                   │                    │                      │           │
│          └───────────────────┴────────┬───────────┴──────────────────────┘           │
│                                       │                                              │
│                              ┌────────▼─────────┐                                    │
│                              │  Worklet bridge  │  ← typed messages                  │
│                              │  (MessagePort)   │                                    │
│                              └────────┬─────────┘                                    │
└───────────────────────────────────────┼──────────────────────────────────────────────┘
                                        │
┌───────────────────────── Audio thread (AudioWorklet) ────────────────────────────────┐
│                                                                                      │
│                            ┌───────────▼──────────┐                                  │
│                            │  mm01-dsp (Rust/WASM)│                                  │
│                            │  ┌────────────────┐  │                                  │
│                            │  │ Voice (VCO/VCA │  │                                  │
│                            │  │ /VCF/EG/LFO …) │  │                                  │
│                            │  ├────────────────┤  │                                  │
│                            │  │ Sequencer +    │  │                                  │
│                            │  │ MIDI clock     │  │                                  │
│                            │  ├────────────────┤  │                                  │
│                            │  │ Parameter      │  │                                  │
│                            │  │ smoothing      │  │                                  │
│                            │  └───────┬────────┘  │                                  │
│                            └──────────┼───────────┘                                  │
│                                       │ audio buffer                                 │
│                            ┌──────────▼───────────┐                                  │
│                            │ AudioContext output  │ → speakers                       │
│                            └──────────────────────┘                                  │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

## Why this split

- **AudioWorklet for DSP** is the only mechanism in the browser that runs audio code on a dedicated real-time-priority thread with a fixed 128-sample render quantum. Anything that must not glitch (voice, filter, envelopes, sequencer timing) lives here.
- **WebAssembly for the DSP language** gives us deterministic, allocation-free code at near-native speed, with no GC pauses in the audio callback. Writing the inner loops in JS would not meet the latency budget at higher polyphony, modulation depth, or oversampling rates.
- **Rust** specifically: memory safety without a runtime, mature `wasm-bindgen` / `wasm-pack` tooling, and (most importantly) a port target for the VCV Rack DSP code that preserves the original's structure. The VCV modules are written as small, self-contained C++ structs with `process()` methods — these map almost line-for-line onto Rust structs with `process()` methods.
- **TypeScript on the main thread** for everything that touches the DOM, Web MIDI, or persistent state. Putting MIDI I/O on the main thread is not a choice — `navigator.requestMIDIAccess()` is only available there.
- **Sequencer and clock live in the worklet**, not the main thread. They need sample-accurate timing to stay phase-locked with audio and to emit MIDI clock at jitter-free 24 PPQN. The main thread schedules outgoing MIDI messages based on events the worklet emits, and forwards incoming MIDI clock the other way.

## Project layout

```
mm-01/
├── crates/
│   └── mm01-dsp/             # Rust crate, compiled to WASM
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs        # wasm-bindgen exports (Engine, message handlers)
│           ├── voice/        # VCO, VCF, VCA, EG, LFO, S&H, glide
│           ├── seq/          # step sequencer + transport
│           ├── clock/        # MIDI clock master/slave, SPP
│           ├── params/       # parameter smoothing, sample-rate-aware ramps
│           └── primitives/   # shared math (oversampling, polyblep, etc.)
├── web/                      # TypeScript app (Vite)
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   ├── main.ts               # entry, AudioContext setup, worklet load
│   ├── audio/
│   │   ├── worklet.ts        # AudioWorkletProcessor (imports WASM)
│   │   └── bridge.ts         # main-thread side of the message protocol
│   ├── midi/                 # Web MIDI in/out, clock translation
│   ├── ui/                   # panel, knobs, sliders, keyboard, sequencer
│   ├── state/                # patch state, persistence
│   └── wasm/                 # wasm-pack output (gitignored)
├── reference/                # cloned VCV repos (gitignored, see DSP Code)
└── docs/
    ├── spec.md
    └── architecture.md
```

The Rust crate is its own Cargo project, not part of a workspace — there is only one crate. Build output (`.wasm` + JS glue) is emitted into `web/wasm/` and imported from `worklet.ts`. The Vite `root` is `web/`; `index.html`, `main.ts`, and the feature folders sit directly under it (no inner `src/`).

## DSP layer (Rust → WASM)

The crate exposes a single `Engine` type via `wasm-bindgen`. The worklet instantiates one `Engine` and drives it per render quantum:

```rust
#[wasm_bindgen]
pub struct Engine { /* voice, seq, clock, params */ }

#[wasm_bindgen]
impl Engine {
    pub fn new(sample_rate: f32) -> Engine;
    pub fn process(&mut self, out_left: &mut [f32], out_right: &mut [f32]);
    pub fn handle_message(&mut self, bytes: &[u8]); // control → DSP
    pub fn drain_events(&mut self) -> Vec<u8>;      // DSP → main (note out, step, clock)
}
```

Rules the crate must follow:

- **No allocation in `process()`.** All buffers are pre-sized at construction. Events emitted from `process()` are pushed into a fixed-capacity ring buffer that `drain_events` consumes between callbacks.
- **No panics in `process()`.** Indexing uses checked construction at setup and `unsafe` `get_unchecked` in hot loops where bounds are proven.
- **`f32` end-to-end** to match `AudioWorkletProcessor` buffers.
- **Deterministic given inputs and sample rate.** No reads from `Date`, `Math.random` equivalents, etc. — RNG (for noise, S&H) is seeded explicitly.

Parameter changes arrive as messages but are not applied raw — every control routes through a per-parameter smoother (one-pole or linear ramp depending on the control) to avoid zipper noise on knob twiddles.

## Audio runtime (AudioWorklet)

The worklet processor is thin. Its job is to:

1. Receive the compiled `WebAssembly.Module` from the main thread (via `processorOptions`) and instantiate it.
2. Allocate the WASM-side output buffers once at construction.
3. On each `process()` call: forward queued messages into the engine, run `engine.process()`, copy WASM memory into the `Float32Array` outputs Web Audio gives us, and post any drained events back to the main thread.
4. Never block, never allocate, never `await`.

The worklet feeds a single `AudioWorkletNode` connected directly to `audioContext.destination`. No intermediate Web Audio nodes — the DSP graph is internal to WASM. This keeps the routing inside one language and avoids redundant buffer copies through `GainNode`/`BiquadFilterNode`.

## Main-thread layer (TypeScript)

The main thread owns:

- **AudioContext lifecycle.** Created on first user gesture (Web Audio autoplay policy). The worklet module and WASM binary are fetched and compiled here, then handed off.
- **UI.** Panel controls, on-screen keyboard, sequencer grid. Control changes are debounced/throttled appropriately for the medium (mouse drag at animation frame rate is fine; computer-keyboard key events are forwarded immediately).
- **MIDI.** `navigator.requestMIDIAccess()`, port selection, mapping between MIDI messages and engine messages. Incoming MIDI clock and transport messages are forwarded into the worklet, which decides whether to act on them (slave mode) or ignore them (master mode). Outgoing MIDI is sent based on events the worklet emits — note on/off from the sequencer, clock ticks when in master mode.
- **Patch state.** The current values of all controls, the sequence pattern, MIDI routing, etc. The main thread is the source of truth for what the user sees; the worklet holds the DSP-side mirror but only as a target for its smoothers and oscillators. On boot or after a tab is restored, the main thread replays the full patch into the worklet.

UI framework choice (React, Svelte, Solid, or plain DOM) is deliberately not pinned here — it doesn't cross the worklet boundary and can be revisited per iteration. Iteration 1 can ship with plain TypeScript + DOM if that's faster.

## Control & event protocol

Messages between main thread and worklet are encoded as compact binary (a tagged-union over a small `ArrayBuffer`) rather than JSON. Reasons: zero parsing cost on the audio side, no per-message GC pressure, and a stable schema that's defined once in Rust (`#[repr(C)]` enums) and mirrored in a generated TS file.

Two channels, both over the same `MessagePort`:

**Main → Worklet (control):**
- `NoteOn { note, velocity }` / `NoteOff { note }`
- `ParamSet { id, value }` — any continuous control (cutoff, resonance, env times, …)
- `TransportStart` / `TransportStop` / `TransportContinue` / `SongPositionPointer { sixteenths }`
- `ExternalClockTick` — forwarded MIDI clock pulse (used in slave mode)
- `SetTempo { bpm }` — used in master mode
- `LoadPattern { steps[…] }`

**Worklet → Main (events):**
- `NoteOutOn { note, velocity }` / `NoteOutOff { note }` — from the sequencer, for MIDI out
- `StepAdvanced { index }` — for UI playhead
- `ClockTickOut` — for MIDI out in master mode (24 PPQN)
- `XRun` — diagnostic, emitted if a process callback overran

For high-frequency state that the UI just needs to *display* (e.g. current step, VU meter), a `SharedArrayBuffer` ring is a viable optimisation later. We do not need it for iteration 1 — `postMessage` of step events at sequencer rate is well under any pressure point.

## Sequencer & clock

Both live in the worklet so they share a sample clock with the DSP:

- The transport runs off a sample counter; tempo is converted to samples-per-pulse at the engine's sample rate. This guarantees the sequencer phase-locks to the audio it produces.
- In **master** mode, the engine emits `ClockTickOut` events 24 times per quarter note, plus transport events on start/stop/continue. The main thread translates these into MIDI bytes and sends them out.
- In **slave** mode, the engine consumes `ExternalClockTick` messages and adjusts its internal phase to follow. Jitter from `postMessage` is absorbed by a small filter (running average of recent tick intervals) so the audio-side tempo doesn't twitch on every message.
- **Song Position Pointer** is handled by translating sixteenth-note offsets into the engine's step index and intra-step phase before any further ticks are processed.

## Build & tooling

- **Rust → WASM:** `wasm-pack build ../crates/mm01-dsp --target web --release --out-dir ../../web/wasm` invoked from a `web/` npm script (`--out-dir` is resolved relative to the crate). Output lands in `web/wasm/` (gitignored).
- **Web app:** Vite for dev server, build, and TS compilation. The AudioWorklet file is built as a separate entry so it can be loaded via `audioWorklet.addModule()`.
- **Cross-origin isolation:** the dev server and production hosting must serve `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` so we can use `SharedArrayBuffer` if/when we move to it. Configured in `vite.config.ts`.
- **Testing:** Rust unit tests for DSP modules (run natively against reference impulse/sine inputs, not in the browser). TS tests for the message codec and MIDI translation. No end-to-end audio assertions yet — listening is the test.
- **CI:** lint + test for both halves; produce a static-hosted bundle as the deliverable.

## Iteration mapping

The architecture above describes the full target. For iteration 2 (per [spec.md](spec.md)):

- `voice::vco` expands to produce saw, pulse, and three selectable sub-oscillator shapes from a single phase accumulator — all phase-locked because they share the accumulator. All hard-edged outputs are band-limited via PolyBLEP from `primitives/`.
- `voice::noise` lands — a new module producing white noise from an explicitly seeded xorshift (per the determinism rule in the DSP layer section).
- `voice::mixer` lands — a 4-input linear summer with per-input gain. No clipping, no normalisation; the deliberate post-unity overdrive is the spec.
- `voice::vca` from iteration 1 stays at the end of the chain, still gate-driven on/off. `voice::vcf`, `eg`, `lfo`, `seq`, and `clock` remain absent.
- The protocol adds `ParamSet` IDs for footage, sub-osc shape select, and the four mixer levels. No new message *types* — the tagged-union is unchanged.
- The UI gains a footage selector, a sub-osc shape selector, and four mixer level controls. Keyboard and bridge wiring are unchanged.

No part of the iteration 2 scope contradicts the long-term shape; we add modules and parameter IDs without restructuring the boundary.

### DSP Code

All DSP code will be ported from the VCV Rack project. Basic synth modules are found in the repo https://github.com/VCVRack/Fundamental.git

The core VCV rack project is found at https://github.com/VCVRack/Rack.git

Clone these repos and use then as reference. Port the DSP code 1:1 if possible.
