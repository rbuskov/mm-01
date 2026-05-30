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

/// dB range of the level fader taper: position 1.0 → 0 dB (unity), and the
/// taper is linear-in-dB down to roughly −FADER_RANGE_DB before blending to
/// silence at the bottom. Larger = wider/steeper.
const FADER_RANGE_DB: f32 = 40.0;

/// Convert a 0..1 fader *position* into a linear gain using an audio (dB) taper.
///
/// A linear-amplitude fader feels dead across its top half (position 0.5 is only
/// −6 dB; 0.9 is −0.9 dB), bunching all the audible change into the bottom. This
/// maps position linearly in dB so loudness changes evenly across the travel —
/// what real mixer faders do — and blends to true zero at the bottom so the
/// channel fully closes. Position 1.0 → 1.0, position 0.0 → 0.0.
pub fn level_taper(position: f32) -> f32 {
    let p = position.clamp(0.0, 1.0);
    if p <= 0.0 {
        return 0.0;
    }
    let g = 10f32.powf((p - 1.0) * FADER_RANGE_DB / 20.0);
    let floor = 10f32.powf(-FADER_RANGE_DB / 20.0);
    ((g - floor) / (1.0 - floor)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::{level_taper, Mixer};

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
    fn taper_endpoints_and_clamping() {
        assert_eq!(level_taper(0.0), 0.0);
        assert_eq!(level_taper(-0.5), 0.0);
        assert!((level_taper(1.0) - 1.0).abs() < 1e-6);
        assert!((level_taper(2.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn taper_is_monotonic_and_below_linear() {
        // Audio taper sits below the linear line in the middle (e.g. 0.5 is well
        // under −6 dB) and rises monotonically.
        let mid = level_taper(0.5);
        assert!(mid < 0.5, "0.5 position should be quieter than linear: {mid}");
        let mut prev = 0.0;
        for i in 0..=20 {
            let g = level_taper(i as f32 / 20.0);
            assert!(g >= prev, "must be monotonic");
            prev = g;
        }
    }
}
