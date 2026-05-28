/// <reference path="./worklet-types.d.ts" />
import init, { Engine } from "../wasm/mm01_dsp.js";

class MM01Processor extends AudioWorkletProcessor {
  private engine: Engine | null = null;

  constructor(options?: AudioWorkletNodeOptions) {
    super();
    const { wasmModule } = (options?.processorOptions ?? {}) as {
      wasmModule?: WebAssembly.Module;
    };
    if (!wasmModule) {
      throw new Error("worklet: wasmModule missing in processorOptions");
    }
    init({ module_or_path: wasmModule }).then(() => {
      this.engine = new Engine(sampleRate);
      this.port.postMessage({ type: "ready" });
    });
    this.port.onmessage = (e) => {
      const data = e.data;
      if (this.engine && data instanceof Uint8Array) {
        this.engine.handle_message(data);
      }
    };
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    if (!this.engine) return true;
    const out = outputs[0];
    const left = out[0];
    const right = out[1] ?? out[0];
    this.engine.process(left, right);
    return true;
  }
}

registerProcessor("mm01", MM01Processor);
