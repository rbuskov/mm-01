pub struct Vco {
    sample_rate: f32,
    phase: f32,
    increment: f32,
}

impl Vco {
    pub fn new(sample_rate: f32) -> Vco {
        Vco {
            sample_rate,
            phase: 0.0,
            increment: 0.0,
        }
    }

    pub fn set_note(&mut self, midi_note: u8) {
        let freq = 440.0 * (2.0_f32).powf((midi_note as f32 - 69.0) / 12.0);
        self.increment = freq / self.sample_rate;
    }

    pub fn tick(&mut self) -> f32 {
        // Naive sawtooth, range -1..1. Aliasing is acceptable for iteration 1;
        // band-limiting (polyblep) lands with the full voice port.
        let s = 2.0 * self.phase - 1.0;
        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        s
    }
}
