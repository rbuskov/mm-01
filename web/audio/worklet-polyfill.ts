// AudioWorkletGlobalScope does not provide the Encoding API (TextDecoder /
// TextEncoder) — it inherits from WorkletGlobalScope, which omits it, and
// Chrome notably does not expose them. The wasm-bindgen glue in
// `../wasm/mm01_dsp.js` instantiates `new TextDecoder()` at module top-level,
// which throws a ReferenceError during evaluation — so the module never reaches
// its `registerProcessor("mm01", ...)` call, and constructing the
// AudioWorkletNode fails with "node name 'mm01' is not defined".
//
// This module installs a minimal UTF-8 polyfill onto the global scope BEFORE
// the glue runs (worklet.ts imports it first). It is a no-op anywhere the real
// Encoding API exists (main thread, tests), so it is safe to bundle everywhere.
//
// Scope: the only string crossing the wasm boundary here is wasm-bindgen error
// text (decode), and it is never on the audio render path, so a simple
// hand-rolled UTF-8 codec is sufficient. `fatal` is accepted but not enforced
// (invalid sequences decode leniently rather than throwing).

const g = globalThis as unknown as {
  TextDecoder?: unknown;
  TextEncoder?: unknown;
};

if (typeof g.TextDecoder === "undefined") {
  g.TextDecoder = class {
    readonly encoding = "utf-8";
    constructor(_label?: string, _options?: { fatal?: boolean; ignoreBOM?: boolean }) {}

    decode(input?: ArrayBuffer | ArrayBufferView): string {
      if (input == null) return "";
      const bytes =
        input instanceof Uint8Array
          ? input
          : new Uint8Array("buffer" in input ? input.buffer : input);
      let out = "";
      for (let i = 0; i < bytes.length; ) {
        const b1 = bytes[i++];
        if (b1 < 0x80) {
          out += String.fromCharCode(b1);
        } else if (b1 < 0xe0) {
          const b2 = bytes[i++] & 0x3f;
          out += String.fromCharCode(((b1 & 0x1f) << 6) | b2);
        } else if (b1 < 0xf0) {
          const b2 = bytes[i++] & 0x3f;
          const b3 = bytes[i++] & 0x3f;
          out += String.fromCharCode(((b1 & 0x0f) << 12) | (b2 << 6) | b3);
        } else {
          const b2 = bytes[i++] & 0x3f;
          const b3 = bytes[i++] & 0x3f;
          const b4 = bytes[i++] & 0x3f;
          let cp = ((b1 & 0x07) << 18) | (b2 << 12) | (b3 << 6) | b4;
          cp -= 0x10000;
          out += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3ff));
        }
      }
      return out;
    }
  };
}

if (typeof g.TextEncoder === "undefined") {
  g.TextEncoder = class {
    readonly encoding = "utf-8";

    encode(input = ""): Uint8Array {
      const bytes: number[] = [];
      for (let i = 0; i < input.length; i++) {
        let cp = input.charCodeAt(i);
        if (cp < 0x80) {
          bytes.push(cp);
        } else if (cp < 0x800) {
          bytes.push(0xc0 | (cp >> 6), 0x80 | (cp & 0x3f));
        } else if (cp >= 0xd800 && cp <= 0xdbff) {
          const lo = input.charCodeAt(++i);
          cp = 0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00);
          bytes.push(
            0xf0 | (cp >> 18),
            0x80 | ((cp >> 12) & 0x3f),
            0x80 | ((cp >> 6) & 0x3f),
            0x80 | (cp & 0x3f),
          );
        } else {
          bytes.push(0xe0 | (cp >> 12), 0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
        }
      }
      return new Uint8Array(bytes);
    }
  };
}
