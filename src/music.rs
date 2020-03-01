const SAMPLE_RATE: f64 = 44.100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frametime() {
        let zero = FrameTime(0);
        assert_eq!(0.0, zero.to_ms());
        assert_eq!(zero, FrameTime::from_ms(0f64));

        assert_eq!(1000.0, FrameTime(44100).to_ms());
        assert_eq!(44100, FrameTime::from_ms(1000.0).0);

        let ms = 23538.5f64;
        assert_eq!(ms.floor(), FrameTime::from_ms(ms).to_ms().floor())
    }

    #[test]
    fn test_get_beat() {
        let tempo = Tempo::from_bpm(120f32);
        assert_eq!(0, tempo.beat(FrameTime(0)));
        assert_eq!(1, tempo.beat(FrameTime(22500)));
        assert_eq!(-1, tempo.beat(FrameTime(-22500)));
    }

    #[test]
    fn test_beat_normalization() {
        let ts = TimeSignature::new(3, 4).unwrap();
        assert_eq!(0, ts.beat_of_measure(0));
        assert_eq!(1, ts.beat_of_measure(1));
        assert_eq!(0, ts.beat_of_measure(3));
        assert_eq!(0, ts.beat_of_measure(-3));
        assert_eq!(1, ts.beat_of_measure(-2));
    }
}


#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub fn from_ms(ms: f64) -> FrameTime {
        FrameTime((ms * SAMPLE_RATE) as i64)
    }

    pub fn to_ms(&self) -> f64 {
        ((self.0 as f64) / SAMPLE_RATE)
    }
}

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

    // converts from (possibly negative) beat-of-song to always positive beat-of-measure
    pub fn beat_of_measure(&self, beat: i64) -> u8 {
        beat.rem_euclid(self.upper as i64) as u8
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

    pub fn beat(&self, time: FrameTime) -> i64 {
        let bps = self.bpm() / 60.0;
        let mspb = 1000.0 / bps;
        (time.to_ms() as f32 / mspb) as i64
    }
}

