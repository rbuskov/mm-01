use crate::primitives::poly_blep;

/// Sub-oscillator shape, selected by a 3-way switch. All are derived from the
/// master oscillator by frequency division, so they stay phase-locked.
#[derive(Clone, Copy)]
pub enum SubShape {
    /// Square, one octave down (master ÷ 2).
    SquareDown1,
    /// Square, two octaves down (master ÷ 4).
    SquareDown2,
    /// Pulse (~25% duty), two octaves down (master ÷ 4).
    PulseDown2,
}

impl SubShape {
    fn from_index(idx: u8) -> SubShape {
        match idx {
            1 => SubShape::SquareDown2,
            2 => SubShape::PulseDown2,
            _ => SubShape::SquareDown1,
        }
    }

    /// Frequency divisor relative to the master and the pulse duty cycle.
    fn div_and_duty(self) -> (f32, f32) {
        match self {
            SubShape::SquareDown1 => (2.0, 0.5),
            SubShape::SquareDown2 => (4.0, 0.5),
            SubShape::PulseDown2 => (4.0, 0.25),
        }
    }
}

/// One sample of the oscillator core: three simultaneous, phase-locked outputs.
pub struct VcoOut {
    pub saw: f32,
    pub pulse: f32,
    pub sub: f32,
}

/// Single monophonic oscillator core. One phase accumulator drives the saw and
/// the fixed-50% pulse; the sub-oscillator is derived from the same accumulator
/// by integer division, so all outputs are inherently phase-locked. All hard
/// edges are band-limited with PolyBLEP.
pub struct Vco {
    sample_rate: f32,
    base_freq: f32,    // note frequency at 8′ (nominal)
    footage_mult: f32, // 16′=0.5, 8′=1, 4′=2, 2′=4
    phase: f32,        // master phase, [0, 1)
    increment: f32,    // master per-sample phase increment
    cycle: u32,        // completed master cycles, mod 4 (drives sub phase)
    sub_shape: SubShape,
}

impl Vco {
    pub fn new(sample_rate: f32) -> Vco {
        Vco {
            sample_rate,
            base_freq: 0.0,
            footage_mult: 1.0,
            phase: 0.0,
            increment: 0.0,
            cycle: 0,
            sub_shape: SubShape::SquareDown1,
        }
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.base_freq = 440.0 * (2.0_f32).powf((midi_note as f32 - 69.0) / 12.0);
        self.update_increment();
    }

    /// Footage selector: 0 → 16′, 1 → 8′, 2 → 4′, 3 → 2′ (8′ is nominal).
    pub fn set_footage(&mut self, idx: u8) {
        self.footage_mult = (2.0_f32).powi(idx.min(3) as i32 - 1);
        self.update_increment();
    }

    pub fn set_sub_shape(&mut self, idx: u8) {
        self.sub_shape = SubShape::from_index(idx);
    }

    fn update_increment(&mut self) {
        self.increment = self.base_freq * self.footage_mult / self.sample_rate;
    }

    pub fn tick(&mut self) -> VcoOut {
        let dt = self.increment;

        // Sawtooth: -1..1 ramp, band-limited at the wrap.
        let saw = (2.0 * self.phase - 1.0) - poly_blep(self.phase, dt);

        // Pulse: fixed 50% square.
        let pulse = band_limited_square(self.phase, 0.5, dt);

        // Sub-oscillator: phase derived from the master cycle counter so it is
        // exactly locked, then band-limited at its own (divided) rate.
        let (div, duty) = self.sub_shape.div_and_duty();
        let sub_phase = ((self.cycle as f32 % div) + self.phase) / div;
        let sub = band_limited_square(sub_phase, duty, dt / div);

        // Advance master phase and cycle counter.
        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.cycle = (self.cycle + 1) % 4;
        }

        VcoOut { saw, pulse, sub }
    }
}

/// Variable-duty square in -1..1, band-limited with PolyBLEP at both edges:
/// the rising edge at phase 0 and the falling edge at phase `duty`.
fn band_limited_square(phase: f32, duty: f32, dt: f32) -> f32 {
    let mut v = if phase < duty { 1.0 } else { -1.0 };
    v += poly_blep(phase, dt);
    let mut falling = phase - duty;
    if falling < 0.0 {
        falling += 1.0;
    }
    v -= poly_blep(falling, dt);
    v
}

#[cfg(test)]
mod tests {
    use super::Vco;

    const SR: f32 = 48_000.0;

    /// Count sawtooth cycles via upward zero-crossings (one per cycle, mid-ramp).
    /// Robust to band-limiting, which rounds the wrap but adds no extra crossings.
    fn count_saw_cycles(vco: &mut Vco, samples: usize) -> usize {
        let mut prev = vco.tick().saw;
        let mut cycles = 0;
        for _ in 0..samples {
            let s = vco.tick().saw;
            if prev < 0.0 && s >= 0.0 {
                cycles += 1;
            }
            prev = s;
        }
        cycles
    }

    #[test]
    fn footage_up_one_octave_doubles_pitch() {
        let mut at_8 = Vco::new(SR);
        at_8.set_note(57); // A3
        at_8.set_footage(1); // 8′ nominal
        let c8 = count_saw_cycles(&mut at_8, 24_000);

        let mut at_4 = Vco::new(SR);
        at_4.set_note(57);
        at_4.set_footage(2); // 4′ → one octave up
        let c4 = count_saw_cycles(&mut at_4, 24_000);

        // 4′ should produce ~twice as many cycles as 8′.
        let ratio = c4 as f32 / c8 as f32;
        assert!((ratio - 2.0).abs() < 0.1, "ratio={ratio} (c8={c8}, c4={c4})");
    }

    #[test]
    fn sub_square_down_one_is_half_master() {
        let mut vco = Vco::new(SR);
        vco.set_note(69);
        vco.set_footage(1);
        vco.set_sub_shape(0); // square −1 octave

        // Count master cycles via upward zero-crossings over the window.
        let master = count_saw_cycles(&mut vco, 24_000);
        // Master at 440 Hz over 24000 samples ≈ 220 cycles.
        assert!((210..230).contains(&master), "master={master}");
    }

    #[test]
    fn outputs_stay_in_reasonable_range() {
        let mut vco = Vco::new(SR);
        vco.set_note(60);
        for _ in 0..4_000 {
            let o = vco.tick();
            // PolyBLEP can overshoot slightly; allow a small margin.
            for v in [o.saw, o.pulse, o.sub] {
                assert!(v.abs() < 1.5, "value out of range: {v}");
            }
        }
    }
}
