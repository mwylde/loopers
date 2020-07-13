use crate::error::SaveLoadError;
use crate::music::FrameTime;
use crate::protos::{LooperMode, SavedLooper};
use crate::sample;
use crate::sample::{Sample, XfadeDirection};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    use tempfile::tempdir;

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
    fn test_post_xfade() {
        let mut l = Looper::new(1);
        l.transition_to(LooperMode::Record);
        let mut input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        let mut input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];

        l.process_input(0, &[&input_left, &input_right]);

        for i in 0..CROSS_FADE_SAMPLES {
            let q = i as f32 / CROSS_FADE_SAMPLES as f32;
            input_left[i] = -q / (1f32 - q);
            input_right[i] = q / (1f32 - q);
        }

        l.transition_to(LooperMode::Playing);

        for i in (0..CROSS_FADE_SAMPLES).step_by(16) {
            l.process_input(
                l.length_in_samples() + i as u64,
                &[&input_left[i..i + 16], &input_right[i..i + 16]],
            );
        }

        let output_left = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let output_right = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o = [output_left, output_right];
        l.process_output(FrameTime(0), &mut o);

        for i in 0..o[0].len() {
            if i < CROSS_FADE_SAMPLES {
                assert!(
                    (0f64 - o[0][i]).abs() < 0.000001,
                    "left is {} at idx {}, expected 0",
                    o[0][i],
                    i
                );
                assert!(
                    (0f64 - o[1][i]).abs() < 0.000001,
                    "right is {} at idx {}, expected 0",
                    o[1][i],
                    i
                );
            } else {
                assert_eq!(1f64, o[0][i]);
                assert_eq!(-1f64, o[1][i]);
            }
        }
    }

    #[test]
    fn test_pre_xfade() {
        let mut l = Looper::new(1);

        let mut input_left = vec![17f32; CROSS_FADE_SAMPLES];
        let mut input_right = vec![-17f32; CROSS_FADE_SAMPLES];

        // process some random input
        for i in (0..CROSS_FADE_SAMPLES).step_by(16) {
            l.process_input(
                l.length_in_samples() + i as u64,
                &[&input_left[i..i + 16], &input_right[i..i + 16]],
            );
        }

        // construct our real input
        for i in 0..CROSS_FADE_SAMPLES {
            // q = i / CROSS_FADE_SAMPLES
            // 0 = d[i] * (1 - q) + x * q
            // -d[i] * (1-q) = x*q
            // (-i * (1-q)) / q

            let q = 1.0 - i as f32 / CROSS_FADE_SAMPLES as f32;

            if i != 0 {
                input_left[i] = -q / (1f32 - q);
                input_right[i] = q / (1f32 - q);
            }
        }

        // process that
        for i in (0..CROSS_FADE_SAMPLES).step_by(16) {
            l.process_input(
                l.length_in_samples() + i as u64,
                &[&input_left[i..i + 16], &input_right[i..i + 16]],
            );
        }

        l.transition_to(LooperMode::Record);

        input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];

        l.process_input(0, &[&input_left, &input_right]);

        l.transition_to(LooperMode::Overdub);

        let output_left = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let output_right = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o = [output_left, output_right];
        l.process_output(FrameTime(0), &mut o);

        for i in 0..o[0].len() {
            if i <= CROSS_FADE_SAMPLES {
                assert_eq!(1f64, o[0][i]);
                assert_eq!(-1f64, o[1][i]);
            } else {
                assert!(
                    (0f64 - o[0][i]).abs() < 0.000001,
                    "left is {} at idx {}, expected 0",
                    o[0][i],
                    i
                );
                assert!(
                    (0f64 - o[1][i]).abs() < 0.000001,
                    "right is {} at idx {}, expected 0",
                    o[1][i],
                    i
                );
            }
        }
    }

    #[test]
    fn test_serialization() {
        let dir = tempdir().unwrap();
        let mut input_left = vec![];
        let mut input_right = vec![];

        let mut input_left2 = vec![];
        let mut input_right2 = vec![];

        for t in (0..16).map(|x| x as f32 / 44100.0) {
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

        let deserialized = Looper::from_serialized(&state, dir.path()).unwrap();

        assert_eq!(looper.id, deserialized.id);
        assert_eq!(2, deserialized.samples.len());

        for i in 0..input_left.len() {
            assert!(
                (looper.samples[0].buffer[0][i] - deserialized.samples[0].buffer[0][i]).abs()
                    < 0.00001
            );
            assert!(
                (looper.samples[0].buffer[1][i] - deserialized.samples[0].buffer[1][i]).abs()
                    < 0.00001
            );

            assert!(
                (looper.samples[1].buffer[0][i] - deserialized.samples[1].buffer[0][i]).abs()
                    < 0.00001
            );
            assert!(
                (looper.samples[1].buffer[0][i] - deserialized.samples[1].buffer[1][i]).abs()
                    < 0.00001
            );
        }
    }
}

const CROSS_FADE_SAMPLES: usize = 8192;

struct StateMachine {
    transitions: Vec<(
        Vec<LooperMode>,
        Vec<LooperMode>,
        for<'r> fn(&'r mut Looper, LooperMode) -> LooperMode,
    )>,
}

impl StateMachine {
    fn new() -> StateMachine {
        use LooperMode::*;
        StateMachine {
            transitions: vec![
                (vec![Record, Overdub], vec![], Looper::handle_crossfades),
                (vec![], vec![Overdub], Looper::prepare_for_overdubbing),
                (vec![], vec![Record], Looper::prepare_for_recording),
            ],
        }
    }

    fn handle_transition(&self, looper: &mut Looper, next_state: LooperMode) {
        let cur = looper.mode;
        let mut next_state = next_state;
        for transition in &self.transitions {
            if (transition.0.is_empty() || transition.0.contains(&cur))
                && (transition.1.is_empty() || transition.1.contains(&next_state))
            {
                next_state = transition.2(looper, next_state);
            }
        }
        looper.mode = next_state;
    }
}

lazy_static! {
    static ref STATE_MACHINE: StateMachine = StateMachine::new();
}

// The Looper struct encapsulates behavior similar to a single hardware looper. Internally, it is
// driven by a state machine, which controls how it responds to input buffers (e.g., by recording
// or overdubbing to its internal buffers) and output buffers (e.g., by playing).
pub struct Looper {
    pub id: u32,
    pub samples: Vec<Sample>,
    pub mode: LooperMode,
    pub deleted: bool,
    input_buffer: Sample,
    input_buffer_idx: usize,
    xfade_samples_left: usize,
    xfade_sample_idx: usize,
}

impl Looper {
    pub fn new(id: u32) -> Looper {
        use LooperMode::*;

        let looper = Looper {
            id,
            samples: vec![],
            mode: None,
            deleted: false,
            // input buffer is used to record _before_ actual recording starts, and will be xfaded
            // with the end of the actual sample
            input_buffer: Sample::with_size(CROSS_FADE_SAMPLES),
            input_buffer_idx: 0,
            // xfade samples are recorded _after_ actual recording ends, and are xfaded immediately
            // with the beginning of the actual sample
            xfade_samples_left: 0,
            xfade_sample_idx: 0,
        };

        looper
    }
}

impl Looper {
    // state transition functions
    fn handle_crossfades(&mut self, next_state: LooperMode) -> LooperMode {
        self.xfade_samples_left = CROSS_FADE_SAMPLES;
        self.xfade_sample_idx = self.samples.len() - 1;

        // handle fading the pre-recorded samples (stored in input buffer) with the _end_ of the
        // actual loop
        if let Some(s) = self.samples.last_mut() {
            let size = self.input_buffer_idx.min(CROSS_FADE_SAMPLES);
            if let Some(write_start) = s.length().checked_sub(size as u64) {
                // TODO: I'm sure there's a way to do this without allocating
                let mut left = vec![0f32; size];
                let mut right = vec![0f32; size];

                let len = self.input_buffer.length();
                let read_start =
                    (self.input_buffer_idx as i64 - size as i64).rem_euclid(len as i64) as usize;

                for i in 0..size {
                    left[i] = self.input_buffer.buffer[0][(i + read_start) % len as usize];
                    right[i] = self.input_buffer.buffer[1][(i + read_start) % len as usize];
                }

                s.xfade(
                    CROSS_FADE_SAMPLES,
                    0,
                    write_start,
                    &[&left, &right],
                    XfadeDirection::IN,
                    sample::norm,
                );
            } else {
                println!("Couldn't post crossfade because start was wrong");
            }
        }

        self.input_buffer.clear();
        self.input_buffer_idx = 0;

        next_state
    }

    fn prepare_for_recording(&mut self, next_state: LooperMode) -> LooperMode {
        self.samples.clear();
        self.samples.push(Sample::new());
        next_state
    }

    fn prepare_for_overdubbing(&mut self, next_state: LooperMode) -> LooperMode {
        let len = self.length_in_samples();
        if len == 0 {
            println!("trying to move to overdub with 0-length looper");
            return LooperMode::Record;
        }

        let overdub_sample = Sample::with_size(len as usize);

        // TODO: currently, overdub buffers coming from record are not properly crossfaded until
        //       overdubbing is finished
        // if we're currently recording, we will start our sample off with a crossfade from
        // 0 to the stuff we just recorded. this will be further crossfaded
        // if self.mode == LooperMode::Record {
        //     if let Some(s) = self.samples.last() {
        //         let count = len.min(CROSS_FADE_SAMPLES as u64) as usize;
        //         let range = len as usize - count..len as usize;
        //         assert_eq!(range.len(), count);
        //         self.input_buffer.replace(self.xfade_sample_idx as u64,
        //                                   &[&(&s.buffer[0])[range.clone()],
        //                                       &(&s.buffer[1])[range]]);
        //         self.input_buffer_idx += count;
        //     } else {
        //         println!("no previous sample when moving to overdub!");
        //     }
        // }

        self.samples.push(overdub_sample);
        next_state
    }

    pub fn transition_to(&mut self, mode: LooperMode) {
        println!("Transition {:?} to {:?}", self.mode, mode);

        if self.mode == mode {
            // do nothing if we're not changing state
            return;
        }

        STATE_MACHINE.handle_transition(self, mode);
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
            let s = self
                .samples
                .last_mut()
                .expect("No samples for looper in overdub mode");
            s.overdub(time_in_samples % len, inputs);
        } else if self.mode == LooperMode::Record {
            // in record mode, we extend the current buffer with the new samples
            let s = self
                .samples
                .last_mut()
                .expect("No samples for looper in record mode");
            s.record(inputs);
        } else {
            // record to our circular input buffer, which will be used to cross-fade the end
            self.input_buffer
                .replace(self.input_buffer_idx as u64, inputs);
            self.input_buffer_idx += inputs[0].len();
        }

        // after recording finishes, cross fade some samples with the beginning of the loop to
        // reduce popping
        if self.xfade_samples_left > 0 {
            if let Some(s) = self.samples.get_mut(self.xfade_sample_idx) {
                // this assumes that things are sample-aligned
                s.xfade(
                    CROSS_FADE_SAMPLES,
                    CROSS_FADE_SAMPLES as u64 - self.xfade_samples_left as u64,
                    (CROSS_FADE_SAMPLES - self.xfade_samples_left) as u64,
                    inputs,
                    XfadeDirection::OUT,
                    sample::norm,
                );
                self.xfade_samples_left =
                    (self.xfade_samples_left as i64 - inputs[0].len() as i64).max(0) as usize;
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
            let mut writer = hound::WavWriter::create(&p, spec.clone())?;

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
        let mut looper = Looper::new(state.id);

        for sample_path in &state.samples {
            let mut reader = hound::WavReader::open(&path.join(sample_path))?;

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
