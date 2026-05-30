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
pub const PARAM_AMP_SOURCE: u8 = 7; // 0→ENV, 1→GATE
pub const PARAM_VOLUME: u8 = 8;
pub const PARAM_ENV_ATTACK: u8 = 9; // normalised 0..1
pub const PARAM_ENV_DECAY: u8 = 10;
pub const PARAM_ENV_SUSTAIN: u8 = 11;
pub const PARAM_ENV_RELEASE: u8 = 12;
pub const PARAM_ENV_TRIGGER_MODE: u8 = 13; // 0→GATE+TRIG, 1→GATE, 2→LFO
pub const PARAM_LFO_RATE: u8 = 14; // normalised 0..1
pub const PARAM_LFO_WAVE: u8 = 15; // 0→tri, 1→square, 2→random, 3→noise

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
