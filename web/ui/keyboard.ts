import type { Bridge } from "../audio/bridge";

const LOW = 36;   // C2
const HIGH = 67;  // G4

const BLACK_SET = new Set([1, 3, 6, 8, 10]);
const isBlack = (note: number) => BLACK_SET.has(((note % 12) + 12) % 12);

// Lower row: Z S X D C V G B H N J M , — semitones 0..12 from base
const LOWER_ROW = ["KeyZ", "KeyS", "KeyX", "KeyD", "KeyC", "KeyV", "KeyG", "KeyB", "KeyH", "KeyN", "KeyJ", "KeyM", "Comma"];
// Upper row: Q 2 W 3 E R 5 T 6 Y 7 U I — semitones 12..24 from base
const UPPER_ROW = ["KeyQ", "Digit2", "KeyW", "Digit3", "KeyE", "KeyR", "Digit5", "KeyT", "Digit6", "KeyY", "Digit7", "KeyU", "KeyI"];

const COMPUTER_KEY_MAP: Map<string, number> = new Map();
LOWER_ROW.forEach((code, i) => COMPUTER_KEY_MAP.set(code, i));
UPPER_ROW.forEach((code, i) => COMPUTER_KEY_MAP.set(code, i + 12));

const DEFAULT_BASE = 48; // C3
const MIN_BASE = 24;     // C1 — allows reaching down to C2 via lower row
const MAX_BASE = 60;     // C5 — allows G4 from lower row's G semitone (offset 7)

interface KeyboardState {
  base: number;
  keyEls: Map<number, HTMLElement>;
  held: Set<number>;             // notes currently sounding
  mouseDown: boolean;
  mouseNote: number | null;
  computerHeld: Map<string, number>; // physical key code → MIDI note it triggered
  baseLabel: HTMLElement;
}

export function mountKeyboard(host: HTMLElement, bridge: Bridge): void {
  host.innerHTML = "";

  const controls = document.createElement("div");
  controls.className = "keyboard-controls";
  const downBtn = document.createElement("button");
  downBtn.textContent = "Octave −";
  const upBtn = document.createElement("button");
  upBtn.textContent = "Octave +";
  const baseLabel = document.createElement("span");
  baseLabel.className = "octave-label";
  controls.append(downBtn, upBtn, baseLabel);

  const kb = document.createElement("div");
  kb.className = "keyboard";

  host.append(controls, kb);

  const state: KeyboardState = {
    base: DEFAULT_BASE,
    keyEls: new Map(),
    held: new Set(),
    mouseDown: false,
    mouseNote: null,
    computerHeld: new Map(),
    baseLabel,
  };

  renderKeys(kb, state, bridge);
  updateBaseLabel(state);

  downBtn.addEventListener("click", () => shiftBase(state, -12, bridge));
  upBtn.addEventListener("click", () => shiftBase(state, +12, bridge));

  window.addEventListener("mouseup", () => endMouseNote(state, bridge));
  window.addEventListener("blur", () => releaseAll(state, bridge));

  window.addEventListener("keydown", (e) => {
    if (e.repeat) return;
    if (e.code === "Minus") { shiftBase(state, -12, bridge); e.preventDefault(); return; }
    if (e.code === "Equal") { shiftBase(state, +12, bridge); e.preventDefault(); return; }
    const offset = COMPUTER_KEY_MAP.get(e.code);
    if (offset === undefined) return;
    const note = state.base + offset;
    if (note < LOW || note > HIGH) return;
    if (state.computerHeld.has(e.code)) return;
    state.computerHeld.set(e.code, note);
    pressNote(state, bridge, note);
    e.preventDefault();
  });

  window.addEventListener("keyup", (e) => {
    const note = state.computerHeld.get(e.code);
    if (note === undefined) return;
    state.computerHeld.delete(e.code);
    releaseNote(state, bridge, note);
    e.preventDefault();
  });
}

function renderKeys(kb: HTMLElement, state: KeyboardState, bridge: Bridge): void {
  kb.innerHTML = "";
  state.keyEls.clear();

  const whiteNotes: number[] = [];
  for (let n = LOW; n <= HIGH; n++) if (!isBlack(n)) whiteNotes.push(n);

  for (const n of whiteNotes) {
    const el = document.createElement("div");
    el.className = "key white";
    el.dataset.note = String(n);
    kb.append(el);
    state.keyEls.set(n, el);
    attachPointerHandlers(el, n, state, bridge);
  }

  // Black keys positioned between adjacent whites by percentage.
  const totalWhite = whiteNotes.length;
  for (let n = LOW; n <= HIGH; n++) {
    if (!isBlack(n)) continue;
    // Find the white key immediately below (n-1 will be white for black notes).
    const whiteBelowIdx = whiteNotes.indexOf(n - 1);
    if (whiteBelowIdx < 0) continue;
    const el = document.createElement("div");
    el.className = "key black";
    el.dataset.note = String(n);
    el.style.left = `calc(${((whiteBelowIdx + 1) / totalWhite) * 100}% - 1.2%)`;
    kb.append(el);
    state.keyEls.set(n, el);
    attachPointerHandlers(el, n, state, bridge);
  }
}

function attachPointerHandlers(el: HTMLElement, note: number, state: KeyboardState, bridge: Bridge): void {
  el.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    state.mouseDown = true;
    state.mouseNote = note;
    pressNote(state, bridge, note);
    e.preventDefault();
  });
  el.addEventListener("mouseenter", () => {
    if (!state.mouseDown) return;
    if (state.mouseNote === note) return;
    if (state.mouseNote !== null) releaseNote(state, bridge, state.mouseNote);
    state.mouseNote = note;
    pressNote(state, bridge, note);
  });
  el.addEventListener("mouseleave", () => {
    if (state.mouseDown && state.mouseNote === note) {
      releaseNote(state, bridge, note);
      state.mouseNote = null;
    }
  });
  el.addEventListener("touchstart", (e) => {
    state.mouseDown = true;
    state.mouseNote = note;
    pressNote(state, bridge, note);
    e.preventDefault();
  }, { passive: false });
  el.addEventListener("touchend", (e) => {
    releaseNote(state, bridge, note);
    if (state.mouseNote === note) state.mouseNote = null;
    state.mouseDown = false;
    e.preventDefault();
  });
}

function endMouseNote(state: KeyboardState, bridge: Bridge): void {
  if (state.mouseNote !== null) releaseNote(state, bridge, state.mouseNote);
  state.mouseNote = null;
  state.mouseDown = false;
}

function pressNote(state: KeyboardState, bridge: Bridge, note: number): void {
  state.held.add(note);
  state.keyEls.get(note)?.classList.add("active");
  bridge.noteOn(note);
}

function releaseNote(state: KeyboardState, bridge: Bridge, note: number): void {
  if (!state.held.has(note)) return;
  state.held.delete(note);
  state.keyEls.get(note)?.classList.remove("active");
  bridge.noteOff(note);
}

function releaseAll(state: KeyboardState, bridge: Bridge): void {
  for (const note of [...state.held]) releaseNote(state, bridge, note);
  state.computerHeld.clear();
  state.mouseDown = false;
  state.mouseNote = null;
}

function shiftBase(state: KeyboardState, delta: number, bridge: Bridge): void {
  releaseAll(state, bridge);
  state.base = Math.max(MIN_BASE, Math.min(MAX_BASE, state.base + delta));
  updateBaseLabel(state);
}

function updateBaseLabel(state: KeyboardState): void {
  state.baseLabel.textContent = `Computer-keyboard base: ${noteName(state.base)}`;
}

function noteName(midi: number): string {
  const names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const octave = Math.floor(midi / 12) - 1;
  return `${names[midi % 12]}${octave}`;
}
