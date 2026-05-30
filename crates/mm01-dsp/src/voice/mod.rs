mod eg;
mod lfo;
mod mixer;
mod noise;
mod vca;
mod vco;

use eg::Eg;
use lfo::Lfo;
use mixer::{clamp_level, Mixer};
use noise::Noise;
use vca::Vca;
use vco::Vco;

const STACK_CAP: usize = 16;

/// Map a normalised 0..1 control into a frequency/time range exponentially, so
/// the perceptual resolution is even across the (wide) span.
fn exp_map(norm: f32, min: f32, max: f32) -> f32 {
    let n = norm.clamp(0.0, 1.0);
    min * (max / min).powf(n)
}

pub struct Voice {
    vco: Vco,
    noise: Noise,
    mixer: Mixer,
    eg: Eg,
    lfo: Lfo,
    vca: Vca,
    volume: f32,
    held: [u8; STACK_CAP],
    held_len: usize,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Voice {
        Voice {
            vco: Vco::new(sample_rate),
            noise: Noise::new(),
            mixer: Mixer::new(),
            eg: Eg::new(sample_rate),
            lfo: Lfo::new(sample_rate),
            vca: Vca::new(),
            volume: 0.8,
            held: [0; STACK_CAP],
            held_len: 0,
        }
    }

    pub fn set_footage(&mut self, idx: u8) {
        self.vco.set_footage(idx);
    }

    pub fn set_sub_shape(&mut self, idx: u8) {
        self.vco.set_sub_shape(idx);
    }

    pub fn set_mix_saw(&mut self, level: f32) {
        self.mixer.saw = clamp_level(level);
    }

    pub fn set_mix_pulse(&mut self, level: f32) {
        self.mixer.pulse = clamp_level(level);
    }

    pub fn set_mix_sub(&mut self, level: f32) {
        self.mixer.sub = clamp_level(level);
    }

    pub fn set_mix_noise(&mut self, level: f32) {
        self.mixer.noise = clamp_level(level);
    }

    pub fn set_amp_source(&mut self, idx: u8) {
        self.vca.set_source(idx);
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }

    pub fn set_env_attack(&mut self, norm: f32) {
        self.eg.set_attack(exp_map(norm, 0.0015, 4.0));
    }

    pub fn set_env_decay(&mut self, norm: f32) {
        self.eg.set_decay(exp_map(norm, 0.002, 10.0));
    }

    pub fn set_env_sustain(&mut self, norm: f32) {
        self.eg.set_sustain(norm);
    }

    pub fn set_env_release(&mut self, norm: f32) {
        self.eg.set_release(exp_map(norm, 0.002, 10.0));
    }

    pub fn set_trigger_mode(&mut self, idx: u8) {
        self.eg.set_mode(idx);
    }

    pub fn set_lfo_rate(&mut self, norm: f32) {
        self.lfo.set_rate(exp_map(norm, 0.1, 30.0));
    }

    pub fn set_lfo_wave(&mut self, idx: u8) {
        self.lfo.set_wave(idx);
    }

    pub fn note_on(&mut self, note: u8) {
        // A gate rising edge is the first key after silence; otherwise legato.
        let rising = self.held_len == 0;
        self.remove(note);
        if self.held_len < STACK_CAP {
            self.held[self.held_len] = note;
            self.held_len += 1;
        } else {
            // Drop oldest to make room for newest.
            for i in 1..STACK_CAP {
                self.held[i - 1] = self.held[i];
            }
            self.held[STACK_CAP - 1] = note;
        }
        self.vco.set_note(note);
        self.eg.trigger(rising);
    }

    pub fn note_off(&mut self, note: u8) {
        self.remove(note);
        if self.held_len == 0 {
            self.eg.release();
        } else {
            self.vco.set_note(self.held[self.held_len - 1]);
        }
    }

    pub fn tick(&mut self) -> f32 {
        let osc = self.vco.tick();
        let noise = self.noise.tick();
        let mixed = self.mixer.mix(osc.saw, osc.pulse, osc.sub, noise);

        // The LFO's only job this iteration is to clock the envelope's LFO
        // trigger mode — one retrigger per cycle. It has no audio-rate path.
        let lfo = self.lfo.tick();
        if lfo.wrapped {
            self.eg.lfo_tick();
        }

        let env = self.eg.process();
        let gate = self.held_len > 0;
        self.vca.process(mixed, env, gate) * self.volume
    }

    fn remove(&mut self, note: u8) {
        let mut i = 0;
        while i < self.held_len {
            if self.held[i] == note {
                for j in i + 1..self.held_len {
                    self.held[j - 1] = self.held[j];
                }
                self.held_len -= 1;
                return;
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Voice;

    fn peak(voice: &mut Voice, samples: usize) -> f32 {
        (0..samples).map(|_| voice.tick().abs()).fold(0.0, f32::max)
    }

    #[test]
    fn silent_until_gated() {
        let mut v = Voice::new(48_000.0);
        assert_eq!(peak(&mut v, 256), 0.0);
    }

    #[test]
    fn sounds_on_note() {
        let mut v = Voice::new(48_000.0);
        v.note_on(60);
        assert!(peak(&mut v, 2_000) > 0.1, "note_on should produce signal");
    }

    #[test]
    fn envelope_release_fades_to_silence() {
        let mut v = Voice::new(48_000.0);
        v.note_on(60);
        let _ = peak(&mut v, 2_000);
        v.note_off(60);
        // Exponential release (~90 ms default); give it ~1 s to settle.
        let _ = peak(&mut v, 48_000);
        assert!(peak(&mut v, 512) < 1e-3, "envelope should fade to silence");
    }

    #[test]
    fn gate_amp_source_is_immediate() {
        let mut v = Voice::new(48_000.0);
        v.set_amp_source(1); // GATE
        v.note_on(60);
        assert!(peak(&mut v, 256) > 0.1);
        v.note_off(60);
        assert_eq!(
            peak(&mut v, 256),
            0.0,
            "gate amp should cut immediately on release"
        );
    }

    #[test]
    fn volume_scales_output() {
        fn measure(vol: f32) -> f32 {
            let mut v = Voice::new(48_000.0);
            v.set_amp_source(1); // gate → steady gain for a clean comparison
            v.set_volume(vol);
            v.note_on(60);
            peak(&mut v, 4_000)
        }
        let full = measure(1.0);
        let half = measure(0.5);
        assert!((half / full - 0.5).abs() < 0.05, "full={full} half={half}");
    }

    #[test]
    fn noise_channel_produces_signal_without_oscillator() {
        let mut v = Voice::new(48_000.0);
        v.set_mix_saw(0.0);
        v.set_mix_noise(1.0);
        v.note_on(60);
        assert!(peak(&mut v, 2_000) > 0.1, "noise channel should be audible");
    }
}
