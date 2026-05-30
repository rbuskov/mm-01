import type { Bridge } from "../audio/bridge";
import {
  PARAM_FOOTAGE,
  PARAM_SUB_SHAPE,
  PARAM_MIX_SAW,
  PARAM_MIX_PULSE,
  PARAM_MIX_SUB,
  PARAM_MIX_NOISE,
} from "../audio/bridge";

const FOOTAGE_OPTIONS = ["16′", "8′", "4′", "2′"];
const FOOTAGE_DEFAULT = 1; // 8′ (nominal)

const SUB_OPTIONS = ["Sq −1", "Sq −2", "Pulse −2"];
const SUB_DEFAULT = 0;

interface MixerSpec {
  label: string;
  param: number;
  default: number;
}

// Default to saw-only at full level — matches the engine's defaults and the
// iteration-1 timbre.
const MIXER: MixerSpec[] = [
  { label: "Saw", param: PARAM_MIX_SAW, default: 1 },
  { label: "Pulse", param: PARAM_MIX_PULSE, default: 0 },
  { label: "Sub", param: PARAM_MIX_SUB, default: 0 },
  { label: "Noise", param: PARAM_MIX_NOISE, default: 0 },
];

/// Build a labelled segmented selector (radio group). Calls `onSelect` with the
/// chosen index, and selects `initial` up front.
function segmented(
  title: string,
  options: string[],
  initial: number,
  onSelect: (index: number) => void,
): HTMLElement {
  const section = document.createElement("div");
  section.className = "panel-section";

  const heading = document.createElement("span");
  heading.className = "panel-label";
  heading.textContent = title;
  section.appendChild(heading);

  const group = document.createElement("div");
  group.className = "segmented";
  group.setAttribute("role", "radiogroup");
  group.setAttribute("aria-label", title);

  const buttons: HTMLButtonElement[] = [];
  options.forEach((label, index) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "seg-btn";
    btn.textContent = label;
    btn.setAttribute("role", "radio");
    const select = () => {
      buttons.forEach((b, i) => {
        const on = i === index;
        b.classList.toggle("active", on);
        b.setAttribute("aria-checked", String(on));
      });
      onSelect(index);
    };
    btn.addEventListener("click", select);
    buttons.push(btn);
    group.appendChild(btn);
  });

  // Apply initial selection.
  buttons[initial]?.click();
  section.appendChild(group);
  return section;
}

/// Build the four-channel source mixer (vertical sliders).
function mixer(bridge: Bridge): HTMLElement {
  const section = document.createElement("div");
  section.className = "panel-section";

  const heading = document.createElement("span");
  heading.className = "panel-label";
  heading.textContent = "Source mixer";
  section.appendChild(heading);

  const row = document.createElement("div");
  row.className = "mixer";

  for (const spec of MIXER) {
    const channel = document.createElement("label");
    channel.className = "mixer-channel";

    const slider = document.createElement("input");
    slider.type = "range";
    slider.min = "0";
    slider.max = "1";
    slider.step = "0.01";
    slider.value = String(spec.default);
    slider.className = "mixer-slider";
    slider.setAttribute("aria-label", `${spec.label} level`);
    slider.addEventListener("input", () => {
      bridge.paramSet(spec.param, slider.valueAsNumber);
    });

    const name = document.createElement("span");
    name.className = "mixer-name";
    name.textContent = spec.label;

    channel.append(slider, name);
    row.appendChild(channel);

    // Push the default into the engine so UI and DSP start in sync.
    bridge.paramSet(spec.param, spec.default);
  }

  section.appendChild(row);
  return section;
}

export function mountPanel(host: HTMLElement, bridge: Bridge): void {
  host.innerHTML = "";
  host.classList.add("panel-controls");

  host.appendChild(
    segmented("Footage", FOOTAGE_OPTIONS, FOOTAGE_DEFAULT, (i) =>
      bridge.paramSet(PARAM_FOOTAGE, i),
    ),
  );
  host.appendChild(
    segmented("Sub osc", SUB_OPTIONS, SUB_DEFAULT, (i) =>
      bridge.paramSet(PARAM_SUB_SHAPE, i),
    ),
  );
  host.appendChild(mixer(bridge));
}
