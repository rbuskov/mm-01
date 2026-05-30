pub const TAG_NOTE_ON: u8 = 0;
pub const TAG_NOTE_OFF: u8 = 1;
pub const TAG_PARAM_SET: u8 = 2;

pub const PARAM_MASTER_GAIN: u8 = 0;
pub const PARAM_FOOTAGE: u8 = 1; // 0→16′, 1→8′, 2→4′, 3→2′
pub const PARAM_SUB_SHAPE: u8 = 2; // 0→sq −1, 1→sq −2, 2→pulse −2
pub const PARAM_MIX_SAW: u8 = 3;
pub const PARAM_MIX_PULSE: u8 = 4;
pub const PARAM_MIX_SUB: u8 = 5;
pub const PARAM_MIX_NOISE: u8 = 6;

pub enum Msg {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    ParamSet { id: u8, value: f32 },
}

impl Msg {
    pub fn decode(bytes: &[u8]) -> Option<Msg> {
        let tag = *bytes.first()?;
        match tag {
            TAG_NOTE_ON => {
                let note = *bytes.get(1)?;
                let velocity = *bytes.get(2)?;
                Some(Msg::NoteOn { note, velocity })
            }
            TAG_NOTE_OFF => {
                let note = *bytes.get(1)?;
                Some(Msg::NoteOff { note })
            }
            TAG_PARAM_SET => {
                let id = *bytes.get(1)?;
                let v = bytes.get(2..6)?;
                let value = f32::from_le_bytes([v[0], v[1], v[2], v[3]]);
                Some(Msg::ParamSet { id, value })
            }
            _ => None,
        }
    }
}
