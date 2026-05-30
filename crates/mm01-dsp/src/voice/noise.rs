/// White noise generator, independent of pitch. Uses an explicitly seeded
/// xorshift32 PRNG so output is deterministic (per the DSP-layer determinism
/// rule) and allocation/branch-free in the audio callback.
pub struct Noise {
    state: u32,
}

impl Noise {
    pub fn new() -> Noise {
        // Fixed non-zero seed: xorshift must never start at 0.
        Noise { state: 0x2545_f491 }
    }

    pub fn tick(&mut self) -> f32 {
        // xorshift32 (Marsaglia).
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        // Map u32 → [-1, 1).
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::Noise;

    #[test]
    fn in_range_and_varies() {
        let mut n = Noise::new();
        let mut prev = n.tick();
        let mut changed = false;
        for _ in 0..1000 {
            let s = n.tick();
            assert!((-1.0..=1.0).contains(&s));
            if s != prev {
                changed = true;
            }
            prev = s;
        }
        assert!(changed, "noise must not be constant");
    }

    #[test]
    fn deterministic_for_seed() {
        let mut a = Noise::new();
        let mut b = Noise::new();
        for _ in 0..100 {
            assert_eq!(a.tick(), b.tick());
        }
    }
}
