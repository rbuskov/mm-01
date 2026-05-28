mod vca;
mod vco;

use vca::Vca;
use vco::Vco;

const STACK_CAP: usize = 16;

pub struct Voice {
    vco: Vco,
    vca: Vca,
    held: [u8; STACK_CAP],
    held_len: usize,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Voice {
        Voice {
            vco: Vco::new(sample_rate),
            vca: Vca::new(),
            held: [0; STACK_CAP],
            held_len: 0,
        }
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
        self.vca.process(osc)
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
