use wasm_bindgen::prelude::*;

mod msg;
mod voice;

use voice::Voice;

#[wasm_bindgen]
pub struct Engine {
    sample_rate: f32,
    voice: Voice,
    master_gain: f32,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Engine {
        Engine {
            sample_rate,
            voice: Voice::new(sample_rate),
            master_gain: 1.0,
        }
    }

    pub fn process(&mut self, out_left: &mut [f32], out_right: &mut [f32]) {
        let n = out_left.len().min(out_right.len());
        for i in 0..n {
            let s = self.voice.tick() * self.master_gain;
            out_left[i] = s;
            out_right[i] = s;
        }
    }

    pub fn handle_message(&mut self, bytes: &[u8]) {
        if let Some(msg) = msg::Msg::decode(bytes) {
            match msg {
                msg::Msg::NoteOn { note, velocity: _ } => self.voice.note_on(note),
                msg::Msg::NoteOff { note } => self.voice.note_off(note),
                msg::Msg::ParamSet { id, value } => self.set_param(id, value),
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<u8> {
        Vec::new()
    }
}

impl Engine {
    fn set_param(&mut self, id: u8, value: f32) {
        if id == msg::PARAM_MASTER_GAIN {
            self.master_gain = value.clamp(0.0, 1.0);
        }
    }
}
