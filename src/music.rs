#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeSignature {
    pub upper: u8,
    pub lower: u8,
}

impl TimeSignature {
    pub fn new(upper: u8, lower: u8) -> Option<TimeSignature> {
        if lower == 0 || (lower & (lower - 1)) != 0 {
            // lower must be a power of 2
            return None
        }
        Some(TimeSignature { upper, lower })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tempo {
    pub mbpm: u64
}

impl Tempo {
    pub fn from_bpm(bpm: f32) -> Tempo {
        Tempo { mbpm: (bpm * 1000f32).round() as u64 }
    }

    pub fn bpm(&self) -> f32 {
        (self.mbpm as f32) / 1000f32
    }
}
