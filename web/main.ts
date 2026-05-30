import { Bridge } from "./audio/bridge";
import { mountKeyboard } from "./ui/keyboard";
import { mountPanel } from "./ui/panel";

const startBtn = document.getElementById("start") as HTMLButtonElement;
const panelHost = document.getElementById("panel") as HTMLElement;
const keyboardHost = document.getElementById("keyboard-host") as HTMLElement;

let bridge: Bridge | null = null;

startBtn.addEventListener("click", async () => {
  if (bridge) return;
  startBtn.disabled = true;
  startBtn.textContent = "Loading…";
  try {
    bridge = await Bridge.start();
    startBtn.textContent = "Running";
    mountPanel(panelHost, bridge);
    mountKeyboard(keyboardHost, bridge);
  } catch (err) {
    startBtn.disabled = false;
    startBtn.textContent = "Start Audio";
    console.error(err);
    alert(`Failed to start audio: ${(err as Error).message}`);
  }
});
