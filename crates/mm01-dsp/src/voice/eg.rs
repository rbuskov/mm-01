//! Four-stage ADSR envelope with exponential per-stage curves.
//!
//! Port of the well-known "earlevel" exponential ADSR design: each stage is a
//! one-pole approach toward a target offset by a curvature ratio, so the ramps
//! are exponential rather than linear (audibly important at the short end of
//! the time ranges). The attack uses a gentler curve than decay/release.

// Curvature ratios: larger = more linear, smaller = more sharply exponential.
// These also set what level the nominal stage time reaches: the curve covers
// the start→target span to within `ratio` over the stage time. A very small DR
// ratio (e.g. 0.0001 → −80 dB) front-loads almost all the audible change into
// the first ~half of the nominal time, so decay/release sound roughly twice as
// fast as the knob says and overly "plinky". 0.02 keeps a clearly exponential
// shape while making the audible decay/release time track the nominal time
// (release reaches silence at ~95% of nominal).
const TARGET_RATIO_A: f32 = 0.3;
const TARGET_RATIO_DR: f32 = 0.02;

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// How new key events interact with the envelope.
#[derive(Clone, Copy)]
pub enum TriggerMode {
    /// Every key press restarts from Attack, including legato.
    GateTrig,
    /// Starts on the first key, sustains across legato; restarts only after a
    /// full release followed by a new press.
    Gate,
    /// While a key is held, retriggered once per LFO cycle.
    Lfo,
}

impl TriggerMode {
    fn from_index(idx: u8) -> TriggerMode {
        match idx {
            1 => TriggerMode::Gate,
            2 => TriggerMode::Lfo,
            _ => TriggerMode::GateTrig,
        }
    }
}

pub struct Eg {
    sample_rate: f32,
    stage: Stage,
    output: f32,
    mode: TriggerMode,
    gate: bool,

    attack_time: f32, // seconds
    decay_time: f32,
    sustain: f32, // 0..1
    release_time: f32,

    attack_coef: f32,
    attack_base: f32,
    decay_coef: f32,
    decay_base: f32,
    release_coef: f32,
    release_base: f32,
}

fn calc_coef(rate_samples: f32, target_ratio: f32) -> f32 {
    if rate_samples <= 0.0 {
        return 0.0;
    }
    (-((1.0 + target_ratio) / target_ratio).ln() / rate_samples).exp()
}

impl Eg {
    pub fn new(sample_rate: f32) -> Eg {
        let mut eg = Eg {
            sample_rate,
            stage: Stage::Idle,
            output: 0.0,
            mode: TriggerMode::GateTrig,
            gate: false,
            attack_time: 0.005,
            decay_time: 0.14,
            sustain: 0.8,
            release_time: 0.09,
            attack_coef: 0.0,
            attack_base: 0.0,
            decay_coef: 0.0,
            decay_base: 0.0,
            release_coef: 0.0,
            release_base: 0.0,
        };
        eg.recompute();
        eg
    }

    pub fn set_attack(&mut self, seconds: f32) {
        self.attack_time = seconds.max(0.0);
        self.recompute();
    }

    pub fn set_decay(&mut self, seconds: f32) {
        self.decay_time = seconds.max(0.0);
        self.recompute();
    }

    pub fn set_sustain(&mut self, level: f32) {
        self.sustain = level.clamp(0.0, 1.0);
        self.recompute();
    }

    pub fn set_release(&mut self, seconds: f32) {
        self.release_time = seconds.max(0.0);
        self.recompute();
    }

    pub fn set_mode(&mut self, idx: u8) {
        self.mode = TriggerMode::from_index(idx);
    }

    fn recompute(&mut self) {
        let sr = self.sample_rate;
        self.attack_coef = calc_coef(self.attack_time * sr, TARGET_RATIO_A);
        self.attack_base = (1.0 + TARGET_RATIO_A) * (1.0 - self.attack_coef);
        self.decay_coef = calc_coef(self.decay_time * sr, TARGET_RATIO_DR);
        self.decay_base = (self.sustain - TARGET_RATIO_DR) * (1.0 - self.decay_coef);
        self.release_coef = calc_coef(self.release_time * sr, TARGET_RATIO_DR);
        self.release_base = (0.0 - TARGET_RATIO_DR) * (1.0 - self.release_coef);
    }

    /// A key was pressed. `rising` is true when this is the first held key
    /// (a gate rising edge), false for an overlapping/legato press.
    pub fn trigger(&mut self, rising: bool) {
        self.gate = true;
        match self.mode {
            TriggerMode::GateTrig => self.stage = Stage::Attack,
            TriggerMode::Gate | TriggerMode::Lfo => {
                if rising {
                    self.stage = Stage::Attack;
                }
            }
        }
    }

    /// The last held key was released.
    pub fn release(&mut self) {
        self.gate = false;
        self.stage = Stage::Release;
    }

    /// Called once per LFO cycle; retriggers the envelope in LFO mode while held.
    pub fn lfo_tick(&mut self) {
        if matches!(self.mode, TriggerMode::Lfo) && self.gate {
            self.stage = Stage::Attack;
        }
    }

    pub fn process(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => {}
            Stage::Attack => {
                self.output = self.attack_base + self.output * self.attack_coef;
                if self.output >= 1.0 {
                    self.output = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.output = self.decay_base + self.output * self.decay_coef;
                if self.output <= self.sustain {
                    self.output = self.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {
                self.output = self.sustain;
            }
            Stage::Release => {
                self.output = self.release_base + self.output * self.release_coef;
                if self.output <= 0.0 {
                    self.output = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::Eg;

    fn run(eg: &mut Eg, n: usize) -> f32 {
        let mut last = 0.0;
        for _ in 0..n {
            last = eg.process();
        }
        last
    }

    fn peak(eg: &mut Eg, n: usize) -> f32 {
        (0..n).map(|_| eg.process()).fold(0.0, f32::max)
    }

    #[test]
    fn reaches_sustain_then_releases_to_zero() {
        let mut eg = Eg::new(48_000.0);
        eg.set_attack(0.001);
        eg.set_decay(0.001);
        eg.set_sustain(0.5);
        eg.set_release(0.001);
        eg.trigger(true);
        // After attack+decay settle, output should sit at the sustain level.
        let level = run(&mut eg, 2_000);
        assert!((level - 0.5).abs() < 0.02, "sustain level={level}");
        eg.release();
        let level = run(&mut eg, 2_000);
        assert!(level < 1e-3, "released level={level}");
    }

    #[test]
    fn gate_mode_does_not_retrigger_on_legato() {
        let mut eg = Eg::new(48_000.0);
        eg.set_mode(1); // GATE
        eg.set_attack(1.0); // slow attack so a restart would be visible
        eg.trigger(true); // first key
        let after_first = run(&mut eg, 480);
        eg.trigger(false); // legato press — must NOT restart
        let after_legato = eg.process();
        assert!(
            after_legato >= after_first,
            "legato press must not reset attack ({after_first} -> {after_legato})"
        );
    }

    #[test]
    fn lfo_tick_retriggers_while_held() {
        let mut eg = Eg::new(48_000.0);
        eg.set_mode(2); // LFO
        eg.set_attack(0.001);
        eg.set_decay(0.001);
        eg.set_sustain(0.0); // collapses to ~0 after decay
        eg.trigger(true);
        let low = run(&mut eg, 2_000);
        assert!(low < 0.05, "should have decayed near zero: {low}");
        eg.lfo_tick(); // retrigger
        // Measure the peak of the new transient (it re-attacks then collapses).
        let high = peak(&mut eg, 200);
        assert!(high > 0.3, "lfo tick should restart attack: {high}");
    }
}
