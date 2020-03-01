use crate::sample::{Sample, SamplePlayer};
use crate::sample::PlayOutput::Done;
use crate::music::{Tempo, TimeSignature, FrameTime};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
    }
}

pub struct Metronome<'a> {
    tempo: Tempo,
    time_signature: TimeSignature,
    beat_normal: Sample,
    beat_emphasis: Sample,
    time: FrameTime,
    player: Option<SamplePlayer<'a>>,
}

impl <'a> Metronome<'a> {
    pub fn new(tempo: Tempo, time_signature: TimeSignature,
               beat_normal: Sample, beat_emphasis: Sample) -> Metronome {
        Metronome {
            tempo, time_signature, beat_normal, beat_emphasis,
            time: FrameTime(0), player: Some(SamplePlayer::new(&beat_emphasis))
        }
    }

    pub fn reset(&mut self) {
        self.time = FrameTime(0);
    }

    pub fn advance(&mut self, out: &mut[&mut[f32]; 2]) {
        assert_eq!(out[0].len(), out[1].len());

        let len = out[0].len();
        if let Some(player) = &mut self.player {
            // we're already in process of playing a sample
            if player.play(out) == Done {
                self.player = None;
            }
            self.time.0 += len as i64;
        } else {
        }
    }
}
