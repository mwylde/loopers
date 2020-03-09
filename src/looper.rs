use crate::sample::Sample;
use crate::protos::{LooperMode, SavedLooper};
use crate::music::FrameTime;
use std::path::Path;
use crate::error::SaveLoadError;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::f32::consts::PI;

    #[test]
    fn test_new() {
        let looper = Looper::new(1);
        assert_eq!(LooperMode::None, looper.mode);
        assert_eq!(1, looper.id);
        assert_eq!(0, looper.length_in_samples());
    }

    #[test]
    fn test_transitions() {
        let mut looper = Looper::new(1);

        assert_eq!(LooperMode::None, looper.mode);

        looper.transition_to(LooperMode::Record);
        assert_eq!(LooperMode::Record, looper.mode);
        assert_eq!(1, looper.samples.len());

        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        looper.process_input(0, &[&data[0], &data[1]]);

        looper.transition_to(LooperMode::Overdub);
        assert_eq!(2, looper.samples.len());
        for s in &looper.samples {
            assert_eq!(2, s.length());
        }

        looper.transition_to(LooperMode::Playing);
        assert_eq!(LooperMode::Playing, looper.mode);

        looper.transition_to(LooperMode::Record);
        assert_eq!(1, looper.samples.len());
        assert_eq!(0, looper.length_in_samples());
    }

    #[test]
    fn test_io() {
        let mut l = Looper::new(1);
        l.transition_to(LooperMode::Record);
        let input_left = vec![1f32, 2.0, 3.0, 4.0];
        let input_right = vec![-1f32, -2.0, -3.0, -4.0];
        l.process_input(0, &[&input_left, &input_right]);

        let output_left = vec![1f64; 2];
        let output_right = vec![-1f64; 2];

        l.transition_to(LooperMode::Playing);

        let mut o = [output_left, output_right];

        l.process_output(FrameTime(0), &mut o);
        assert_eq!(vec![2.0f64, 3.0], o[0]);
        assert_eq!(vec![-2.0f64, -3.0], o[1]);

        l.process_output(FrameTime(2), &mut o);
        assert_eq!(vec![5f64, 7.0], o[0]);
        assert_eq!(vec![-5f64, -7.0], o[1]);

        l.process_output(FrameTime(4), &mut o);
        assert_eq!(vec![6f64, 9.0], o[0]);
        assert_eq!(vec![-6f64, -9.0], o[1]);
    }

    #[test]
    fn test_serialization() {
        let dir = tempdir().unwrap();
        let mut input_left = vec![];
        let mut input_right = vec![];

        let mut input_left2 = vec![];
        let mut input_right2 = vec![];

        for t in (0 .. 44100).map(|x| x as f32 / 44100.0) {
            let sample = (t * 440.0 * 2.0 * PI).sin();
            input_left.push(sample / 2.0);
            input_right.push(sample / 2.0);

            let sample = (t * 540.0 * 2.0 * PI).sin();
            input_left2.push(sample / 2.0);
            input_right2.push(sample / 2.0);
        }

        let mut looper = Looper::new(5);

        looper.transition_to(LooperMode::Record);
        looper.process_input(0, &[&input_left, &input_right]);

        looper.transition_to(LooperMode::Overdub);
        looper.process_input(0, &[&input_left2, &input_right2]);


        let state = looper.serialize(dir.path()).unwrap();

        let deserialized = Looper::from_serialized(&state,dir.path()).unwrap();

        assert_eq!(looper.id, deserialized.id);
        assert_eq!(2, deserialized.samples.len());

        assert_eq!(input_left, deserialized.samples[0].buffer[0]);
        assert_eq!(input_right, deserialized.samples[0].buffer[1]);

        assert_eq!(input_left2, deserialized.samples[1].buffer[0]);
        assert_eq!(input_right2, deserialized.samples[1].buffer[1]);

    }
}

const CROSS_FADE_SAMPLES: usize = 1024;

// The Looper struct encapsulates behavior similar to a single hardware looper. Internally, it is
// driven by a state machine, which controls how it responds to input buffers (e.g., by recording
// or overdubbing to its internal buffers) and output buffers (e.g., by playing).
pub struct Looper {
    pub id: u32,
    pub samples: Vec<Sample>,
    pub mode: LooperMode,
    pub deleted: bool,
    xfade_samples_left: usize,
}

impl Looper {
    pub fn new(id: u32) -> Looper {
        let looper = Looper {
            id,
            samples: vec![],
            mode: LooperMode::None,
            deleted: false,
            xfade_samples_left: 0,
        };

        looper
    }
}

impl Looper {
    pub fn transition_to(&mut self, mode: LooperMode) {
        println!("Transition {:?} to {:?}", self.mode, mode);
        match (self.mode, mode) {
            (x, y) if x == y => {},
            (LooperMode::Record, LooperMode::None) |
            (LooperMode::Record, LooperMode::Playing) |
            (LooperMode::Overdub, LooperMode::None) |
            // (LooperMode::Overdub, LooperMode::Playing) => {
            //     self.xfade_samples_left = CROSS_FADE_SAMPLES;
            // },
            (_, LooperMode::None) => {},
            (_, LooperMode::Ready) => {},
            (_, LooperMode::Record) => {
                self.samples.clear();
                self.samples.push(Sample::new());
            },
            (_, LooperMode::Overdub) => {
                let len = self.length_in_samples();
                if len == 0 {
                    println!( "trying to move to overdub with 0-length looper");
                    self.mode = LooperMode::Record;
                    return;
                }
                self.samples.push(Sample::with_size(len as usize));
            },
            (_, LooperMode::Playing) => {},
        }

        self.mode = mode;
    }

    // In process_output, we modify the specified output buffers according to our internal state. In
    // Playing or Overdub mode, we will add our buffer to the output. Otherwise, we do nothing.
    pub fn process_output(&self, time: FrameTime, outputs: &mut [Vec<f64>; 2]) {
        if time.0 < 0 {
            return;
        }

        let output_len = outputs[0].len();
        let sample_len = self.length_in_samples() as usize;

        if self.mode == LooperMode::Playing || self.mode == LooperMode::Overdub {
            if !self.samples.is_empty() {
                for sample in &self.samples {
                    let b = &sample.buffer;
                    if b[0].is_empty() {
                        continue;
                    }

                    for i in 0usize..2 {
                        for t in 0..output_len {
                            outputs[i][t] += b[i][(time.0 as usize + t) % sample_len] as f64;
                        }
                    }
                }
            }
        }
    }

    // In process_input, we modify our internal buffers based on the input. In Record mode, we
    // append the data in the input buffers to our current sample. In Overdub mode, we sum the data
    // with whatever is currently in our buffer at the point of time_in_samples.
    pub fn process_input(&mut self, time_in_samples: u64, inputs: &[&[f32]]) {
        let len = self.length_in_samples();
        if self.mode == LooperMode::Overdub {
            // in overdub mode, we add the new samples to our existing buffer
            let s = self.samples.last_mut().expect("No samples for looper in overdub mode");
            s.overdub(time_in_samples % len, inputs);
        } else if self.mode == LooperMode::Record {
            // in record mode, we extend the current buffer with the new samples
            let s = self.samples.last_mut().expect("No samples for looper in record mode");
            s.record(inputs);
        } else if self.xfade_samples_left > 0 {
            if let Some(s) = self.samples.last_mut() {
                // linear
                s.xfade(CROSS_FADE_SAMPLES, time_in_samples, inputs);
                self.xfade_samples_left = (self.xfade_samples_left - inputs[0].len()).max(0);
            } else {
                println!("tried to cross fade but no samples... something is likely wrong");
            }
        }
    }

    pub fn length_in_samples(&self) -> u64 {
        self.samples.get(0).map(|s| s.length()).unwrap_or(0)
    }

    pub fn serialize(&self, path: &Path) -> Result<SavedLooper, SaveLoadError> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut saved = SavedLooper {
            id: self.id,
            samples: Vec::with_capacity(self.samples.len()),
        };


        for (i, s) in self.samples.iter().enumerate() {
            let p = path.join(format!("loop_{}_{}.wav", self.id, i));
            let mut writer = hound::WavWriter::create(&p,
                                                      spec.clone())?;

            for j in 0..s.length() as usize {
                writer.write_sample(s.buffer[0][j])?;
                writer.write_sample(s.buffer[1][j])?;
            }
            writer.finalize()?;
            saved.samples.push(p.to_str().unwrap().to_string());
        }

        Ok(saved)
    }

    pub fn from_serialized(state: &SavedLooper, path: &Path) -> Result<Looper, SaveLoadError> {
        let mut looper = Looper {
            id: state.id,
            samples: Vec::with_capacity(state.samples.len()),
            mode: LooperMode::None,
            deleted: false,
            xfade_samples_left: 0,
        };

        for sample_path in &state.samples {
            let mut reader = hound::WavReader::open(
                &path.join(sample_path))?;

            let mut sample = Sample::new();
            let mut left = Vec::with_capacity(reader.len() as usize / 2);
            let mut right = Vec::with_capacity(reader.len() as usize / 2);

            for (i, s) in reader.samples().enumerate() {
                if i % 2 == 0 {
                    left.push(s?);
                } else {
                    right.push(s?);
                }
            }

            sample.record(&[&left, &right]);
            looper.samples.push(sample);
        }

        Ok(looper)
    }
}
