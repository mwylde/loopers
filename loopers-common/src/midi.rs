#[derive(Debug)]
pub enum MidiEvent {
    ControllerChange {
        channel: u8,
        controller: u8,
        data: u8,
    },
}

impl MidiEvent {
    pub fn from_bytes(bs: &[u8]) -> Option<Self> {
        if bs.len() == 3 && bs[0] >> 4 == 0xb {
            Some(MidiEvent::ControllerChange {
                channel: bs[0] & 0b1111,
                controller: bs[1],
                data: bs[2],
            })
        } else {
            None
        }
    }
}
