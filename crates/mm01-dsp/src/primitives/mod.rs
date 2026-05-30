//! Shared DSP primitives.

/// PolyBLEP (polynomial band-limited step) residual for anti-aliasing the hard
/// discontinuities of saw/pulse oscillators.
///
/// `t` is the oscillator phase in `[0, 1)`, `dt` the per-sample phase increment
/// (i.e. `freq / sample_rate`). The returned correction is added at rising
/// discontinuities and subtracted at falling ones to remove the step's
/// aliased harmonics. Outside the two-sample neighbourhood of a discontinuity
/// it returns 0, so it is cheap to apply unconditionally.
///
/// Reference: the same minBLEP-family band-limiting VCV Rack's `Fundamental`
/// VCO uses, in its lighter polynomial form (spec permits "PolyBLEP or
/// equivalent").
pub fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        // Just after the step: 2x - x^2 - 1
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        // Just before the step (wrapping): x^2 + 2x + 1
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::poly_blep;

    #[test]
    fn zero_away_from_discontinuity() {
        // Mid-phase, far from any step, the residual must be exactly zero.
        assert_eq!(poly_blep(0.5, 0.01), 0.0);
    }

    #[test]
    fn nonzero_near_discontinuity() {
        // Just after and just before a step it must contribute a correction.
        assert!(poly_blep(0.001, 0.01) != 0.0);
        assert!(poly_blep(0.999, 0.01) != 0.0);
    }

    #[test]
    fn zero_increment_is_safe() {
        assert_eq!(poly_blep(0.0, 0.0), 0.0);
    }
}
