use super::noise::Noise;
use crate::primitives::poly_blep;

/// LFO waveform. Triangle and square are bipolar shapes; "random" is a
/// sample-and-hold step (one new value per cycle); "noise" is continuous white
/// noise. This iteration only the *rate* matters (it clocks the envelope's LFO
/// trigger mode) — the waveform is wired for later iterations.
#[derive(Clone, Copy)]
pub enum LfoWave {
    Triangle,
    Square,
    Random,
    Noise,
}

impl LfoWave {
    fn from_index(idx: u8) -> LfoWave {
        match idx {
            1 => LfoWave::Square,
            2 => LfoWave::Random,
            3 => LfoWave::Noise,
            _ => LfoWave::Triangle,
        }
    }
}

/// One LFO sample: its current `value` and whether the phase just wrapped
/// (one `wrapped` tick per cycle — the envelope's retrigger clock).
pub struct LfoOut {
    pub value: f32,
    pub wrapped: bool,
}

pub struct Lfo {
    sample_rate: f32,
    phase: f32,
    increment: f32,
    wave: LfoWave,
    sh: f32, // sample-and-hold value for the Random waveform
    noise: Noise,
}

impl Lfo {
    pub fn new(sample_rate: f32) -> Lfo {
        let mut noise = Noise::new();
        let sh = noise.tick();
        Lfo {
            sample_rate,
            phase: 0.0,
            increment: 3.0 / sample_rate, // default 3 Hz
            wave: LfoWave::Triangle,
            sh,
            noise,
        }
    }

    pub fn set_rate(&mut self, hz: f32) {
        self.increment = hz.max(0.0) / self.sample_rate;
    }

    pub fn set_wave(&mut self, idx: u8) {
        self.wave = LfoWave::from_index(idx);
    }

    pub fn tick(&mut self) -> LfoOut {
        let dt = self.increment;
        let value = match self.wave {
            // Triangle: -1 at phase 0, +1 at phase 0.5.
            LfoWave::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            LfoWave::Square => {
                let mut v = if self.phase < 0.5 { 1.0 } else { -1.0 };
                v += poly_blep(self.phase, dt);
                let mut falling = self.phase - 0.5;
                if falling < 0.0 {
                    falling += 1.0;
                }
                v -= poly_blep(falling, dt);
                v
            }
            LfoWave::Random => self.sh,
            LfoWave::Noise => self.noise.tick(),
        };

        self.phase += self.increment;
        let mut wrapped = false;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            wrapped = true;
            // New sample-and-hold value at each cycle boundary.
            self.sh = self.noise.tick();
        }

        LfoOut { value, wrapped }
    }
}

#[cfg(test)]
mod tests {
    use super::Lfo;

    #[test]
    fn wraps_at_the_set_rate() {
        let sr = 48_000.0;
        let mut lfo = Lfo::new(sr);
        lfo.set_rate(10.0); // 10 Hz → 10 wraps per second
        let mut wraps = 0;
        for _ in 0..(sr as usize) {
            if lfo.tick().wrapped {
                wraps += 1;
            }
        }
        assert!((9..=11).contains(&wraps), "wraps={wraps}");
    }

    #[test]
    fn triangle_stays_bipolar_in_range() {
        let mut lfo = Lfo::new(48_000.0);
        lfo.set_rate(5.0);
        lfo.set_wave(0); // triangle
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..48_000 {
            let v = lfo.tick().value;
            min = min.min(v);
            max = max.max(v);
        }
        assert!(min < -0.9 && max > 0.9, "min={min} max={max}");
        assert!(min >= -1.001 && max <= 1.001, "out of range min={min} max={max}");
    }
}
