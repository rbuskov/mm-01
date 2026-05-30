mod mixer;
mod noise;
mod vca;
mod vco;

use mixer::{clamp_level, Mixer};
use noise::Noise;
use vca::Vca;
use vco::Vco;

const STACK_CAP: usize = 16;

pub struct Voice {
    vco: Vco,
    noise: Noise,
    mixer: Mixer,
    vca: Vca,
    held: [u8; STACK_CAP],
    held_len: usize,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Voice {
        Voice {
            vco: Vco::new(sample_rate),
            noise: Noise::new(),
            mixer: Mixer::new(),
            vca: Vca::new(),
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

    pub fn note_on(&mut self, note: u8) {
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
        self.vca.set_gate(true);
    }

    pub fn note_off(&mut self, note: u8) {
        self.remove(note);
        if self.held_len == 0 {
            self.vca.set_gate(false);
        } else {
            self.vco.set_note(self.held[self.held_len - 1]);
        }
    }

    pub fn tick(&mut self) -> f32 {
        let osc = self.vco.tick();
        let noise = self.noise.tick();
        let mixed = self.mixer.mix(osc.saw, osc.pulse, osc.sub, noise);
        self.vca.process(mixed)
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
    fn sounds_on_note_and_stops_on_release() {
        let mut v = Voice::new(48_000.0);
        v.note_on(60);
        assert!(peak(&mut v, 2_000) > 0.1, "note_on should produce signal");
        v.note_off(60);
        assert_eq!(peak(&mut v, 512), 0.0, "release should silence the voice");
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
