use crate::api::{FrameTime, SAMPLE_RATE};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::FrameTime;

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
        assert_eq!(0, tempo.beat(FrameTime(22049)));

        assert_eq!(1, tempo.beat(FrameTime(22050)));
        assert_eq!(1, tempo.beat(FrameTime(22500)));
        assert_eq!(1, tempo.beat(FrameTime(44099)));

        assert_eq!(2, tempo.beat(FrameTime(44100)));
        assert_eq!(2, tempo.beat(FrameTime(44101)));

        assert_eq!(-1, tempo.beat(FrameTime(-22050)));
        assert_eq!(-2, tempo.beat(FrameTime(-44100)));

        assert_eq!(15, tempo.beat(FrameTime(352768)));
        assert_eq!(16, tempo.beat(FrameTime(352800)));
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

    #[test]
    fn test_next_beat() {
        let ts = Tempo::from_bpm(120.0);
        assert_eq!(FrameTime(352800), ts.next_full_beat(FrameTime(352768)));

        assert_eq!(FrameTime(352800), ts.next_full_beat(FrameTime(352800)));

        assert_eq!(FrameTime(0), ts.next_full_beat(FrameTime(0)));
    }

    #[test]
    fn test_next_beat_consistency() {
        let ts = TimeSignature::new(4, 4).unwrap();
        let tempo = Tempo::from_bpm(120.0);

        // test that beat_of_measure is working the same as next_full_beat
        let time = 5644544;
        let frames = 256;

        let prev_beat_of_measure = ts
            .beat_of_measure(tempo.beat(FrameTime(time)));

        let beat_of_measure = ts
            .beat_of_measure(tempo.beat(FrameTime(time + frames)));

        assert_ne!(prev_beat_of_measure, beat_of_measure);

        let next_beat_time = tempo
            .next_full_beat(FrameTime(time));

        assert!(
            next_beat_time.0 <= time + frames as i64,
            format!(
                "{} > {} (time = {})",
                next_beat_time.0,
                time + frames as i64,
                time
            )
        );

        // test that we never suggest beats in the past
        let time = 176384;
        assert!(tempo.next_full_beat(FrameTime(time)).0 >= time);
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeSignature {
    pub upper: u8,
    pub lower: u8,
}

impl TimeSignature {
    pub fn new(upper: u8, lower: u8) -> Option<TimeSignature> {
        if lower == 0 || (lower & (lower - 1)) != 0 {
            // lower must be a power of 2
            return None;
        }
        Some(TimeSignature { upper, lower })
    }

    // converts from (possibly negative) beat-of-song to always positive beat-of-measure
    pub fn beat_of_measure(&self, beat: i64) -> u8 {
        beat.rem_euclid(self.upper as i64) as u8
    }

    pub fn measure(&self, beat: i64) -> i64 {
        beat / self.lower as i64
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tempo {
    samples_per_beat: u64,
}

impl Tempo {
    pub fn from_bpm(bpm: f32) -> Tempo {
        assert!(bpm > 0.0, "bpm must be positve");
        Tempo {
            samples_per_beat: ((SAMPLE_RATE * 1000.0) / (bpm as f64 / 60.0)) as u64,
        }
    }

    pub fn bpm(&self) -> f32 {
        ((SAMPLE_RATE * 1000.0) / self.samples_per_beat as f64 * 60.0) as f32
    }

    pub fn samples_per_beat(&self) -> u64 {
        self.samples_per_beat
    }

    pub fn beat(&self, time: FrameTime) -> i64 {
        time.0 / self.samples_per_beat as i64
    }

    /// Returns the exact time of the next full beat from the given `time` (e.g., the 0 time of
    /// the beat). If `time` already points to the 0 of a beat, will return `time`.
    pub fn next_full_beat(&self, time: FrameTime) -> FrameTime {
        let cur = self.beat(time);
        let rem = time.0.rem_euclid(self.samples_per_beat as i64);

        if rem == 0 {
            FrameTime(cur * self.samples_per_beat as i64)
        } else {
            FrameTime((cur + 1) * self.samples_per_beat as i64)
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub struct MetricStructure {
    pub time_signature: TimeSignature,
    pub tempo: Tempo,
}

impl MetricStructure {
    pub fn new(upper: u8, lower: u8, bpm: f32) -> Option<MetricStructure> {
        let time_signature = TimeSignature::new(upper, lower)?;
        Some(MetricStructure {
            time_signature,
            tempo: Tempo::from_bpm(bpm),
        })
    }
}
