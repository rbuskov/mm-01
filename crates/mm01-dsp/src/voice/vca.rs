/// Amp control source, picked by a 2-position switch.
#[derive(Clone, Copy)]
pub enum AmpSource {
    /// Gain follows the ADSR envelope contour.
    Env,
    /// Gain is full while a key is held, zero otherwise (organ-style on/off).
    Gate,
}

impl AmpSource {
    fn from_index(idx: u8) -> AmpSource {
        match idx {
            1 => AmpSource::Gate,
            _ => AmpSource::Env,
        }
    }
}

/// Continuous-gain voltage-controlled amplifier. The control signal is selected
/// between the envelope and the raw gate; the master volume is applied
/// downstream at the voice output, not here.
pub struct Vca {
    source: AmpSource,
}

impl Vca {
    pub fn new() -> Vca {
        Vca {
            source: AmpSource::Env,
        }
    }

    pub fn set_source(&mut self, idx: u8) {
        self.source = AmpSource::from_index(idx);
    }

    pub fn process(&self, input: f32, env: f32, gate: bool) -> f32 {
        let gain = match self.source {
            AmpSource::Env => env,
            AmpSource::Gate => {
                if gate {
                    1.0
                } else {
                    0.0
                }
            }
        };
        input * gain
    }
}
