use crate::sample;
use crate::sample::{Sample, XfadeDirection};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use crossbeam_queue::ArrayQueue;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crate::error::SaveLoadError;
use loopers_common::api::{
    FrameTime, LooperCommand, LooperMode, LooperSpeed, Part, PartSet, SavedLooper,
};
use loopers_common::gui_channel::GuiCommand::{AddNewSample, AddOverdubSample};
use loopers_common::gui_channel::{
    GuiCommand, GuiSender, LooperState, Waveform, WAVEFORM_DOWNSAMPLE,
};
use loopers_common::music::PanLaw;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::mem::swap;

use atomic::Atomic;
use std::sync::atomic::Ordering;


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
            looper.backend.as_ref().unwrap().mode(),
            expected,
            "backend in unexpected state"
        );
        assert_eq!(looper.mode(), expected, "looper in unexpected state");
    }

    fn verify_length(looper: &Looper, expected: u64) {
        assert_eq!(
            looper.backend.as_ref().unwrap().length_in_samples(false),
            expected,
            "backend has unexpected length"
        );
        assert_eq!(
            looper.length(), expected,
            "looper has unexpected length"
        );
    }

    fn looper_for_test() -> Looper {
        let mut l = Looper::new(1, PartSet::new(), GuiSender::disconnected());
        l.pan_law = PanLaw::Transparent;
        l.backend.as_mut().unwrap().enable_crossfading = false;
        l
    }

    #[test]
    fn test_transfer_buf() {
        let mut t = TransferBuf {
            id: 0,
            time: FrameTime(12),
            size: 6,
            data: [[0i32; TRANSFER_BUF_SIZE]; 2],
        };

        for i in 0usize..6 {
            t.data[0][i] = i as i32 + 1;
            t.data[1][i] = -(i as i32 + 1);
        }

        assert!(!t.contains_t(FrameTime(0)));
        assert!(!t.contains_t(FrameTime(11)));

        assert!(t.contains_t(FrameTime(12)));
        assert!(t.contains_t(FrameTime(17)));

        assert!(!t.contains_t(FrameTime(18)));

        assert_eq!(Some((1, -1)), t.get_t(FrameTime(12)));
        assert_eq!(Some((6, -6)), t.get_t(FrameTime(17)));
    }

    #[test]
    fn test_new() {
        install_test_logger();

        let looper = looper_for_test();
        verify_mode(&looper, LooperMode::Playing);
        assert_eq!(1, looper.id);
        assert_eq!(0, looper.length());
    }

    #[test]
    fn test_transitions() {
        install_test_logger();

        let mut looper = looper_for_test();

        verify_mode(&looper, LooperMode::Playing);

        looper.transition_to(LooperMode::Recording);
        process_until_done(&mut looper);
        verify_mode(&looper, LooperMode::Recording);
        assert_eq!(1, looper.backend.as_ref().unwrap().samples.len());

        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        looper.process_input(0, &[&data[0], &data[1]], Part::A);
        process_until_done(&mut looper);
        looper.transition_to(LooperMode::Overdubbing);
        process_until_done(&mut looper);

        assert_eq!(2, looper.backend.as_ref().unwrap().samples.len());
        for s in &looper.backend.as_ref().unwrap().samples {
            assert_eq!(2, s.length());
        }

        looper.transition_to(LooperMode::Playing);
        process_until_done(&mut looper);
        verify_mode(&looper, LooperMode::Playing);

        looper.transition_to(LooperMode::Recording);
        process_until_done(&mut looper);
        assert_eq!(1, looper.backend.as_ref().unwrap().samples.len());
        verify_length(&looper, 0);
    }

    #[test]
    fn test_io() {
        install_test_logger();

        let mut l = looper_for_test();
        l.backend.as_mut().unwrap().enable_crossfading = false;

        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);

        let mut input_left = vec![0f32; TRANSFER_BUF_SIZE];
        let mut input_right = vec![0f32; TRANSFER_BUF_SIZE];
        for i in 0..TRANSFER_BUF_SIZE {
            input_left[i] = i as f32;
            input_right[i] = -(i as f32);
        }

        l.process_input(0, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        let mut o_l = vec![1f64; TRANSFER_BUF_SIZE];
        let mut o_r = vec![-1f64; TRANSFER_BUF_SIZE];

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);
        l.process_input(
            input_left.len() as u64,
            &[&input_left, &input_right],
            Part::A,
        );
        process_until_done(&mut l);

        l.process_output(
            FrameTime(input_left.len() as i64),
            &mut [&mut o_l, &mut o_r],
            Part::A,
            false,
        );
        process_until_done(&mut l);

        for i in 0..TRANSFER_BUF_SIZE {
            assert_eq!(o_l[i], (i + 1) as f64);
            assert_eq!(o_r[i], -((i + 1) as f64));
        }
    }

    #[test]
    fn test_overdub() {
        install_test_logger();

        let mut l = looper_for_test();
        l.backend.as_mut().unwrap().enable_crossfading = false;

        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);

        let mut input_left = vec![0f32; TRANSFER_BUF_SIZE];
        let mut input_right = vec![0f32; TRANSFER_BUF_SIZE];
        for i in 0..TRANSFER_BUF_SIZE {
            input_left[i] = i as f32 + 1.0;
            input_right[i] = -(i as f32 + 1.0);
        }

        let mut t = 0 as i64;

        l.process_input(t as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        let mut o_l = vec![0f64; TRANSFER_BUF_SIZE];
        let mut o_r = vec![0f64; TRANSFER_BUF_SIZE];
        l.process_output(FrameTime(t), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        t += TRANSFER_BUF_SIZE as i64;

        for (l, r) in o_l.iter().zip(&o_r) {
            assert_eq!(*l, 0.0);
            assert_eq!(*r, 0.0);
        }

        l.transition_to(LooperMode::Overdubbing);
        process_until_done(&mut l);

        // first record our overdub
        let mut o_l = vec![0f64; TRANSFER_BUF_SIZE];
        let mut o_r = vec![0f64; TRANSFER_BUF_SIZE];
        l.process_output(FrameTime(t), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        l.process_input(t as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        t += TRANSFER_BUF_SIZE as i64;

        for (i, (l, r)) in o_l.iter().zip(&o_r).enumerate() {
            assert_eq!(*l, (i + 1) as f64);
            assert_eq!(*r, -((i + 1) as f64));
        }

        // on the next go-around, it should be played back
        let mut o_l = vec![0f64; TRANSFER_BUF_SIZE];
        let mut o_r = vec![0f64; TRANSFER_BUF_SIZE];
        l.process_output(FrameTime(t), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        l.process_input(t as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        for (i, (l, r)) in o_l.iter().zip(&o_r).enumerate() {
            let v = ((i + 1) * 2) as f64;
            assert_eq!(*l, v);
            assert_eq!(*r, -v);
        }
    }

    #[test]
    fn test_solo() {
        install_test_logger();

        let mut l = looper_for_test();
        l.transition_to(LooperMode::Recording);

        let input_left = vec![1f32; 128];
        let input_right = vec![-1f32; 128];

        l.process_input(0, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        let mut o_l = vec![0f64; 128];
        let mut o_r = vec![0f64; 128];

        // with solo true and us in Playing, there should be no output
        l.set_time(FrameTime(0));
        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        l.process_output(
            FrameTime(0),
            &mut [&mut o_l, &mut o_r],
            Part::A,
            true,
        );

        for i in 0..128 {
            assert_eq!(0.0, o_l[i]);
            assert_eq!(0.0, o_r[i]);
        }

        // with solo true and us in Solo, there should be output
        l.set_time(FrameTime(0));
        l.transition_to(LooperMode::Soloed);
        process_until_done(&mut l);

        l.process_output(
            FrameTime(0),
            &mut [&mut o_l, &mut o_r],
            Part::A,
            true,
        );

        for i in 0..128 {
            assert_eq!(1.0, o_l[i]);
            assert_eq!(-1.0, o_r[i]);
        }

        // with solo true and us in Solo, but in another part, there should not be outoput
        o_l = vec![0f64; 128];
        o_r = vec![0f64; 128];

        l.set_time(FrameTime(0));
        process_until_done(&mut l);

        l.process_output(
            FrameTime(0),
            &mut [&mut o_l, &mut o_r],
            Part::B,
            true,
        );

        for i in 0..128 {
            assert_eq!(0.0, o_l[i]);
            assert_eq!(0.0, o_r[i]);
        }
    }

    #[test]
    fn test_non_harmonious_lengths() {
        install_test_logger();

        // ensure that everything works correctly when our looper length is not a multiple of the
        // buffer size or our TRANSFER_BUF_SIZE

        let buf_size = 128;

        let mut l = looper_for_test();
        l.transition_to(LooperMode::Recording);

        let mut input_left = vec![1f32; buf_size];
        let mut input_right = vec![-1f32; buf_size];

        let mut time = 0i64;

        let mut o_l = vec![0f64; buf_size];
        let mut o_r = vec![0f64; buf_size];
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        l.process_input(0, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        time += buf_size as i64;

        // first give the part before the state change (which will be recorded)
        l.process_output(
            FrameTime(time),
            &mut [&mut o_l[0..100], &mut o_r[0..100]],
            Part::A,
            false,
        );
        process_until_done(&mut l);

        input_left = vec![2f32; buf_size];
        input_right = vec![-2f32; buf_size];
        l.process_input(
            time as u64,
            &[&input_left[0..100], &input_right[0..100]],
            Part::A,
        );
        process_until_done(&mut l);

        time += 100;

        // then transition
        l.transition_to(LooperMode::Overdubbing);
        process_until_done(&mut l);

        let len = buf_size + 100;
        verify_length(&l, len as u64);

        l.process_output(
            FrameTime(time),
            &mut [&mut o_l[100..buf_size], &mut o_r[100..buf_size]],
            Part::A,
            false,
        );
        process_until_done(&mut l);

        l.process_input(
            time as u64,
            &[&input_left[100..buf_size], &input_right[100..buf_size]],
            Part::A,
        );
        process_until_done(&mut l);

        time += buf_size as i64 - 100;

        for i in 0..buf_size {
            if i < 100 {
                assert_eq!(o_l[i], 0f64);
                assert_eq!(o_r[i], 0f64);
            } else {
                assert_eq!(o_l[i], 1f64);
                assert_eq!(o_r[i], -1f64);
            }
        }

        o_l = vec![0f64; buf_size];
        o_r = vec![0f64; buf_size];
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        for i in 0..buf_size {
            let t = time as usize + i;
            if t % len < buf_size {
                assert_eq!(o_l[i], 1f64);
                assert_eq!(o_r[i], -1f64);
            } else {
                assert_eq!(o_l[i], 2f64);
                assert_eq!(o_r[i], -2f64);
            }
        }

        // now we play from the beginning
        l.set_time(FrameTime(0));
        process_until_done(&mut l);
        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        o_l = vec![0f64; buf_size];
        o_r = vec![0f64; buf_size];
        l.process_output(FrameTime(0), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        for i in 0..buf_size {
            if i < buf_size - 100 {
                assert_eq!(o_l[i], 3f64);
                assert_eq!(o_r[i], -3f64);
            } else {
                assert_eq!(o_l[i], 1.0f64);
                assert_eq!(o_r[i], -1.0f64);
            }
        }
    }

    #[test]
    fn test_offset() {
        install_test_logger();

        let offset = 4u64;

        let mut l = looper_for_test();
        l.backend.as_mut().unwrap().enable_crossfading = false;

        l.set_time(FrameTime(offset as i64));

        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);

        let mut input_left = vec![0f32; TRANSFER_BUF_SIZE];
        let mut input_right = vec![0f32; TRANSFER_BUF_SIZE];
        for i in 0..TRANSFER_BUF_SIZE {
            input_left[i] = i as f32;
            input_right[i] = -(i as f32);
        }

        l.process_input(offset, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        let mut o_l = vec![1f64; TRANSFER_BUF_SIZE];
        let mut o_r = vec![-1f64; TRANSFER_BUF_SIZE];

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);
        l.process_input(
            offset + input_left.len() as u64,
            &[&input_left, &input_right],
            Part::A,
        );
        process_until_done(&mut l);

        l.process_output(
            FrameTime(offset as i64 + input_left.len() as i64),
            &mut [&mut o_l, &mut o_r],
            Part::A,
            false,
        );
        process_until_done(&mut l);

        for i in 0..TRANSFER_BUF_SIZE {
            assert_eq!(o_l[i], (i + 1) as f64);
            assert_eq!(o_r[i], -((i + 1) as f64));
        }
    }

    #[test]
    fn test_post_xfade() {
        install_test_logger();

        let mut l = looper_for_test();
        l.backend.as_mut().unwrap().enable_crossfading = true;
        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);

        let mut time = 0i64;

        let mut input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        let mut input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];
        let mut o_l = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o_r = vec![0f64; CROSS_FADE_SAMPLES * 2];

        l.process_input(time as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
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
                Part::A,
            );
            process_until_done(&mut l);

            let mut o_l = vec![0f64; 32];
            let mut o_r = vec![0f64; 32];
            l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
            process_until_done(&mut l);

            time += 32;
        }

        let mut o_l = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o_r = vec![0f64; CROSS_FADE_SAMPLES * 2];

        l.process_input(time as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);

        verify_length(&l, CROSS_FADE_SAMPLES as u64 * 2);

        for i in 0..o_l.len() {
            if i < CROSS_FADE_SAMPLES {
                assert!(
                    (0f64 - o_l[i]).abs() < 0.000001,
                    "left is {} at idx {}, expected 0",
                    o_l[i],
                    time + i as i64
                );
                assert!(
                    (0f64 - o_r[i]).abs() < 0.000001,
                    "right is {} at idx {}, expected 0",
                    o_r[i],
                    time + i as i64
                );
            } else {
                assert_eq!(1f64, o_l[i], "mismatch at {}", time + i as i64);
                assert_eq!(-1f64, o_r[i], "mismatch at {}", time + i as i64);
            }
        }
    }

    #[test]
    fn test_pre_xfade() {
        install_test_logger();

        let mut l = looper_for_test();

        let mut input_left = vec![17f32; CROSS_FADE_SAMPLES];
        let mut input_right = vec![-17f32; CROSS_FADE_SAMPLES];

        let mut time = 0i64;
        // process some random input
        for i in (0..CROSS_FADE_SAMPLES).step_by(32) {
            l.process_input(
                time as u64,
                &[&input_left[i..i + 32], &input_right[i..i + 32]],
                Part::A,
            );
            process_until_done(&mut l);

            let mut o_l = vec![0f64; 32];
            let mut o_r = vec![0f64; 32];
            l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
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
                Part::A,
            );
            process_until_done(&mut l);

            let mut o_l = vec![0f64; 32];
            let mut o_r = vec![0f64; 32];
            l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
            process_until_done(&mut l);

            time += 32;
        }

        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);

        input_left = vec![1f32; CROSS_FADE_SAMPLES * 2];
        input_right = vec![-1f32; CROSS_FADE_SAMPLES * 2];

        let mut o_l = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o_r = vec![0f64; CROSS_FADE_SAMPLES * 2];

        l.process_input(time as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);
        time += input_left.len() as i64;

        l.transition_to(LooperMode::Playing);
        process_until_done(&mut l);

        // Go around again (we don't have the crossfaded samples until the second time around)
        l.process_input(time as u64, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);
        time += input_left.len() as i64;

        let mut o_l = vec![0f64; CROSS_FADE_SAMPLES * 2];
        let mut o_r = vec![0f64; CROSS_FADE_SAMPLES * 2];
        l.process_output(FrameTime(time), &mut [&mut o_l, &mut o_r], Part::A, false);
        process_until_done(&mut l);

        for i in 0..o_l.len() {
            if i > CROSS_FADE_SAMPLES {
                assert!(
                    (0f64 - o_l[i]).abs() < 0.000001,
                    "left is {} at idx {}, expected 0",
                    o_l[i],
                    i
                );
                assert!(
                    (0f64 - o_r[i]).abs() < 0.000001,
                    "right is {} at idx {}, expected 0",
                    o_r[i],
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

        let mut l = Looper::new(5, PartSet::new(), GuiSender::disconnected());

        l.transition_to(LooperMode::Recording);
        process_until_done(&mut l);
        l.process_input(0, &[&input_left, &input_right], Part::A);
        process_until_done(&mut l);

        l.transition_to(LooperMode::Overdubbing);
        process_until_done(&mut l);
        l.process_input(0, &[&input_left2, &input_right2], Part::A);
        process_until_done(&mut l);

        let (tx, rx) = bounded(1);
        l.channel()
            .send(ControlMessage::Serialize(dir.path().to_path_buf(), tx))
            .unwrap();
        process_until_done(&mut l);

        let state = rx.recv().unwrap().unwrap();

        let deserialized =
            Looper::from_serialized(&state, dir.path(), GuiSender::disconnected()).unwrap();

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

const CROSS_FADE_SAMPLES: usize = 8192;

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
                (vec![Recording], vec![], LooperBackend::finish_recording),
                (
                    vec![Recording, Overdubbing],
                    vec![],
                    LooperBackend::handle_crossfades,
                ),
                (
                    vec![],
                    vec![Overdubbing],
                    LooperBackend::prepare_for_overdubbing,
                ),
                (
                    vec![],
                    vec![Recording],
                    LooperBackend::prepare_for_recording,
                ),
                //(vec![], vec![None], LooperBackend::stop),
            ],
        }
    }

    fn handle_transition(&self, looper: &mut LooperBackend, next_state: LooperMode) {
        let cur = looper.mode();
        for transition in &self.transitions {
            if (transition.0.is_empty() || transition.0.contains(&cur))
                && (transition.1.is_empty() || transition.1.contains(&next_state))
            {
                transition.2(looper, next_state);
            }
        }
        looper.mode.store(next_state, Ordering::Relaxed);
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
    Deleted,
    Clear,
    SetSpeed(LooperSpeed),
    SetPan(f32),
    SetLevel(f32),
    SetParts(PartSet),
    Undo,
    Redo,
    StopOutput,
}

const TRANSFER_BUF_SIZE: usize = 16;

#[derive(Clone, Copy)]
struct TransferBuf<DATA: Copy> {
    id: u64,
    time: FrameTime,
    size: usize,
    data: [[DATA; TRANSFER_BUF_SIZE]; 2],
}

impl<DATA: Copy> TransferBuf<DATA> {
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

fn compute_waveform(samples: &[Sample], downsample: usize) -> Waveform {
    let len = samples[0].length() as usize;
    let size = len / downsample + 1;
    let mut out = [Vec::with_capacity(size), Vec::with_capacity(size)];

    for c in 0..2 {
        for t in (0..len).step_by(downsample) {
            let mut p = 0f64;
            let end = downsample.min(len - t);
            for s in samples {
                for j in 0..end {
                    let i = t as usize + j;
                    p += s.buffer[c][i].abs() as f64;
                }
            }

            out[c].push((p as f64 / (samples.len() as f64 * end as f64)) as f32);
        }
    }

    out
}

struct WaveformGenerator {
    id: u32,
    start_time: FrameTime,
    acc: [f64; 2],
    size: usize,
}

impl WaveformGenerator {
    fn new(id: u32) -> Self {
        WaveformGenerator {
            id,
            start_time: FrameTime(0),
            acc: [0.0, 0.0],
            size: 0,
        }
    }

    fn add_buf(
        &mut self,
        mode: LooperMode,
        time: FrameTime,
        samples: &[&[f64]],
        looper_length: u64,
        sender: &mut GuiSender,
    ) {
        if !(self.start_time.0..self.start_time.0 + WAVEFORM_DOWNSAMPLE as i64).contains(&time.0) {
            // there are no samples in this buffer that we can add, so we'll just send on the
            // partial buffer
            debug!(
                "sending partial buffer to GUI because we got a newer one \
            (cur time = {}, buf time = {})",
                self.start_time.0, time.0
            );
            self.flush(mode, looper_length, sender);
            self.start_time = time;
        }

        for i in 0..samples[0].len() {
            if self.size < WAVEFORM_DOWNSAMPLE {
                for c in 0..2 {
                    self.acc[c] += samples[c][i].abs();
                }
                self.size += 1;
            } else {
                self.flush(mode, looper_length, sender);
                self.start_time = FrameTime(time.0 + i as i64);
            }
        }
    }

    fn flush(&mut self, mode: LooperMode, looper_length: u64, sender: &mut GuiSender) {
        let s = [
            (self.acc[0] / self.size as f64).min(1.0) as f32,
            (self.acc[1] / self.size as f64).min(1.0) as f32,
        ];
        match mode {
            LooperMode::Recording => {
                sender.send_update(AddNewSample(self.id, self.start_time, s, looper_length));
            }
            LooperMode::Overdubbing => {
                sender.send_update(AddOverdubSample(self.id, self.start_time, s));
            }
            _ => {}
        }

        self.acc = [0.0, 0.0];
        self.size = 0;
    }
}

enum LooperChange {
    PushSample,
    PopSample(Sample),
    Clear {
        samples: Vec<Sample>,
        in_time: FrameTime,
        out_time: FrameTime,
        offset: FrameTime,
    },
    UnClear,
}

impl Debug for LooperChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LooperChange::PushSample => write!(f, "PushSample"),
            LooperChange::PopSample(sample) => write!(f, "PopSample<{}>", sample.length()),
            LooperChange::Clear { samples, .. } => write!(f, "Clear<{}>", samples.len()),
            LooperChange::UnClear => write!(f, "UnClear"),
        }
    }
}

pub struct LooperBackend {
    pub id: u32,
    pub samples: Vec<Sample>,
    pub mode: Arc<Atomic<LooperMode>>,
    pub length: Arc<Atomic<u64>>,
    pub speed: LooperSpeed,
    pub pan: f32,
    pub level: f32,
    pub parts: PartSet,
    pub deleted: bool,

    offset: FrameTime,

    enable_crossfading: bool,

    out_time: FrameTime,
    in_time: FrameTime,

    input_buffer: Sample,
    input_buffer_idx: usize,
    xfade_samples_left: usize,
    xfade_sample_idx: usize,

    in_queue: Arc<ArrayQueue<TransferBuf<f32>>>,
    out_queue: Arc<ArrayQueue<TransferBuf<f64>>>,

    gui_sender: GuiSender,

    // Visible for benchmark
    pub channel: Receiver<ControlMessage>,

    waveform_generator: WaveformGenerator,

    undo_queue: VecDeque<LooperChange>,
    redo_queue: VecDeque<LooperChange>,

    should_output: bool,
    gui_needs_reset: bool,
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

    pub fn process_until_done(&mut self) {
        loop {
            let msg = self.channel.try_recv();
            match msg {
                Ok(msg) => self.handle_msg(msg),
                Err(_) => break,
            };
        }
    }

    fn current_state(&self) -> LooperState {
        LooperState {
            mode: self.mode(),
            speed: self.speed,
            pan: self.pan,
            level: self.level,
            parts: self.parts,
            offset: self.offset,
            has_undos: !self.undo_queue.is_empty(),
            has_redos: !self.redo_queue.is_empty(),
        }
    }

    pub fn mode(&self) -> LooperMode {
        return self.mode.load(Ordering::Relaxed);
    }

    fn handle_msg(&mut self, msg: ControlMessage) -> bool /* continue */ {
        debug!("[{}] got control message: {:?}", self.id, msg);
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
            ControlMessage::Clear => {
                self.transition_to(LooperMode::Playing);

                let mut samples = vec![];
                swap(&mut samples, &mut self.samples);

                let change = LooperChange::Clear {
                    samples,
                    in_time: self.in_time,
                    out_time: self.out_time,
                    offset: self.offset,
                };

                self.in_time = FrameTime(0);
                self.out_time = FrameTime(0);
                self.offset = FrameTime(0);
                self.xfade_samples_left = 0;
                self.length.store(0, Ordering::Relaxed);
                self.gui_sender
                    .send_update(GuiCommand::ClearLooper(self.id));

                self.add_change(change);
            }
            ControlMessage::SetTime(time) => {
                self.out_time = FrameTime(time.0.max(0));
                self.in_time = time;
                self.should_output = true;
            }
            ControlMessage::ReadOutput(time) => {
                self.out_time = FrameTime(self.out_time.0.max(time.0));
            }
            ControlMessage::Shutdown => {
                info!("Got shutdown message, stopping");
                return false;
            }
            ControlMessage::Deleted => {
                info!("Looper was deleted");
                self.gui_sender
                    .send_update(GuiCommand::RemoveLooper(self.id));
                return false;
            }
            ControlMessage::Serialize(path, channel) => {
                let result = self.serialize(&path);
                if let Err(e) = channel.try_send(result) {
                    warn!("failed to respond to serialize request: {:?}", e);
                }
            }
            ControlMessage::SetSpeed(speed) => {
                self.speed = speed;
                self.gui_needs_reset = true;
            }
            ControlMessage::SetPan(pan) => {
                self.pan = pan;
                self.gui_sender.send_update(GuiCommand::LooperStateChange(
                    self.id,
                    self.current_state(),
                ));
            }
            ControlMessage::SetLevel(level) => {
                self.level = level;
                self.gui_sender.send_update(GuiCommand::LooperStateChange(
                    self.id, self.current_state()
                ));
            }
            ControlMessage::SetParts(parts) => {
                self.parts = parts;
                self.gui_sender.send_update(GuiCommand::LooperStateChange(
                    self.id, self.current_state()
                ));
            },
            ControlMessage::Undo => {
                info!("Performing Undo on queue: {:?}", self.undo_queue);

                if let Some(change) = self.undo_queue.pop_back() {
                    if let Some(change) = self.undo_change(change) {
                        self.redo_queue.push_back(change);
                    }
                }
                self.gui_sender.send_update(GuiCommand::LooperStateChange(
                    self.id, self.current_state()
                ));
            }
            ControlMessage::Redo => {
                info!("Performing Redo on queue: {:?}", self.redo_queue);
                if let Some(change) = self.redo_queue.pop_back() {
                    if let Some(change) = self.undo_change(change) {
                        self.undo_queue.push_back(change);
                    }
                }
                self.gui_sender.send_update(GuiCommand::LooperStateChange(
                    self.id, self.current_state()
                ));
            }
            ControlMessage::StopOutput => {
                self.should_output = false;
            }
        }

        if self.should_output {
            self.fill_output();
            if self.gui_needs_reset {
                self.reset_gui();
                self.gui_needs_reset = false;
            }
        }
        true
    }

    #[inline]
    fn time_loop_idx(&self, t: FrameTime, adjust_for_speed: bool) -> usize {
        let t = (t - self.offset).0;

        if adjust_for_speed {
            match self.speed {
                LooperSpeed::Half => t / 2,
                LooperSpeed::One => t,
                LooperSpeed::Double => t * 2,
            }
        } else {
            t
        }.rem_euclid(self.length.load(Ordering::Relaxed) as i64) as usize
    }

    fn fill_output(&mut self) {
        let sample_len = self.length_in_samples(true) as usize;
        // don't fill the output if we're in record mode, because we don't know our length. the
        // timing won't be correct if we wrap around.
        if sample_len > 0 && self.mode() != LooperMode::Recording && self.out_time.0 >= 0 {
            // make sure we don't pass our input and don't spend too much time doing this
            let mut count = 0;
            let end = self.in_time.0 + sample_len as i64;
            while self.out_time.0 + 1 < end as i64
                && count < 32
                && self.out_queue.len() < self.out_queue.capacity() / 2
            {
                let mut buf = TransferBuf {
                    id: 0,
                    time: self.out_time,
                    size: ((end - self.out_time.0) as usize).min(TRANSFER_BUF_SIZE),
                    data: [[0f64; TRANSFER_BUF_SIZE]; 2],
                };

                for sample in &self.samples {
                    let b = &sample.buffer;
                    if b[0].is_empty() {
                        continue;
                    }

                    for i in 0..2 {
                        for t in 0..buf.size {
                            buf.data[i][t] +=
                                b[i][self.time_loop_idx(self.out_time + FrameTime(t as i64), true)] as f64;
                        }
                    }
                }

                if self.out_queue.push(buf).is_err() {
                    break;
                }

                debug!(
                    "[OUTPUT {}] t = {} [{}; {}] (in time = {})",
                    self.id, self.out_time.0, buf.data[0][0], buf.size, self.in_time.0
                );

                self.out_time.0 += buf.size as i64;
                count += 1;
            }
        }
    }

    fn finish_recording(&mut self, _: LooperMode) {
        // update our out time to the current input time so that we don't bother outputting a bunch
        // of wasted data
        self.out_time = self.in_time;

        self.add_change(LooperChange::UnClear);

        // send our final length to the gui
        self.gui_sender
            .send_update(GuiCommand::SetLoopLengthAndOffset(
                self.id,
                self.length_in_samples(false),
                self.offset,
            ));
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

                if self.enable_crossfading {
                    s.xfade(
                        CROSS_FADE_SAMPLES,
                        0,
                        write_start,
                        &[&left, &right],
                        XfadeDirection::IN,
                        sample::norm,
                    );
                }
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
        self.length.store(0, Ordering::Relaxed);
    }

    fn prepare_for_overdubbing(&mut self, _next_state: LooperMode) {
        let overdub_sample = Sample::with_size(self.length_in_samples(false) as usize);

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

        self.add_change(LooperChange::PushSample);
        self.samples.push(overdub_sample);
    }

    pub fn transition_to(&mut self, mode: LooperMode) {
        debug!("Transition {:?} to {:?}", self.mode, mode);

        if self.mode() == mode {
            // do nothing if we're not changing state
            return;
        }

        STATE_MACHINE.handle_transition(self, mode);

        self.gui_sender.send_update(GuiCommand::LooperStateChange(
            self.id,
            LooperState {
                mode,
                speed: self.speed,
                pan: self.pan,
                level: self.level,
                parts: self.parts,
                offset: self.offset,
                has_undos: !self.undo_queue.is_empty(),
                has_redos: !self.redo_queue.is_empty(),
            },
        ));
    }

    fn handle_input(&mut self, time_in_samples: u64, inputs: &[&[f32]]) {
        if self.mode() == LooperMode::Overdubbing {
            // in overdub mode, we add the new samples to our existing buffer
            let time_in_loop = self.time_loop_idx(FrameTime(time_in_samples as i64), false);

            let s = self
                .samples
                .last_mut()
                .expect("No samples for looper in overdub mode");

            s.overdub(time_in_loop as u64, inputs, self.speed);

            // TODO: this logic should probably be abstracted out into Sample so it can be reused
            //       between here and fill_output
            let mut wv = [vec![0f64; inputs[0].len()], vec![0f64; inputs[0].len()]];
            for c in 0..2 {
                for i in 0..inputs[0].len() {
                    for s in &self.samples {
                        wv[c][i] += s.buffer[c]
                            [self.time_loop_idx(FrameTime(time_in_samples as i64 + i as i64), true)]
                            as f64;
                    }
                }
            }
            self.waveform_generator.add_buf(
                self.mode(),
                FrameTime(time_in_samples as i64),
                &[&wv[0], &wv[1]],
                self.length_in_samples(true),
                &mut self.gui_sender,
            );
        } else if self.mode() == LooperMode::Recording {
            // in record mode, we extend the current buffer with the new samples

            // if these are the first samples, set the offset to the current time
            if self.length_in_samples(false) == 0 {
                self.offset = FrameTime(time_in_samples as i64);
            }

            let s = self
                .samples
                .last_mut()
                .expect("No samples for looper in record mode");
            s.record(inputs);

            self.length.store(s.length(), Ordering::Relaxed);

            // TODO: this allocation isn't really necessary
            let mut wv = [vec![0f64; inputs[0].len()], vec![0f64; inputs[0].len()]];
            for (c, vs) in inputs.iter().enumerate() {
                for (i, v) in vs.iter().enumerate() {
                    wv[c][i] = *v as f64;
                }
            }
            self.waveform_generator.add_buf(
                self.mode(),
                FrameTime(time_in_samples as i64),
                &[&wv[0], &wv[1]],
                self.length_in_samples(true),
                &mut self.gui_sender,
            );
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
                if self.enable_crossfading {
                    s.xfade(
                        CROSS_FADE_SAMPLES,
                        CROSS_FADE_SAMPLES as u64 - self.xfade_samples_left as u64,
                        (CROSS_FADE_SAMPLES - self.xfade_samples_left) as u64,
                        inputs,
                        XfadeDirection::OUT,
                        sample::norm,
                    );
                }
                self.xfade_samples_left =
                    (self.xfade_samples_left as i64 - inputs[0].len() as i64).max(0) as usize;
            } else {
                debug!("tried to cross fade but no samples... something is likely wrong");
            }
        }

        self.in_time = FrameTime(time_in_samples as i64 + inputs[0].len() as i64);
    }

    fn add_change(&mut self, change: LooperChange) {
        self.undo_queue.push_back(change);
        self.redo_queue.clear();
        self.gui_sender.send_update(GuiCommand::LooperStateChange(
            self.id, self.current_state()
        ));
    }

    fn reset_gui(&mut self) {
        if self.length_in_samples(false) > 0 {
            self.gui_sender.send_update(GuiCommand::UpdateLooperWithSamples(
                self.id,
                self.length_in_samples(true),
                Box::new(compute_waveform(&self.samples, WAVEFORM_DOWNSAMPLE)),
                self.current_state(),
            ));
        } else {
            self.gui_sender.send_update(GuiCommand::LooperStateChange(
                self.id, self.current_state()))
        }
    }

    fn undo_change(&mut self, change: LooperChange) -> Option<LooperChange> {
        match change {
            LooperChange::PushSample => {
                let sample = self.samples.pop()
                    .map(|s| LooperChange::PopSample(s));
                self.gui_needs_reset = true;
                sample
            }
            LooperChange::PopSample(buffer) => {
                self.samples.push(buffer);
                self.gui_needs_reset = true;
                Some(LooperChange::PushSample)
            }
            LooperChange::Clear { samples, in_time, out_time, offset } => {
                self.samples = samples;
                self.in_time = in_time;
                self.out_time = out_time;
                self.offset = offset;

                if !self.samples.is_empty() {
                    self.length.store(self.samples[0].length(), Ordering::Relaxed);
                }

                self.gui_needs_reset = true;

                Some(LooperChange::UnClear)
            }
            LooperChange::UnClear => {
                let mut samples = vec![];
                swap(&mut samples, &mut self.samples);
                let change = Some(LooperChange::Clear {
                    samples,
                    in_time: self.in_time,
                    out_time: self.out_time,
                    offset: self.offset,
                });
                self.in_time = FrameTime(0);
                self.out_time = FrameTime(0);
                self.offset = FrameTime(0);
                self.gui_sender
                    .send_update(GuiCommand::ClearLooper(self.id));

                change
            }
        }
    }

    pub fn length_in_samples(&self, adjust_for_speed: bool) -> u64 {
        let len = self.length.load(Ordering::Relaxed);
        if adjust_for_speed {
            match self.speed {
                LooperSpeed::Half => len * 2,
                LooperSpeed::One => len,
                LooperSpeed::Double => len / 2,
            }
        } else {
            len
        }
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
            mode: self.mode(),
            parts: self.parts,
            speed: self.speed,
            pan: self.pan,
            level: self.level,
            samples: Vec::with_capacity(self.samples.len()),
            offset_samples: self.offset.0,
        };

        for (i, s) in self.samples.iter().enumerate() {
            let name = format!("loop_{}_{}.wav", self.id, i);
            let p = path.join(&name);
            let mut writer = hound::WavWriter::create(&p, spec.clone())?;

            for j in 0..s.length() as usize {
                writer.write_sample(s.buffer[0][j])?;
                writer.write_sample(s.buffer[1][j])?;
            }
            writer.finalize()?;
            // use the relative path so that the directory can be moved and still be valid
            saved.samples.push(PathBuf::from(name));
        }

        Ok(saved)
    }
}

// The Looper struct encapsulates behavior similar to a single hardware looper. Internally, it is
// driven by a state machine, which controls how it responds to input buffers (e.g., by recording
// or overdubbing to its internal buffers) and output buffers (e.g., by playing).
pub struct Looper {
    pub id: u32,
    pub deleted: bool,
    pub parts: PartSet,
    pub pan: f32,
    pub level: f32,

    pub pan_law: PanLaw,


    // this is pretty hacky -- we sometimes need a way to see the mode that has been just set on the
    // looper, before it's had a chance to make it to the backend
    local_mode: Option<LooperMode>,
    mode: Arc<Atomic<LooperMode>>,
    length: Arc<Atomic<u64>>,
    pub backend: Option<LooperBackend>,
    msg_counter: u64,
    out_queue: Arc<ArrayQueue<TransferBuf<f32>>>,
    in_queue: Arc<ArrayQueue<TransferBuf<f64>>>,
    channel: Sender<ControlMessage>,

    in_progress_output: Option<TransferBuf<f64>>,

    last_time: FrameTime,
}

impl Looper {
    pub fn new(id: u32, parts: PartSet, gui_output: GuiSender) -> Looper {
        Self::new_with_samples(
            id,
            parts,
            LooperSpeed::One,
            0.0,
            1.0,
            FrameTime(0),
            vec![],
            gui_output,
        )
    }

    fn new_with_samples(
        id: u32,
        parts: PartSet,
        speed: LooperSpeed,
        pan: f32,
        level: f32,
        offset: FrameTime,
        samples: Vec<Sample>,
        mut gui_sender: GuiSender,
    ) -> Looper {
        debug!("Creating new looper with samples {}", id);
        let record_queue = Arc::new(ArrayQueue::new(512 * 1024 / TRANSFER_BUF_SIZE));
        let play_queue = Arc::new(ArrayQueue::new(512 * 1024 / TRANSFER_BUF_SIZE));

        let (s, r) = bounded(1000);

        let length = samples.get(0).map(|s| s.length()).unwrap_or(0);

        let state = LooperState {
            mode: LooperMode::Playing,
            speed,
            pan,
            level,
            parts,
            offset,
            has_undos: false,
            has_redos: false,
        };

        if samples.is_empty() {
            gui_sender.send_update(GuiCommand::AddLooper(id, state));
        } else {
            gui_sender.send_update(GuiCommand::AddLooperWithSamples(
                id,
                length,
                Box::new(compute_waveform(&samples, WAVEFORM_DOWNSAMPLE)),
                state,
            ));
        }

        let mode = Arc::new(Atomic::new(LooperMode::Playing));
        let length = Arc::new(Atomic::new(samples.get(0)
            .map(|s| s.length())
            .unwrap_or(0)));

        let backend = LooperBackend {
            id,
            samples,
            mode: mode.clone(),
            length: length.clone(),
            speed,
            pan,
            level,
            parts,
            deleted: false,
            offset,
            enable_crossfading: true,
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
            gui_sender,
            channel: r,
            waveform_generator: WaveformGenerator::new(id),
            undo_queue: VecDeque::new(),
            redo_queue: VecDeque::new(),
            should_output: true,
            gui_needs_reset: false,
        };

        Looper {
            id,
            backend: Some(backend),
            parts,
            pan,
            level,
            pan_law: PanLaw::Neg4_5,
            deleted: false,
            msg_counter: 0,
            in_queue: play_queue.clone(),
            out_queue: record_queue.clone(),
            channel: s,
            mode,
            length,

            in_progress_output: None,

            last_time: FrameTime(0),
            local_mode: None
        }
    }

    pub fn from_serialized(
        state: &SavedLooper,
        path: &Path,
        gui_output: GuiSender,
    ) -> Result<Looper, SaveLoadError> {
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

        Ok(Self::new_with_samples(
            state.id,
            state.parts,
            state.speed,
            state.pan,
            state.level,
            FrameTime(state.offset_samples),
            samples,
            gui_output,
        ))
    }

    pub fn channel(&self) -> Sender<ControlMessage> {
        self.channel.clone()
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

    fn send_to_backend(&mut self, message: ControlMessage) -> bool {
        match self.channel.try_send(message) {
            Ok(_) => true,
            Err(TrySendError::Full(msg)) => {
                error!(
                    "Failed to process message {:?} in looper {}: channel is full",
                    msg, self.id
                );
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("Backend channel disconnected in looper {}", self.id);
                false
            }
        }
    }

    pub fn local_mode(&self) -> LooperMode {
        return self.local_mode.unwrap_or(self.mode());
    }

    pub fn mode(&self) -> LooperMode {
        self.mode.load(Ordering::Relaxed)
    }

    pub fn length(&self) -> u64 {
        self.length.load(Ordering::Relaxed)
    }

    pub fn set_time(&mut self, time: FrameTime) {
        loop {
            if self.in_queue.pop().is_none() {
                break;
            }
        }
        self.in_progress_output = None;

        if self.mode() == LooperMode::Recording && time < FrameTime(0) {
            // we will clear our buffer
            self.send_to_backend(ControlMessage::Clear);
        }

        self.send_to_backend(ControlMessage::SetTime(time));
    }

    fn clear_queue(&mut self) {
        self.set_time(self.last_time)
    }

    pub fn handle_command(&mut self, command: LooperCommand) {
        use LooperCommand::*;
        match command {
            Record => self.transition_to(LooperMode::Recording),
            Overdub => self.transition_to(LooperMode::Overdubbing),
            Play => self.transition_to(LooperMode::Playing),
            Mute => self.transition_to(LooperMode::Muted),
            Solo => self.transition_to(LooperMode::Soloed),
            Clear => {
                self.send_to_backend(ControlMessage::Clear);
                self.clear_queue();
            }

            SetSpeed(speed) => {
                self.send_to_backend(ControlMessage::StopOutput);
                self.send_to_backend(ControlMessage::SetSpeed(speed));
                self.clear_queue();
            }

            SetPan(pan) => {
                self.pan = pan;
                self.send_to_backend(ControlMessage::SetPan(pan));
            }

            SetLevel(level) => {
                self.level = level;
                self.send_to_backend(ControlMessage::SetLevel(level));
            }

            AddToPart(part) => {
                self.parts[part] = true;
                self.send_to_backend(ControlMessage::SetParts(self.parts));
            }
            RemoveFromPart(part) => {
                self.parts[part] = false;
                if self.parts.is_empty() {
                    // don't allow the user to clear all parts
                    self.parts[part] = true;
                } else {
                    self.send_to_backend(ControlMessage::SetParts(self.parts));
                }
            }
            Delete => {
                self.deleted = true;
                self.send_to_backend(ControlMessage::Deleted);
            }
            RecordOverdubPlay => {
                // TODO: this logic is duplicated in the gui, would be good to unify somehow
                if self.length() == 0 {
                    self.transition_to(LooperMode::Recording);
                } else if self.mode() == LooperMode::Recording || self.mode() == LooperMode::Playing {
                    self.transition_to(LooperMode::Overdubbing);
                } else {
                    self.transition_to(LooperMode::Playing);
                }
            }
            Undo => {
                self.send_to_backend(ControlMessage::StopOutput);
                self.send_to_backend(ControlMessage::Undo);
                self.clear_queue();
            }
            Redo => {
                self.send_to_backend(ControlMessage::StopOutput);
                self.send_to_backend(ControlMessage::Redo);
                self.clear_queue();
            }
        }
    }

    fn output_for_t(&mut self, t: FrameTime) -> Option<(f64, f64)> {
        let mut cur = self
            .in_progress_output
            .or_else(|| self.in_queue.pop())?;
        self.in_progress_output = Some(cur);

        loop {
            if cur.time.0 > t.0 {
                error!(
                    "data is in future for looper id {} (time is {}, needed {})",
                    self.id, cur.time.0, t.0
                );
                self.clear_queue();
                return None;
            }

            if let Some(o) = cur.get_t(t) {
                return Some(o);
            }

            if let Some(buf) = self.in_queue.pop() {
                cur = buf;
                self.in_progress_output = Some(buf);
            } else {
                self.in_progress_output = None;
                return None;
            }
        }
    }

    fn should_output(&self, part: Part, solo: bool) -> bool {
        if !self.parts[part] {
            return false
        }

        if solo && self.mode() != LooperMode::Soloed {
            return false
        }

        return self.mode() == LooperMode::Playing ||
            self.mode() == LooperMode::Overdubbing ||
            self.mode() == LooperMode::Soloed
    }

    // In process_output, we modify the specified output buffers according to our internal state. In
    // Playing or Overdub mode, we will add our buffer to the output. Otherwise, we do nothing.
    //
    // If the solo flag is set, we will only output if we are in solo mode.
    pub fn process_output(
        &mut self,
        time: FrameTime,
        outputs: &mut [&mut [f64]],
        part: Part,
        solo: bool,
    ) {
        if time.0 < 0 || self.length() == 0 {
            return;
        }

        debug!("reading time {}", time.0);

        let mut time = time;
        let mut out_idx = 0;

        let mut missing = 0;
        let mut waiting = 1_000;
        let backoff = crossbeam_utils::Backoff::new();

        // this only really needs to be updated when the pan changes, so we don't need to do this
        // for every buffer
        let pan_l = self.pan_law.left(self.pan);
        let pan_r = self.pan_law.right(self.pan);

        while out_idx < outputs[0].len() {
            if let Some((l, r)) = self.output_for_t(time) {
                if self.should_output(part, solo) {
                    outputs[0][out_idx] += l * pan_l as f64 * self.level as f64;
                    outputs[1][out_idx] += r * pan_r as f64 * self.level as f64;
                }
            } else if waiting > 0 && self.mode() != LooperMode::Recording {
                backoff.spin();
                waiting -= 1;
                continue;
            } else {
                missing += 1;
            }
            out_idx += 1;
            time.0 += 1;
        }

        self.last_time = time;

        if self.mode() != LooperMode::Recording && missing > 0 {
            error!(
                "needed output but queue was empty in looper {} at {} (missed {} samples)",
                self.id, time.0, missing
            );
        }

        match self.channel.try_send(ControlMessage::ReadOutput(time)) {
            Err(TrySendError::Disconnected(_)) => panic!("channel closed"),
            Err(TrySendError::Full(_)) => warn!("channel full while requesting more output"),
            _ => {}
        }

        self.local_mode = None;
    }

    // In process_input, we modify our internal buffers based on the input. In Record mode, we
    // append the data in the input buffers to our current sample. In Overdub mode, we sum the data
    // with whatever is currently in our buffer at the point of time_in_samples.
    pub fn process_input(&mut self, time_in_samples: u64, inputs: &[&[f32]], part: Part) {
        assert_eq!(2, inputs.len());

        debug!("inputting time {}", time_in_samples);

        let msg_id = self.msg_counter;
        self.msg_counter += 1;

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

            if self.parts[part] {
                // if this is not the current part, send 0s
                for i in 0..l.len() {
                    buf.data[0][i] = l[i];
                    buf.data[1][i] = r[i];
                }
            }

            if let Err(_) = self.out_queue.push(buf) {
                // TODO: handle error case where our queue is full
                error!("queue is full on looper {}", self.id);
            }

            time += l.len() as u64;
        }

        self.send_to_backend(ControlMessage::InputDataReady {
            id: msg_id,
            size: inputs[0].len(),
        });
    }

    pub fn transition_to(&mut self, mode: LooperMode) {
        let mut mode = mode;
        if self.length() == 0 && mode == LooperMode::Overdubbing {
            warn!("trying to move to overdub with 0-length looper");
            mode = LooperMode::Recording;
        }

        self.send_to_backend(ControlMessage::TransitionTo(mode));
        self.local_mode = Some(mode);
    }
}

impl Drop for Looper {
    fn drop(&mut self) {
        if let Err(_) = self.channel.send(ControlMessage::Shutdown) {
            warn!("failed to shutdown backend because queue was full");
        }
    }
}
