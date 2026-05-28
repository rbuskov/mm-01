// `?worker&url` makes Vite emit a bundled, self-contained ES module file for
// the worklet (deps inlined) and gives us its URL. AudioWorklet just needs an
// ES-module URL; the "worker" label is incidental.
import workletUrl from "./worklet.ts?worker&url";

export const TAG_NOTE_ON = 0;
export const TAG_NOTE_OFF = 1;
export const TAG_PARAM_SET = 2;

export const PARAM_MASTER_GAIN = 0;

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
