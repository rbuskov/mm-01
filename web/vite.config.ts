import { defineConfig } from "vite";

const coiHeaders = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
  root: ".",
  server: { headers: coiHeaders },
  preview: { headers: coiHeaders },
  build: {
    target: "es2022",
  },
});
