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
