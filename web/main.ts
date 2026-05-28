import { Bridge } from "./audio/bridge";
import { mountKeyboard } from "./ui/keyboard";

const startBtn = document.getElementById("start") as HTMLButtonElement;
const keyboardHost = document.getElementById("keyboard-host") as HTMLElement;

let bridge: Bridge | null = null;

startBtn.addEventListener("click", async () => {
  if (bridge) return;
  startBtn.disabled = true;
  startBtn.textContent = "Loading…";
  try {
    bridge = await Bridge.start();
    startBtn.textContent = "Running";
    mountKeyboard(keyboardHost, bridge);
  } catch (err) {
    startBtn.disabled = false;
    startBtn.textContent = "Start Audio";
    console.error(err);
    alert(`Failed to start audio: ${(err as Error).message}`);
  }
});
