use crate::midi::MidiEvent::{ControllerChange};

#[derive(Debug)]
pub enum MidiEvent {
    ControllerChange {
        pub channel: u8,
        pub controller: u8,
        pub value: u8,
    },
}

impl MidiEvent {
    pub fn from_bytes(bs: &[u8]) -> Option<Self> {
        if bs.len() == 3 && bs[0] >> 4 == 0xb {
            Some(ControllerChange {
                channel: bs[0] & 0b1111,
                controller: bs[1],
                value: bs[2],
            })
        } else {
            None
        }
    }
}
