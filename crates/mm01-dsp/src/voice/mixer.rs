/// Four-input source mixer. Each input has an independent level in `0..=1`.
///
/// The mixer deliberately does NOT normalise: several sources at full level sum
/// past unity, and that overdrive is part of the SH-101's character (it drives
/// the filter on the real unit). No clipping happens here — it just sums.
pub struct Mixer {
    pub saw: f32,
    pub pulse: f32,
    pub sub: f32,
    pub noise: f32,
}

impl Mixer {
    pub fn new() -> Mixer {
        // Default to saw-only at full level, matching the iteration-1 timbre.
        Mixer {
            saw: 1.0,
            pulse: 0.0,
            sub: 0.0,
            noise: 0.0,
        }
    }

    pub fn mix(&self, saw: f32, pulse: f32, sub: f32, noise: f32) -> f32 {
        saw * self.saw + pulse * self.pulse + sub * self.sub + noise * self.noise
    }
}

/// Clamp a mixer level to the valid `0..=1` range.
pub fn clamp_level(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{clamp_level, Mixer};

    #[test]
    fn default_is_saw_only() {
        let m = Mixer::new();
        assert_eq!(m.mix(0.5, 0.9, 0.9, 0.9), 0.5);
    }

    #[test]
    fn sums_past_unity_without_normalising() {
        let m = Mixer {
            saw: 1.0,
            pulse: 1.0,
            sub: 1.0,
            noise: 1.0,
        };
        // Four full sources deliberately overdrive past unity.
        assert_eq!(m.mix(1.0, 1.0, 1.0, 1.0), 4.0);
    }

    #[test]
    fn levels_clamp_to_unit_range() {
        assert_eq!(clamp_level(-0.5), 0.0);
        assert_eq!(clamp_level(2.0), 1.0);
        assert_eq!(clamp_level(0.3), 0.3);
    }
}
