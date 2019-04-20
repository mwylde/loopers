use jack::{AudioIn, Port, AudioOut, MidiIn};
use crossbeam_queue::SegQueue;
use crate::{Message, Command, PlayMode, RecordMode};
use std::sync::Arc;
use std::f32::NEG_INFINITY;

struct Looper {
    buf: Vec<[Vec<f32>; 2]>,
    play_mode: PlayMode,
    record_mode: RecordMode,
}

impl Looper {
    fn new() -> Looper {
        Looper {
            buf: vec![],
            play_mode: PlayMode::PAUSED,
            record_mode: RecordMode::NONE,
        }
    }
}

pub struct Engine {
    in_a: Port<AudioIn>,
    in_b: Port<AudioIn>,
    out_a: Port<AudioOut>,
    out_b: Port<AudioOut>,

    midi_in: Port<MidiIn>,

    gui_output: Arc<SegQueue<Message>>,
    gui_input: Arc<SegQueue<Command>>,

    time: usize,

    loopers: Vec<Looper>,
}

const THRESHOLD: f32 = 0.1;

fn max_abs(b: &[f32]) -> f32 {
    b.iter().map(|v| v.abs())
        .fold(NEG_INFINITY, |a, b| a.max(b))
}

impl Engine {
    pub fn new(in_a: Port<AudioIn>, in_b: Port<AudioIn>,
           out_a: Port<AudioOut>, out_b: Port<AudioOut>,
           midi_in: Port<MidiIn>,
           gui_output: Arc<SegQueue<Message>>,
           gui_input: Arc<SegQueue<Command>>) -> Engine {
        Engine {
            in_a,
            in_b,
            out_a,
            out_b,
            midi_in,
            gui_output,
            gui_input,
            time: 0,
            loopers: vec![Looper::new()]
        }
    }

    pub fn process(&mut self, _ : &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let out_a_p = self.out_a.as_mut_slice(ps);
        let out_b_p = self.out_b.as_mut_slice(ps);
        let in_a_p = self.in_a.as_slice(ps);
        let in_b_p = self.in_b.as_slice(ps);

        let looper = &mut self.loopers[0];
        let record_mode = &mut looper.record_mode;
        let play_mode = &mut looper.play_mode;

        let buffer = &mut looper.buf;

        let gui_output = &mut self.gui_output;
        let time = &mut self.time;

        self.midi_in.iter(ps).for_each(|e| {
            if e.bytes.len() == 3 && e.bytes[0] == 144 {
                match e.bytes[1]{
                    60 => {
                        if buffer.is_empty() || *play_mode == PlayMode::PAUSED {
                            *record_mode = RecordMode::READY;
                            gui_output.push(Message::RecordingStateChanged(*record_mode));
                        } else {
                            *record_mode = RecordMode::OVERDUB;
                            gui_output.push(Message::RecordingStateChanged(*record_mode));
                        }
                    }
                    62 => {
                        *record_mode = RecordMode::NONE;
                        gui_output.push(Message::RecordingStateChanged(*record_mode));
                        *play_mode = PlayMode::PLAYING;
                        gui_output.push(Message::PlayingStateChanged(*play_mode));

                        *time = 0;
                        gui_output.push(Message::TimeChanged(*time as i64));
                    },
                    64 => {
                        *play_mode = PlayMode::PAUSED;
                        gui_output.push(Message::PlayingStateChanged(*play_mode));
                    }
                    _ => {}
                }
            }
        });

        if *record_mode == RecordMode::READY && (max_abs(in_a_p) > THRESHOLD || max_abs(in_b_p) > THRESHOLD) {
            buffer.clear();
            *record_mode = RecordMode::RECORD;
            self.gui_output.push(Message::RecordingStateChanged(*record_mode));
        }

        let mut l = in_a_p.to_vec();
        let mut r = in_b_p.to_vec();

        let times = ps.cycle_times().unwrap();
        let frame_time = times.next_usecs - times.current_usecs;

        if *play_mode == PlayMode::PLAYING {
            if !buffer.is_empty() {
                let len = buffer.len();
                let el = &mut buffer[self.time % len];
                for i in 0..el[0].len() {
                    l[i] += el[0][i];
                    r[i] += el[1][i];

                    if *record_mode == RecordMode::OVERDUB {
                        el[0][i] += in_a_p[i];
                        el[1][i] += in_b_p[i];
                    }
                }

                self.time += 1;
                self.gui_output.push(Message::TimeChanged(((self.time % len) as i64) * frame_time as i64));
            }
        }

        out_a_p.clone_from_slice(&l);
        out_b_p.clone_from_slice(&r);

        if *record_mode == RecordMode::RECORD {
            buffer.push([l, r]);
            self.gui_output.push(Message::TimeChanged(buffer.len() as i64 * frame_time as i64));
            self.gui_output.push(Message::LengthChanged(buffer.len() as i64 * frame_time as i64));
        }

        jack::Control::Continue
    }
}
