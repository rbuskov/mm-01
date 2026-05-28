pub struct Vca {
    gate: bool,
}

impl Vca {
    pub fn new() -> Vca {
        Vca { gate: false }
    }

    pub fn set_gate(&mut self, on: bool) {
        self.gate = on;
    }

    pub fn process(&self, input: f32) -> f32 {
        if self.gate { input } else { 0.0 }
    }
}
