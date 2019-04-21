use jack::{AudioIn, Port, AudioOut, MidiIn};
use crossbeam_queue::SegQueue;
use crate::{Message, Command, PlayMode, RecordMode};
use std::sync::Arc;
use std::f32::NEG_INFINITY;
use rand::Rng;

struct Looper {
    uuid: u128,
    buf: Vec<[Vec<f32>; 2]>,
    play_mode: PlayMode,
    record_mode: RecordMode,
    message_sink: Arc<SegQueue<Message>>
}

impl Looper {
    fn new(message_sink: Arc<SegQueue<Message>>) -> Looper {
        Looper {
            uuid: rand::thread_rng().gen(),
            buf: vec![],
            play_mode: PlayMode::PAUSED,
            record_mode: RecordMode::NONE,
            message_sink,
        }
    }

    fn set_record_mode(&mut self, mode: RecordMode) {
        self.record_mode = mode;
        self.message_sink.push(Message::RecordingStateChanged(mode, self.uuid))
    }

    fn set_play_mode(&mut self, mode: PlayMode) {
        self.play_mode = mode;
        self.message_sink.push(Message::PlayingStateChanged(mode, self.uuid))
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
    selected: usize,
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
            gui_output: gui_output.clone(),
            gui_input,
            time: 0,
            loopers: vec![Looper::new(gui_output)],
            selected: 0,
        }
    }

    fn message(&mut self, message: Message) {
        self.gui_output.push(message);
    }

    fn update_states(&mut self, ps: &jack::ProcessScope) {
        let midi_in = &self.midi_in;
        let time = &mut self.time;
        let looper = &mut self.loopers[self.selected];
        let gui_output = &mut self.gui_output;

        midi_in.iter(ps).for_each(|e| {
            if e.bytes.len() == 3 && e.bytes[0] == 144 {
                match e.bytes[1]{
                    60 => {
                        if looper.buf.is_empty() || looper.play_mode == PlayMode::PAUSED {
                            looper.set_record_mode(RecordMode::READY);
                        } else {
                            looper.set_record_mode(RecordMode::OVERDUB);
                        }
                    }
                    62 => {
                        looper.set_record_mode(RecordMode::NONE);
                        looper.set_play_mode(PlayMode::PLAYING);

                        *time = 0;
                        gui_output.push(Message::TimeChanged(*time as i64));
                    },
                    64 => {
                        looper.set_play_mode(PlayMode::PAUSED);
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn process(&mut self, _ : &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        self.update_states(ps);

        let out_a_p = self.out_a.as_mut_slice(ps);
        let out_b_p = self.out_b.as_mut_slice(ps);
        let in_a_p = self.in_a.as_slice(ps);
        let in_b_p = self.in_b.as_slice(ps);

        let looper = &mut self.loopers[0];

        let gui_output = &mut self.gui_output;
        let time = &mut self.time;

        if looper.record_mode == RecordMode::READY && (max_abs(in_a_p) > THRESHOLD || max_abs(in_b_p) > THRESHOLD) {
            looper.buf.clear();
            looper.set_record_mode(RecordMode::RECORD);
        }

        let mut l = in_a_p.to_vec();
        let mut r = in_b_p.to_vec();

        let times = ps.cycle_times().unwrap();
        let frame_time = times.next_usecs - times.current_usecs;

        if looper.play_mode == PlayMode::PLAYING {
            if !looper.buf.is_empty() {
                let len = looper.buf.len();
                let el = &mut looper.buf[self.time % len];
                for i in 0..el[0].len() {
                    l[i] += el[0][i];
                    r[i] += el[1][i];

                    if looper.record_mode == RecordMode::OVERDUB {
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

        if looper.record_mode == RecordMode::RECORD {
            looper.buf.push([l, r]);
            self.gui_output.push(Message::TimeChanged(looper.buf.len() as i64 * frame_time as i64));
            self.gui_output.push(Message::LengthChanged(looper.buf.len() as i64 * frame_time as i64));
        }

        jack::Control::Continue
    }
}
