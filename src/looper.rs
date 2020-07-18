use crate::error::SaveLoadError;
use crate::music::FrameTime;
use crate::protos::{LooperMode, SavedLooper};
use crate::sample;
use crate::sample::{Sample, XfadeDirection};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use crossbeam_queue::ArrayQueue;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    use tempfile::tempdir;

    fn install_test_logger() {
        let _ = fern::Dispatch::new()
            .level(log::LevelFilter::Debug)
            .chain(fern::Output::call(|record| println!("{}", record.args())))
            .apply();
    }

    fn process_until_done(looper: &mut Looper) {
        loop {
            let msg = looper.backend.as_mut().unwrap().channel.try_recv();
            match msg {
                Ok(msg) => looper.backend.as_mut().unwrap().handle_msg(msg),
                Err(_) => break,
            };
        }
    }

    fn verify_mode(looper: &Looper, expected: LooperMode) {
        assert_eq!(
            looper.backend.as_ref().unwrap().mode,
            expected,
            "backend in unexpected state"
        );
        assert_eq!(looper.mode, expected, "looper in unexpected state");
    }

    fn verify_length(looper: &Looper, expected: u64) {
        assert_eq!(
            looper.backend.as_ref().unwrap().length_in_samples(),
            expected,
            "backend has unexpected length"
        );
        assert_eq!(
            looper.length_in_samples, expected,
            "looper has unexpected length"
        );
    }

    #[test]
    fn test_transfer_buf() {
        let mut t = TransferBuf {
            id: 0,
            time: FrameTime(12),
            size: 22,
            data: [[0i32; TRANSFER_BUF_SIZE]; 2],
        };

        for i in 0usize..22 {
            t.data[0][i] = i as i32 + 1;
            t.data[1][i] = -(i as i32 + 1);
        }

        assert!(!t.contains_t(FrameTime(0)));
        assert!(!t.contains_t(FrameTime(11)));

        assert!(t.contains_t(FrameTime(12)));
        assert!(t.contains_t(FrameTime(33)));

        assert!(!t.contains_t(FrameTime(34)));

        assert_eq!(Some((1, -1)), t.get_t(FrameTime(12)));
        assert_eq!(Some((22, -22)), t.get_t(FrameTime(33)));
    }

    #[test]
    fn test_new() {
        install_test_logger();

        let looper = Looper::new(1);
        verify_mode(&looper, LooperMode::None);
        assert_eq!(1, looper.id);
        assert_eq!(0, looper.length_in_samples());
    }

    #[test]
    fn test_transitions() {
        install_test_logger();

        let mut looper = Looper::new(1);

        verify_mode(&looper, LooperMode::None);

        looper.transition_to(LooperMode::Record);
        process_until_done(&mut looper);
        verify_mode(&looper, LooperMode::Record);
        assert_eq!(1, looper.backend.as_ref().unwrap().samples.len());

        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        looper.process_input(0, &[&data[0], &data[1]]);
        process_until_done(&mut looper);
        looper.transition_to(LooperMode::Overdub);
        process_until_done(&mut looper);

        assert_eq!(2, looper.backend.as_ref().unwrap().samples.len());
        for s in &looper.backend.as_ref().unwrap().samples {
            assert_eq!(2, s.length());
        }

        looper.transition_to(LooperMode::Playing);
        process_until_done(&mut looper);
        verify_mode(&looper, LooperMode::Playing);

        looper.transition_to(LooperMode::Record);
        process_until_done(&mut looper);
        assert_eq!(1, looper.backend.as_ref().unwrap().samples.len());
        verify_length(&looper, 0);
    }

    #[test]
    fn test_io() {
        install_test_logger();

        let mut l = Looper::new(1);
        l.transition_to(LooperMode::Record);
        process_until_done(&mut l);

        let input_left = vec![1f32, 2.0, 3.0, 4.0];
        let input_right = vec![-1f32, -2.0, -3.0, -4.0];
        l.process_input(0, &[&input_left, &input_right]);
        process_until_done(&mut l);

        let output_left = vec![1f64; TRANSFER_BUF_SIZE];
        let output_right = vec![-1f64; TRANSFER_BUF_SIZE];

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        let mut o = [output_left, output_right];

        l.process_output(FrameTime(input_left.len() as i64), &mut o);
        process_until_done(&mut l);
        assert_eq!([2.0f64, 3.0, 4.0, 5.0, 2.0, 3.0], o[0][0..6]);
        assert_eq!([-2.0f64, -3.0, -4.0, -5.0, -2.0, -3.0], o[1][0..6]);
    }

    #[test]
    fn test_non_harmonious_lengths() {
        install_test_logger();

        // ensure that everything works correctly when our looper length is not a multiple of the
        // buffer size or our TRANSFER_BUF_SIZE

        let buf_size = 128;

        let mut l = Looper::new(2);
        l.transition_to(LooperMode::Record);

        let mut input_left = vec![1f32; buf_size];
        let mut input_right = vec![-1f32; buf_size];

        let mut time = 0i64;

        l.process_input(0, &[&input_left, &input_right]);
        process_until_done(&mut l);

        let mut o = [vec![0f64; buf_size], vec![0f64; buf_size]];
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);
        time += buf_size as i64;

        // first give the part before the state change (which will be recorded)
        input_left = vec![2f32; buf_size];
        input_right = vec![-2f32; buf_size];
        l.process_input(time as u64, &[&input_left[0..100], &input_right[0..100]]);
        process_until_done(&mut l);

        // then transition
        l.transition_to(LooperMode::Overdub);
        process_until_done(&mut l);

        let len = buf_size + 100;
        verify_length(&l, len as u64);

        // then give the rest and process output
        l.process_input(time as u64, &[&input_left[100..buf_size],
            &input_right[100..buf_size]]);
        process_until_done(&mut l);

        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);

        for i in 0..buf_size {
            let t = time as usize + i;
            if t < buf_size + 100 {
                assert_eq!(o[0][i], 0f64);
                assert_eq!(o[1][i], 0f64);
            } else if t % len < buf_size  {
                assert_eq!(o[0][i], 1f64);
                assert_eq!(o[1][i], -1f64);
            } else {
                assert_eq!(o[0][i], 2f64);
                assert_eq!(o[1][i], -2f64);

            }
        }

        time += buf_size as i64;

        o = [vec![0f64; buf_size], vec![0f64; buf_size]];
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);

        for i in 0..buf_size {
            let t = time as usize + i;
            if t % len < buf_size {
                assert_eq!(o[0][i], 1f64);
                assert_eq!(o[1][i], -1f64);
            } else {
                assert_eq!(o[0][i], 2f64);
                assert_eq!(o[1][i], -2f64);

            }
        }
    }

    #[test]
    fn test_post_xfade() {
        install_test_logger();

        let mut l = Looper::new(1);
        l.transition_to(LooperMode::Record);
        process_until_done(&mut l);

        let mut time = 0i64;

        let mut input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        let mut input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];
        let mut o = [
            vec![0f64; CROSS_FADE_SAMPLES * 2],
            vec![0f64; CROSS_FADE_SAMPLES * 2],
        ];

        l.process_input(time as u64, &[&input_left, &input_right]);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);
        time += input_left.len() as i64;

        for i in 0..CROSS_FADE_SAMPLES {
            let q = i as f32 / CROSS_FADE_SAMPLES as f32;
            input_left[i] = -q / (1f32 - q);
            input_right[i] = q / (1f32 - q);
        }

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        for i in (0..CROSS_FADE_SAMPLES * 2).step_by(32) {
            l.process_input(
                time as u64,
                &[&input_left[i..i + 32], &input_right[i..i + 32]],
            );
            process_until_done(&mut l);

            let mut o = [vec![0f64; 32], vec![0f64; 32]];
            l.process_output(FrameTime(time), &mut o);
            process_until_done(&mut l);

            time += 32;
        }

        let mut o = [
            vec![0f64; CROSS_FADE_SAMPLES * 2],
            vec![0f64; CROSS_FADE_SAMPLES * 2],
        ];

        l.process_input(time as u64, &[&input_left, &input_right]);
        process_until_done(&mut l);

        l.process_output(FrameTime(time), &mut o);

        verify_length(&l, CROSS_FADE_SAMPLES as u64 * 2);

        for i in 0..o[0].len() {
            if i < CROSS_FADE_SAMPLES {
                assert!(
                    (0f64 - o[0][i]).abs() < 0.000001,
                    "left is {} at idx {}, expected 0",
                    o[0][i],
                    time + i as i64
                );
                assert!(
                    (0f64 - o[1][i]).abs() < 0.000001,
                    "right is {} at idx {}, expected 0",
                    o[1][i],
                    time + i as i64
                );
            } else {
                assert_eq!(1f64, o[0][i], "mismatch at {}", time + i as i64);
                assert_eq!(-1f64, o[1][i], "mismatch at {}", time + i as i64);
            }
        }
    }

    #[test]
    fn test_pre_xfade() {
        install_test_logger();

        let mut l = Looper::new(1);

        let mut input_left = vec![17f32; CROSS_FADE_SAMPLES];
        let mut input_right = vec![-17f32; CROSS_FADE_SAMPLES];

        let mut time = 0i64;
        // process some random input
        for i in (0..CROSS_FADE_SAMPLES).step_by(32) {
            l.process_input(
                time as u64,
                &[&input_left[i..i + 32], &input_right[i..i + 32]],
            );
            process_until_done(&mut l);

            let mut o = [vec![0f64; 32], vec![0f64; 32]];
            l.process_output(FrameTime(time), &mut o);
            process_until_done(&mut l);

            time += 32;
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
        for i in (0..CROSS_FADE_SAMPLES).step_by(32) {
            l.process_input(
                time as u64,
                &[&input_left[i..i + 32], &input_right[i..i + 32]],
            );
            process_until_done(&mut l);

            let mut o = [vec![0f64; 32], vec![0f64; 32]];
            l.process_output(FrameTime(time), &mut o);
            process_until_done(&mut l);

            time += 32;
        }

        l.transition_to(LooperMode::Record);
        process_until_done(&mut l);

        input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];

        let mut o = [
            vec![0f64; CROSS_FADE_SAMPLES * 2],
            vec![0f64; CROSS_FADE_SAMPLES * 2],
        ];

        l.process_input(time as u64, &[&input_left, &input_right]);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);
        time += input_left.len() as i64;

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        // Go around again (we don't have the crossfaded samples until the second time around)
        l.process_input(time as u64, &[&input_left, &input_right]);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);
        time += input_left.len() as i64;

        let mut o = [
            vec![0f64; CROSS_FADE_SAMPLES * 2],
            vec![0f64; CROSS_FADE_SAMPLES * 2],
        ];
        l.process_output(FrameTime(time), &mut o);
        process_until_done(&mut l);

        for i in 0..o[0].len() {
            if i > CROSS_FADE_SAMPLES {
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
        install_test_logger();

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

        let mut l = Looper::new(5);

        l.transition_to(LooperMode::Record);
        process_until_done(&mut l);
        l.process_input(0, &[&input_left, &input_right]);
        process_until_done(&mut l);

        l.transition_to(LooperMode::Overdub);
        process_until_done(&mut l);
        l.process_input(0, &[&input_left2, &input_right2]);
        process_until_done(&mut l);

        let (tx, rx) = bounded(1);
        l.channel()
            .send(ControlMessage::Serialize(dir.path().to_path_buf(), tx))
            .unwrap();
        process_until_done(&mut l);

        let state = rx.recv().unwrap().unwrap();

        let deserialized = Looper::from_serialized(&state, dir.path()).unwrap();

        assert_eq!(l.id, deserialized.id);

        let b1 = l.backend.as_ref().unwrap();
        let b2 = deserialized.backend.as_ref().unwrap();

        assert_eq!(2, b2.samples.len());

        for i in 0..input_left.len() {
            assert!((b1.samples[0].buffer[0][i] - b2.samples[0].buffer[0][i]).abs() < 0.00001);
            assert!((b1.samples[0].buffer[1][i] - b2.samples[0].buffer[1][i]).abs() < 0.00001);

            assert!((b1.samples[1].buffer[0][i] - b2.samples[1].buffer[0][i]).abs() < 0.00001);
            assert!((b1.samples[1].buffer[0][i] - b2.samples[1].buffer[1][i]).abs() < 0.00001);
        }
    }
}

const CROSS_FADE_SAMPLES: usize = 64; //8192;

struct StateMachine {
    transitions: Vec<(
        Vec<LooperMode>,
        Vec<LooperMode>,
        for<'r> fn(&'r mut LooperBackend, LooperMode),
    )>,
}

impl StateMachine {
    fn new() -> StateMachine {
        use LooperMode::*;
        StateMachine {
            transitions: vec![
                (
                    vec![Record],
                    vec![],
                    LooperBackend::finish_recording,
                ),
                (
                    vec![Record, Overdub],
                    vec![],
                    LooperBackend::handle_crossfades,
                ),
                (
                    vec![],
                    vec![Overdub],
                    LooperBackend::prepare_for_overdubbing,
                ),
                (vec![], vec![Record], LooperBackend::prepare_for_recording),
            ],
        }
    }

    fn handle_transition(&self, looper: &mut LooperBackend, next_state: LooperMode) {
        let cur = looper.mode;
        for transition in &self.transitions {
            if (transition.0.is_empty() || transition.0.contains(&cur))
                && (transition.1.is_empty() || transition.1.contains(&next_state))
            {
                transition.2(looper, next_state);
            }
        }
        looper.mode = next_state;
    }
}

lazy_static! {
    static ref STATE_MACHINE: StateMachine = StateMachine::new();
}

#[derive(Debug)]
pub enum ControlMessage {
    InputDataReady { id: u64, size: usize },
    TransitionTo(LooperMode),
    SetTime(FrameTime),
    ReadOutput(FrameTime),
    Shutdown,
    Serialize(PathBuf, Sender<Result<SavedLooper, SaveLoadError>>),
}

const TRANSFER_BUF_SIZE: usize = 32;

#[derive(Clone, Copy)]
struct TransferBuf<DATA: Copy> {
    id: u64,
    time: FrameTime,
    size: usize,
    data: [[DATA; TRANSFER_BUF_SIZE]; 2],
}

impl <DATA: Copy> TransferBuf<DATA> {
    pub fn contains_t(&self, time: FrameTime) -> bool {
        return time.0 >= self.time.0 && time.0 < self.time.0 + self.size as i64;
    }

    pub fn get_t(&self, time: FrameTime) -> Option<(DATA, DATA)> {
        if self.contains_t(time) {
            let idx = (time.0 - self.time.0) as usize;
            Some((self.data[0][idx], self.data[1][idx]))
        } else {
            None
        }
    }
}

struct LooperBackend {
    pub id: u32,
    pub samples: Vec<Sample>,
    pub mode: LooperMode,
    pub deleted: bool,

    out_time: FrameTime,
    in_time: FrameTime,

    input_buffer: Sample,
    input_buffer_idx: usize,
    xfade_samples_left: usize,
    xfade_sample_idx: usize,

    in_queue: Arc<ArrayQueue<TransferBuf<f32>>>,
    out_queue: Arc<ArrayQueue<TransferBuf<f64>>>,

    channel: Receiver<ControlMessage>,
}

impl LooperBackend {
    fn start(mut self) {
        thread::spawn(move || loop {
            match self.channel.recv() {
                Ok(msg) => {
                    if !self.handle_msg(msg) {
                        break;
                    }
                }
                Err(_) => {
                    info!("Channel closed, stopping");
                    break;
                }
            }
        });
    }

    fn handle_msg(&mut self, msg: ControlMessage) -> bool /* continue */ {
        info!("[{}] got control message: {:?}", self.id, msg);
        match msg {
            ControlMessage::InputDataReady { id, size } => {
                let mut read = 0;
                let mut processing = false;
                while read < size {
                    let buf = self
                        .in_queue
                        .pop()
                        .expect("missing expected data from queue");
                    if buf.id == id {
                        processing = true;
                    } else if processing {
                        assert_eq!(read, size, "did not find enough values in input data!");
                        break;
                    } else {
                        warn!(
                            "Skipping unexpected input data in looper {}: \
                               got {}, but waiting for {}",
                            self.id, buf.id, id
                        );
                    }

                    if processing {
                        let to_read = (size - read).min(buf.size);
                        read += to_read;
                        self.handle_input(
                            buf.time.0 as u64,
                            &[&buf.data[0][0..to_read], &buf.data[1][0..to_read]],
                        );
                    }
                }
            }
            ControlMessage::TransitionTo(mode) => {
                self.transition_to(mode);
            }
            ControlMessage::SetTime(time) => {
                self.out_time = time;
                self.in_time = time;
            }
            ControlMessage::ReadOutput(time) => {
                self.in_time = time;
            }
            ControlMessage::Shutdown => {
                info!("Got shutdown message, stopping");
                return false;
            }
            ControlMessage::Serialize(path, channel) => {
                let result = self.serialize(&path);
                if let Err(e) = channel.try_send(result) {
                    warn!("failed to respond to serialize request: {:?}", e);
                }
            }
        }

        self.fill_output();
        true
    }

    fn fill_output(&mut self) {
        let sample_len = self.length_in_samples() as usize;
        // don't fill the output if we're in record mode, because we don't know our length. the
        // timing won't be correct if we wrap around.
        if sample_len > 0 && self.mode != LooperMode::Record && self.out_time.0 >= 0 {
            // make sure we don't pass our input
            while self.out_time.0 < self.in_time.0 + sample_len as i64 {
                let mut buf = TransferBuf {
                    id: 0,
                    time: self.out_time,
                    size: TRANSFER_BUF_SIZE,
                    data: [[0f64; TRANSFER_BUF_SIZE]; 2],
                };

                for sample in &self.samples {
                    let b = &sample.buffer;
                    if b[0].is_empty() {
                        continue;
                    }

                    for i in 0..2 {
                        for t in 0..TRANSFER_BUF_SIZE {
                            buf.data[i][t] +=
                                b[i][(self.out_time.0 as usize + t) % sample_len] as f64;
                        }
                    }
                }

                if self.out_queue.push(buf).is_err() {
                    break;
                }

                debug!(
                    "[OUTPUT] t = {} [{}; {}] (in time = {})",
                    self.out_time.0, buf.data[0][0], TRANSFER_BUF_SIZE, self.in_time.0
                );

                self.out_time.0 += TRANSFER_BUF_SIZE as i64;
            }
        }
    }

    fn finish_recording(&mut self, _: LooperMode) {
        // update our out time to the loop length so that we don't bother outputting a bunch of
        // of wasted data
        self.out_time = FrameTime(self.length_in_samples() as i64);
    }

    // state transition functions
    fn handle_crossfades(&mut self, _next_state: LooperMode) {
        debug!("handling crossfade");
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
                warn!("Couldn't post crossfade because start was wrong");
            }
        }

        self.input_buffer.clear();
        self.input_buffer_idx = 0;
    }

    fn prepare_for_recording(&mut self, _: LooperMode) {
        self.samples.clear();
        self.samples.push(Sample::new());
    }

    fn prepare_for_overdubbing(&mut self, _next_state: LooperMode) {
        let overdub_sample = Sample::with_size(self.length_in_samples() as usize);

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
        //         debug!("no previous sample when moving to overdub!");
        //     }
        // }

        self.samples.push(overdub_sample);
    }

    pub fn transition_to(&mut self, mode: LooperMode) {
        debug!("Transition {:?} to {:?}", self.mode, mode);
        self.fill_output();

        if self.mode == mode {
            // do nothing if we're not changing state
            return;
        }

        STATE_MACHINE.handle_transition(self, mode);
    }

    fn handle_input(&mut self, time_in_samples: u64, inputs: &[&[f32]]) {
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
            debug!("crossfading beginning at time {}", time_in_samples);
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
                debug!("tried to cross fade but no samples... something is likely wrong");
            }
        }

        self.in_time = FrameTime(time_in_samples as i64 + inputs[0].len() as i64);
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
}

// The Looper struct encapsulates behavior similar to a single hardware looper. Internally, it is
// driven by a state machine, which controls how it responds to input buffers (e.g., by recording
// or overdubbing to its internal buffers) and output buffers (e.g., by playing).
pub struct Looper {
    pub id: u32,
    pub mode: LooperMode,
    pub deleted: bool,

    backend: Option<LooperBackend>,
    length_in_samples: u64,
    msg_counter: u64,
    out_queue: Arc<ArrayQueue<TransferBuf<f32>>>,
    in_queue: Arc<ArrayQueue<TransferBuf<f64>>>,
    channel: Sender<ControlMessage>,

    in_progress_output: Option<TransferBuf<f64>>,
}

impl Looper {
    pub fn new(id: u32) -> Looper {
        Self::new_with_samples(id, vec![])
    }

    pub fn channel(&self) -> Sender<ControlMessage> {
        self.channel.clone()
    }

    fn new_with_samples(id: u32, samples: Vec<Sample>) -> Looper {
        let record_queue = Arc::new(ArrayQueue::new(512 * 1024 / TRANSFER_BUF_SIZE));
        let play_queue = Arc::new(ArrayQueue::new(512 * 1024 / TRANSFER_BUF_SIZE));

        let (s, r) = bounded(100);

        let backend = LooperBackend {
            id,
            samples,
            mode: LooperMode::None,
            deleted: false,
            out_time: FrameTime(0),
            in_time: FrameTime(0),
            // input buffer is used to record _before_ actual recording starts, and will be xfaded
            // with the end of the actual sample
            input_buffer: Sample::with_size(CROSS_FADE_SAMPLES),
            input_buffer_idx: 0,
            // xfade samples are recorded _after_ actual recording ends, and are xfaded immediately
            // with the beginning of the actual sample
            xfade_samples_left: 0,
            xfade_sample_idx: 0,
            in_queue: record_queue.clone(),
            out_queue: play_queue.clone(),
            channel: r,
        };

        // backend.start();

        let looper = Looper {
            id,
            backend: Some(backend),
            mode: LooperMode::None,
            deleted: false,
            length_in_samples: 0,
            msg_counter: 0,
            in_queue: play_queue.clone(),
            out_queue: record_queue.clone(),
            channel: s,

            in_progress_output: None,
        };

        looper
    }

    pub fn start(mut self) -> Self {
        let mut backend: Option<LooperBackend> = None;
        std::mem::swap(&mut backend, &mut self.backend);

        match backend {
            Some(backend) => backend.start(),
            _ => warn!("looper already started!"),
        }

        self
    }

    pub fn from_serialized(state: &SavedLooper, path: &Path) -> Result<Looper, SaveLoadError> {
        let mut samples = vec![];
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
            samples.push(sample);
        }

        Ok(Self::new_with_samples(state.id, samples))
    }
}

impl Looper {
    pub fn set_time(&mut self, time: FrameTime) {
        loop {
            if let Err(_) = self.in_queue.pop() {
                break;
            }
        }
        self.channel
            .send(ControlMessage::SetTime(time))
            .expect("channel closed");
    }

    fn output_for_t(&mut self, t: FrameTime) -> Option<(f64, f64)> {
        let mut cur = self.in_progress_output.or_else(|| self.in_queue.pop().ok())?;
        self.in_progress_output = Some(cur);

        loop {
            if cur.time.0 > t.0 {
                debug!(
                    "data is in future for looper id {} (time is {}, needed {})",
                    self.id, cur.time.0, t.0
                );
                return None;
            }

            if let Some(o) = cur.get_t(t) {
                return Some(o);
            }

            if let Ok(buf) = self.in_queue.pop() {
                cur = buf;
                self.in_progress_output = Some(buf);
            } else {
                self.in_progress_output = None;
                return None;
            }
        }
    }

    // In process_output, we modify the specified output buffers according to our internal state. In
    // Playing or Overdub mode, we will add our buffer to the output. Otherwise, we do nothing.
    pub fn process_output(&mut self, time: FrameTime, outputs: &mut [Vec<f64>; 2]) {
        if time.0 < 0 {
            return;
        }

        debug!("reading time {}", time.0);

        let mut time = time;
        let mut out_idx = 0;

        // let mut in_progress = None;
        // std::mem::swap(&mut self.in_progress_output, &mut in_progress);
        //
        // in_progress = in_progress.or_else(|| self.in_queue.pop().ok());

        let mut missing = 0;
        while out_idx < outputs[0].len() {
            if let Some((l, r)) = self.output_for_t(time) {
                if self.mode == LooperMode::Playing || self.mode == LooperMode::Overdub {
                    outputs[0][out_idx] += l;
                    outputs[1][out_idx] += r;
                }
            } else {
                missing += 1;
            }
            out_idx += 1;
            time.0 += 1;
        }

        if self.mode != LooperMode::Record && missing > 0 {
            error!("needed output but queue was empty in looper {} (missed {} samples)",
                   self.id, missing);
        }

        // while out_idx < outputs[0].len() {
        //     if let Some(buf) = in_progress.or_else(|| self.in_queue.pop().ok()) {
        //         if self.mode == LooperMode::Playing || self.mode == LooperMode::Overdub {
        //             if (buf.time.0 + buf.size as i64) < time {
        //                 debug!(
        //                     "skipping old data for looper id {} (time is {}, waiting for {})",
        //                     self.id, buf.time.0, time
        //                 );
        //             } else if buf.time.0 > time {
        //                 error!(
        //                     "data is in future for looper id {} (time is {}, needed {})",
        //                     self.id, buf.time.0, time
        //                 );
        //                 break;
        //             } else {
        //                 assert!(buf.time.0 <= time);
        //                 let still_needed = outputs[0].len() - out_idx;
        //                 let start = (time - buf.time.0) as usize;
        //                 let end = (start + still_needed).min(buf.size);
        //
        //                 debug!("outputting time {}", buf.time.0 + start as i64);
        //                 for c in 0..2 {
        //                     let mut o_i = out_idx;
        //                     for i in start..end {
        //                         outputs[c][o_i] += buf.data[c][i] as f64;
        //                         o_i += 1;
        //                     }
        //                 }
        //
        //                 time += (end - start) as i64;
        //                 out_idx += end - start;
        //
        //                 if end < buf.size {
        //                     // we still have data to be read from this buffer for the next iteration
        //                     self.in_progress_output = Some(buf);
        //                 }
        //             }
        //         }
        //     } else {
        //         // TODO: handle missing data
        //         if self.mode != LooperMode::Record {
        //             error!("needed output but queue was empty in looper {}", self.id);
        //         }
        //         break;
        //     }
        // }

        match self.channel.try_send(ControlMessage::ReadOutput(time)) {
            Err(TrySendError::Disconnected(_)) => panic!("channel closed"),
            Err(TrySendError::Full(_)) => warn!("channel full while requesting more output"),
            _ => {}
        }
    }

    // In process_input, we modify our internal buffers based on the input. In Record mode, we
    // append the data in the input buffers to our current sample. In Overdub mode, we sum the data
    // with whatever is currently in our buffer at the point of time_in_samples.
    pub fn process_input(&mut self, time_in_samples: u64, inputs: &[&[f32]]) {
        assert_eq!(2, inputs.len());

        let msg_id = self.msg_counter;
        self.msg_counter += 1;

        if self.mode == LooperMode::Record {
            // TODO: would be nice to try to find some way to verify this stays in sync with
            //       the backend
            self.length_in_samples += inputs[0].len() as u64;
        }

        let mut buf = TransferBuf {
            id: msg_id,
            time: FrameTime(0 as i64),
            size: 0,
            data: [[0f32; TRANSFER_BUF_SIZE]; 2],
        };

        let mut time = time_in_samples;
        for (l, r) in inputs[0]
            .chunks(TRANSFER_BUF_SIZE)
            .zip(inputs[1].chunks(TRANSFER_BUF_SIZE))
        {
            buf.time = FrameTime(time as i64);
            buf.size = l.len();

            for i in 0..l.len() {
                buf.data[0][i] = l[i];
                buf.data[1][i] = r[i];
            }

            if let Err(_) = self.out_queue.push(buf) {
                // TODO: handle error case where our queue is full
                error!("queue is full on looper {}", self.id);
            }

            time += TRANSFER_BUF_SIZE as u64;
        }

        self.channel
            .send(ControlMessage::InputDataReady {
                id: msg_id,
                size: inputs[0].len(),
            })
            .expect("channel is closed!");
    }

    pub fn transition_to(&mut self, mode: LooperMode) {
        let mut mode = mode;
        if self.length_in_samples() == 0 {
            warn!("trying to move to overdub with 0-length looper");
            mode = LooperMode::Record;
        }

        // TODO: maybe want to turn this behavior into an explicit "reset" command
        if self.mode != LooperMode::Record && mode == LooperMode::Record {
            self.length_in_samples = 0;
        }

        self.channel
            .send(ControlMessage::TransitionTo(mode))
            .expect("channel is closed!");
        self.mode = mode;
    }

    pub fn length_in_samples(&self) -> u64 {
        self.length_in_samples
    }
}

impl Drop for Looper {
    fn drop(&mut self) {
        if let Err(_) = self.channel.send(ControlMessage::Shutdown) {
            warn!("failed to shutdown backend because queue was full");
        }
    }
}
