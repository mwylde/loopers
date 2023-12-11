use crate::sample::PlayOutput::Done;
use crate::sample::{Sample, SamplePlayer};
use crate::MetricStructure;

use loopers_common::api::FrameTime;
use std::sync::Arc;

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use loopers_common::music::Tempo;
    use super::*;

    fn sample(v: f32, n: usize) -> Sample {
        Sample {
            buffer: [vec![v; n], vec![-v; n]],
        }
    }

    #[test]
    fn test_cycle() {
        let normal = sample(2.0, 4);
        let emphasis = sample(4.0, 4);

        let bpm = 60_000f32 / FrameTime(8).to_ms() as f32;

        let mut met = Metronome::new(
            MetricStructure::new(3, 4,
                                 Tempo::from_bpm(bpm)).unwrap(), normal, emphasis);

        assert_eq!(0, met.time.0);

        let mut l = vec![1f32; 2];
        let mut r = vec![-1f32; 2];
        // play first half of emphasis beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![3f32, 3.0], l);
        assert_eq!(vec![-3f32, -3.0], r);
        assert_eq!(2, met.time.0);

        // play second half of emphasis beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![5f32, 5.0], l);
        assert_eq!(vec![-5f32, -5.0], r);
        assert_eq!(4, met.time.0);

        // play nothing
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![5f32, 5.0], l);
        assert_eq!(vec![-5f32, -5.0], r);
        assert_eq!(6, met.time.0);

        // play nothing
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![5f32, 5.0], l);
        assert_eq!(vec![-5f32, -5.0], r);
        assert_eq!(8, met.time.0);

        // play first half of normal beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![6f32, 6.0], l);
        assert_eq!(vec![-6f32, -6.0], r);
        assert_eq!(10, met.time.0);

        // play second half of normal beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![7f32, 7.0], l);
        assert_eq!(vec![-7f32, -7.0], r);
        assert_eq!(12, met.time.0);

        // play nothing twice
        for _ in 0..2 {
            met.advance(&mut [&mut l, &mut r]);
            assert_eq!(vec![7f32, 7.0], l);
            assert_eq!(vec![-7f32, -7.0], r);
        }
        assert_eq!(16, met.time.0);

        // play first half of normal beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![8f32, 8.0], l);
        assert_eq!(vec![-8f32, -8.0], r);
        assert_eq!(18, met.time.0);

        // play second half of normal beat
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![9f32, 9.0], l);
        assert_eq!(vec![-9f32, -9.0], r);
        assert_eq!(20, met.time.0);

        // play nothing twice
        for _ in 0..2 {
            met.advance(&mut [&mut l, &mut r]);
            assert_eq!(vec![9f32, 9.0], l);
            assert_eq!(vec![-9f32, -9.0], r);
        }
        assert_eq!(24, met.time.0);

        // and now we should be back to emphasis
        met.advance(&mut [&mut l, &mut r]);
        assert_eq!(vec![11f32, 11.0], l);
        assert_eq!(vec![-11f32, -11.0], r);
        assert_eq!(26, met.time.0);
    }
}

pub struct Metronome {
    metric_structure: MetricStructure,
    beat_normal: Arc<Sample>,
    beat_emphasis: Arc<Sample>,
    time: FrameTime,
    player: Option<SamplePlayer>,
    volume: f32,
}

impl Metronome {
    pub fn new(
        metric_structure: MetricStructure,
        beat_normal: Sample,
        beat_emphasis: Sample,
    ) -> Metronome {
        let beat_emphasis = Arc::new(beat_emphasis);
        let player = SamplePlayer::new(beat_emphasis.clone());

        Metronome {
            metric_structure,
            beat_normal: Arc::new(beat_normal),
            beat_emphasis,
            time: FrameTime(0),
            player: Some(player),
            volume: 1.0,
        }
    }

    pub fn set_metric_structure(&mut self, ms: MetricStructure) {
        self.metric_structure = ms;
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn beat(&self) -> i64 {
        self.metric_structure.tempo.beat(self.time)
    }

    pub fn reset(&mut self) {
        self.time = FrameTime(0);
        self.player = Some(SamplePlayer::new(self.beat_emphasis.clone()))
    }

    pub fn advance(&mut self, out: &mut [&mut [f32]; 2]) {
        assert_eq!(out[0].len(), out[1].len());

        // TODO: it would be more accurate to do this analytically, i.e., use the current
        //   time without relying on the exact timing of the calls
        if let Some(player) = &mut self.player {
            if player.play(out, self.volume / 2.0) == Done {
                self.player = None;
            }
        }

        let len = out[0].len();

        let cur_beat = self.beat();
        self.time.0 += len as i64;
        let next_beat = self.beat();

        // println!("{} -> {} / {} -> {}", cur_beat, next_beat, self.time.0 - len as i64, self.time.0);

        if next_beat != cur_beat {
            let sample = if self
                .metric_structure
                .time_signature
                .beat_of_measure(next_beat)
                == 0
            {
                self.beat_emphasis.clone()
            } else {
                self.beat_normal.clone()
            };

            self.player = Some(SamplePlayer::new(sample));
        }
    }
}
