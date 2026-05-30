import type { Bridge } from "../audio/bridge";
import {
  PARAM_FOOTAGE,
  PARAM_SUB_SHAPE,
  PARAM_MIX_SAW,
  PARAM_MIX_PULSE,
  PARAM_MIX_SUB,
  PARAM_MIX_NOISE,
  PARAM_AMP_SOURCE,
  PARAM_VOLUME,
  PARAM_ENV_ATTACK,
  PARAM_ENV_DECAY,
  PARAM_ENV_SUSTAIN,
  PARAM_ENV_RELEASE,
  PARAM_ENV_TRIGGER_MODE,
  PARAM_LFO_RATE,
  PARAM_LFO_WAVE,
} from "../audio/bridge";

/// A labelled panel section (heading + arbitrary control elements).
function section(title: string, ...controls: HTMLElement[]): HTMLElement {
  const sec = document.createElement("div");
  sec.className = "panel-section";
  const heading = document.createElement("span");
  heading.className = "panel-label";
  heading.textContent = title;
  sec.appendChild(heading);
  controls.forEach((c) => sec.appendChild(c));
  return sec;
}

/// A segmented selector (radio group). Selects `initial` up front, which also
/// pushes the initial value to the engine via `onSelect`.
function segmentedControl(
  ariaLabel: string,
  options: string[],
  initial: number,
  onSelect: (index: number) => void,
): HTMLElement {
  const group = document.createElement("div");
  group.className = "segmented";
  group.setAttribute("role", "radiogroup");
  group.setAttribute("aria-label", ariaLabel);

  const buttons: HTMLButtonElement[] = [];
  options.forEach((label, index) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "seg-btn";
    btn.textContent = label;
    btn.setAttribute("role", "radio");
    btn.addEventListener("click", () => {
      buttons.forEach((b, i) => {
        const on = i === index;
        b.classList.toggle("active", on);
        b.setAttribute("aria-checked", String(on));
      });
      onSelect(index);
    });
    buttons.push(btn);
    group.appendChild(btn);
  });

  buttons[initial]?.click();
  return group;
}

/// A labelled horizontal slider sending a normalised 0..1 value to `param`.
function hSlider(
  label: string,
  param: number,
  initial: number,
  bridge: Bridge,
): HTMLElement {
  const row = document.createElement("label");
  row.className = "slider-row";

  const name = document.createElement("span");
  name.className = "slider-name";
  name.textContent = label;

  const input = document.createElement("input");
  input.type = "range";
  input.min = "0";
  input.max = "1";
  input.step = "0.01";
  input.value = String(initial);
  input.className = "h-slider";
  input.setAttribute("aria-label", label);
  input.addEventListener("input", () =>
    bridge.paramSet(param, input.valueAsNumber),
  );

  row.append(name, input);
  bridge.paramSet(param, initial); // start engine in sync with the UI
  return row;
}

interface MixerSpec {
  label: string;
  param: number;
  default: number;
}

// Default to saw-only at full level — matches the engine and iteration-1 timbre.
const MIXER: MixerSpec[] = [
  { label: "Saw", param: PARAM_MIX_SAW, default: 1 },
  { label: "Pulse", param: PARAM_MIX_PULSE, default: 0 },
  { label: "Sub", param: PARAM_MIX_SUB, default: 0 },
  { label: "Noise", param: PARAM_MIX_NOISE, default: 0 },
];

/// The four-channel source mixer (vertical sliders).
function mixer(bridge: Bridge): HTMLElement {
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
    slider.addEventListener("input", () =>
      bridge.paramSet(spec.param, slider.valueAsNumber),
    );

    const name = document.createElement("span");
    name.className = "mixer-name";
    name.textContent = spec.label;

    channel.append(slider, name);
    row.appendChild(channel);
    bridge.paramSet(spec.param, spec.default);
  }
  return row;
}

export function mountPanel(host: HTMLElement, bridge: Bridge): void {
  host.innerHTML = "";
  host.classList.add("panel-controls");

  host.appendChild(
    section(
      "Footage",
      segmentedControl("Footage", ["16′", "8′", "4′", "2′"], 1, (i) =>
        bridge.paramSet(PARAM_FOOTAGE, i),
      ),
    ),
  );

  host.appendChild(
    section(
      "Sub osc",
      segmentedControl("Sub osc", ["Sq −1", "Sq −2", "Pulse −2"], 0, (i) =>
        bridge.paramSet(PARAM_SUB_SHAPE, i),
      ),
    ),
  );

  host.appendChild(section("Source mixer", mixer(bridge)));

  host.appendChild(
    section(
      "Amp",
      segmentedControl("Amp source", ["ENV", "GATE"], 0, (i) =>
        bridge.paramSet(PARAM_AMP_SOURCE, i),
      ),
      hSlider("Volume", PARAM_VOLUME, 0.8, bridge),
    ),
  );

  host.appendChild(
    section(
      "Envelope",
      hSlider("A", PARAM_ENV_ATTACK, 0.15, bridge),
      hSlider("D", PARAM_ENV_DECAY, 0.5, bridge),
      hSlider("S", PARAM_ENV_SUSTAIN, 0.8, bridge),
      hSlider("R", PARAM_ENV_RELEASE, 0.45, bridge),
      segmentedControl(
        "Trigger mode",
        ["Gate+Trig", "Gate", "LFO"],
        0,
        (i) => bridge.paramSet(PARAM_ENV_TRIGGER_MODE, i),
      ),
    ),
  );

  host.appendChild(
    section(
      "LFO",
      hSlider("Rate", PARAM_LFO_RATE, 0.6, bridge),
      segmentedControl("LFO waveform", ["Tri", "Sqr", "Rnd", "Noise"], 0, (i) =>
        bridge.paramSet(PARAM_LFO_WAVE, i),
      ),
    ),
  );
}
