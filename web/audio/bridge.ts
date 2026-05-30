// The worklet is pre-bundled by esbuild into a self-contained classic script
// at `public/mm01-worklet.js` (see the `build:worklet` npm script). It must be
// a classic script with no `import` statements: AudioWorkletGlobalScope cannot
// resolve ES module imports, so anything Vite's `?worker&url` serves in dev
// (raw ESM with an injected `vite/.../env.mjs` import) fails to register the
// processor. Loading the pre-bundled file works identically in dev and build.
const workletUrl = `${import.meta.env.BASE_URL}mm01-worklet.js`;

export const TAG_NOTE_ON = 0;
export const TAG_NOTE_OFF = 1;
export const TAG_PARAM_SET = 2;

// ParamSet IDs — must match crates/mm01-dsp/src/msg.rs.
export const PARAM_MASTER_GAIN = 0;
export const PARAM_FOOTAGE = 1; // 0→16′, 1→8′, 2→4′, 3→2′
export const PARAM_SUB_SHAPE = 2; // 0→sq −1, 1→sq −2, 2→pulse −2
export const PARAM_MIX_SAW = 3;
export const PARAM_MIX_PULSE = 4;
export const PARAM_MIX_SUB = 5;
export const PARAM_MIX_NOISE = 6;
export const PARAM_AMP_SOURCE = 7; // 0→ENV, 1→GATE
export const PARAM_VOLUME = 8;
export const PARAM_ENV_ATTACK = 9; // normalised 0..1
export const PARAM_ENV_DECAY = 10;
export const PARAM_ENV_SUSTAIN = 11;
export const PARAM_ENV_RELEASE = 12;
export const PARAM_ENV_TRIGGER_MODE = 13; // 0→GATE+TRIG, 1→GATE, 2→LFO
export const PARAM_LFO_RATE = 14; // normalised 0..1
export const PARAM_LFO_WAVE = 15; // 0→tri, 1→square, 2→random, 3→noise

export function encodeNoteOn(note: number, velocity = 100): Uint8Array {
  return new Uint8Array([TAG_NOTE_ON, note & 0x7f, velocity & 0x7f]);
}

export function encodeNoteOff(note: number): Uint8Array {
  return new Uint8Array([TAG_NOTE_OFF, note & 0x7f]);
}

export function encodeParamSet(id: number, value: number): Uint8Array {
  const buf = new ArrayBuffer(6);
  const view = new DataView(buf);
  view.setUint8(0, TAG_PARAM_SET);
  view.setUint8(1, id & 0xff);
  view.setFloat32(2, value, true);
  return new Uint8Array(buf);
}

export class Bridge {
  private constructor(
    private ctx: AudioContext,
    private node: AudioWorkletNode,
  ) {}

  static async start(): Promise<Bridge> {
    const ctx = new AudioContext({ latencyHint: "interactive" });
    if (ctx.state === "suspended") await ctx.resume();

    const wasmResp = await fetch(new URL("../wasm/mm01_dsp_bg.wasm", import.meta.url));
    const wasmBytes = await wasmResp.arrayBuffer();
    const wasmModule = await WebAssembly.compile(wasmBytes);

    await ctx.audioWorklet.addModule(workletUrl);

    const node = new AudioWorkletNode(ctx, "mm01", {
      numberOfInputs: 0,
      numberOfOutputs: 1,
      outputChannelCount: [2],
      processorOptions: { wasmModule },
    });

    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("worklet did not become ready")), 5000);
      node.port.onmessage = (e) => {
        if (e.data?.type === "ready") {
          clearTimeout(timer);
          resolve();
        }
      };
    });

    node.connect(ctx.destination);
    return new Bridge(ctx, node);
  }

  send(bytes: Uint8Array): void {
    this.node.port.postMessage(bytes, [bytes.buffer]);
  }

  noteOn(note: number, velocity = 100): void {
    this.send(encodeNoteOn(note, velocity));
  }

  noteOff(note: number): void {
    this.send(encodeNoteOff(note));
  }

  paramSet(id: number, value: number): void {
    this.send(encodeParamSet(id, value));
  }

  get audioContext(): AudioContext {
    return this.ctx;
  }
}
