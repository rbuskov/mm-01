# MM-01 Specification

MM-01 is a browser-based clone of the Roland SH-101 synthesizer. It faithfully reproduces the SH-101's full signal-path architecture and includes a playable on-screen keyboard for auditioning sounds and a built-in monophonic step sequencer. It does **not** include an arpeggiator.

MM01 supports two-way MIDI integration:
**MIDI input:** play the synth from an external MIDI keyboard or controller, and slave MM01's sequencer to an incoming MIDI clock.

**MIDI output:** the step sequencer transmits notes to external MIDI gear, and MM01 can act as the clock master.

MIDI clock sync is bidirectional and includes full **MIDI transport messages** — Start, Stop, Continue, and **Song Position Pointer (SPP)** — so MM01 stays aligned with external gear no matter where playback begins in the pattern.

## Iteration 1: Basic synth voice and keyboard

The goal of this iteration is to get sound out of the browser as quickly as possible. We build the smallest signal path that can produce a pitched note, and a minimal on-screen keyboard to trigger it. Fidelity to the SH-101 is **not** a concern yet — that comes in later iterations as we flesh out the voice.

### Voice

The voice is a single VCO feeding a gate-controlled VCA. Nothing else.

- **VCO** — one oscillator producing a single sawtooth waveform. Pitch is set directly by the note being played; there is no glide, vibrato, pulse width, sub-oscillator, or noise source yet.
- **VCA** — a simple on/off amplifier driven by the gate signal. When a key is held, the VCA is fully open; when released, it is fully closed. There is no envelope, so notes start and stop instantly (expect audible clicks at this stage — that is fine for now and will be solved when we add the envelope generator in a later iteration).

The voice is monophonic. Playing a new note while another is held simply re-pitches the VCO; releasing any key while another remains held keeps the gate open.

### Keyboard

An on-screen keyboard is rendered below the (currently empty) synth panel for auditioning the voice.

- **Range** — matches the SH-101's 32-key range (C2–G4).
- **Mouse / touch** — pressing a key opens the gate and sets the VCO pitch; releasing (or moving off the key) closes the gate.
- **Computer keyboard** — the standard two-row mapping (`Z S X D C V G B H N J M ,` for the lower octave, `Q 2 W 3 E R 5 T 6 Y 7 U I` for the upper) plays notes. Holding a key keeps the gate open; releasing closes it.
- **Octave shift** — two buttons (or `-` / `+` keys) shift the computer-keyboard mapping up or down by one octave so the full range is reachable.

There is no velocity sensitivity, no MIDI input, and no visual feedback beyond the key's pressed state.

## Iteration 2: Oscillator and source mixer

This iteration replaces the single-saw VCO from iteration 1 with the SH-101's full source section: one oscillator core producing three simultaneous waveforms, a noise generator, and a 4-input mixer that sums them into the voice's audio path. Everything downstream — filter, amp shaping, envelope, LFO — and PWM remain out of scope. The iteration-1 gate-controlled VCA stays in place so notes can still be auditioned.

### Oscillator

A single monophonic oscillator core. Pitch is set by the played note plus a **footage** selector (16′ / 8′ / 4′ / 2′, with 8′ as nominal — i.e. the note as played). All three waveform outputs come from the same core, so they are inherently phase-locked, and each gets its own level in the source mixer.

- **Sawtooth** — full-range ramp.
- **Pulse** — fixed 50% square. No width control and no modulation yet; PWM lands in a later iteration.
- **Sub-oscillator** — derived from the master by frequency division. One shape at a time, picked by a 3-way selector:
  - Square, one octave down (−1 oct)
  - Square, two octaves down (−2 oct)
  - Pulse, two octaves down (−2 oct), roughly 25% duty (narrower than the squares)

### Noise

White noise generator, independent of pitch. Feeds the mixer as a fourth source alongside the three oscillator outputs.

### Source mixer

Four linear-summing inputs, each with an independent level (0 → max):

- Sawtooth
- Pulse / square
- Sub-oscillator (post select-switch)
- Noise

The mixer **does not normalise**. Several sources at full level deliberately sum past unity — that overdrive is part of the SH-101's character and is what drives the filter on the real unit in a later iteration. The mixer itself does no clipping; it just sums and outputs a single mono signal.

### DSP notes

The saw, pulse, and both sub-oscillator shapes have hard discontinuities and must be band-limited (PolyBLEP, wavetable, or equivalent) to avoid aliasing. Noise does not need band-limiting.

### Out of scope

PWM, VCF, envelope-shaped VCA, envelope generator, LFO, portamento, and any pitch/CV modulation. The iteration-1 gate-driven VCA remains, but only as a hard on/off — not the SH-101's full amp.
