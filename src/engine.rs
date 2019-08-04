use jack::{AudioIn, Port, AudioOut, MidiIn};
use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::f32::NEG_INFINITY;
use crate::protos::*;
use crate::protos::command::CommandOneof;
use crate::sample::Sample;

const SAMPLE_RATE: f64 = 44.100;

struct Looper {
    id: u32,
    samples: Vec<Sample>,
    mode: LooperMode,
    time: usize,
    deleted: bool,
}

impl Looper {
    fn new(id: u32) -> Looper {
        let looper = Looper {
            id,
            samples: vec![],
            mode: LooperMode::None,
            time: 0,
            deleted: false,
        };

        looper
    }
}

impl Looper {
    fn transition_to(&mut self, mode: LooperMode) {
        match (self.mode, mode) {
            (x, y) if x == y => {},
            (_, LooperMode::None) => {},
            (_, LooperMode::Ready) => {},
            (_, LooperMode::Record) => {
                self.samples.clear();
                self.samples.push(Sample::new());
            },
            (_, LooperMode::Overdub) => {
                self.samples.push(Sample::new());
            },
            (_, LooperMode::Playing) => {},
        }

        self.mode = mode;
    }

    fn process_event(&mut self, engine: &mut Engine, looper_event: LooperCommand) {
        if let Some(typ) = LooperCommandType::from_i32(looper_event.command_type) {
            match typ as LooperCommandType {
                LooperCommandType::EnableReady => {
                    self.transition_to(LooperMode::Ready);
                }
                LooperCommandType::EnableRecord => {
                    self.transition_to(LooperMode::Record);
                },
                LooperCommandType::EnableOverdub => {
                    self.transition_to(LooperMode::Overdub);
                },
                LooperCommandType::EnableMutiply => {
                    // TODO
                },
                LooperCommandType::Stop => {
                    self.transition_to(LooperMode::None);
                }

                LooperCommandType::EnablePlay => {
                    self.transition_to(LooperMode::Playing);
                },
                LooperCommandType::Select => {
                    engine.active = self.id;
                },
                LooperCommandType::Delete => {
                    self.deleted = true;
                },
            }
        } else {
            // TODO: log this
        }        
    }
}

pub struct TimeSignature {
    upper: u8,
    lower: u8,
}

impl TimeSignature {
    pub fn new(upper: u8, lower: u8) -> Option<TimeSignature> {
        if lower == 0 || (lower & (lower - 1)) != 0 {
            // lower must be a power of 2
            return None
        }
        Some(TimeSignature { upper, lower })
    }
}

pub struct Tempo {
    mbpm: u64
}

impl Tempo {
    pub fn from_bpm(bpm: f32) -> Tempo {
        Tempo { mbpm: (bpm * 1000f32).round() as u64 }
    }

    pub fn bpm(&self) -> f32 {
        (self.mbpm as f32) / 1000f32
    }
}


pub struct Engine {
    in_a: Port<AudioIn>,
    in_b: Port<AudioIn>,
    out_a: Port<AudioOut>,
    out_b: Port<AudioOut>,

    midi_in: Port<MidiIn>,

    time: u64,
    time_signature: TimeSignature,
    tempo: Tempo,

    gui_output: Arc<SegQueue<State>>,
    gui_input: Arc<SegQueue<Command>>,

    loopers: Vec<Looper>,
    active: u32,

    id_counter: u32,
}

const THRESHOLD: f32 = 0.05;

fn max_abs(b: &[f32]) -> f32 {
    b.iter().map(|v| v.abs())
        .fold(NEG_INFINITY, |a, b| a.max(b))
}

impl Engine {
    pub fn new(in_a: Port<AudioIn>, in_b: Port<AudioIn>,
           out_a: Port<AudioOut>, out_b: Port<AudioOut>,
           midi_in: Port<MidiIn>,
           gui_output: Arc<SegQueue<State>>,
           gui_input: Arc<SegQueue<Command>>) -> Engine {
        Engine {
            in_a,
            in_b,
            out_a,
            out_b,
            midi_in,

            time: 0,
            time_signature: TimeSignature::new(4, 4).unwrap(),
            tempo: Tempo::from_bpm(120.0),

            gui_output,
            gui_input,
            loopers: vec![Looper::new(0)],
            active: 0,
            id_counter: 1,
        }
    }

    fn looper_by_id_mut(&mut self, id: u32) -> Option<&mut Looper> {
        self.loopers.iter_mut().find(|l| l.id == id)
    }

    fn looper_by_id(&self, id: u32) -> Option<&Looper> {
        self.loopers.iter().find(|l| l.id == id)
    }

//    fn commands_from_midi(&self, ps: &jack::ProcessScope) {
//        let midi_in = &self.midi_in;
//
//        let looper = self.looper_by_id(self.active).unwrap();
//
//        fn looper_command(id: u32, typ: LooperCommandType) -> Command {
//            Command {
//                command_oneof: Some(CommandOneof::LooperCommand(LooperCommand {
//                    loopers: vec![id],
//                    command_type: typ as i32,
//                }))
//            }
//        }
//
//        fn global_command(typ: GlobalCommandType) -> Command {
//            Command {
//                command_oneof: Some(CommandOneof::GlobalCommand(GlobalCommand {
//                    command: typ as i32,
//                }))
//            }
//        }
//
//        for e in midi_in.iter(ps) {
//            if e.bytes.len() == 3 && e.bytes[0] == 144 {
//                match e.bytes[1] {
//                    60 => {
//                        if looper.buffers.is_empty() || looper.play_mode == PlayMode::Paused {
//                            self.gui_input.push(looper_command(looper.id, EnableReady));
//                        } else {
//                            self.gui_input.push(looper_command(looper.id, EnableOverdub));
//                        }
//                    }
//                    62 => {
//                        self.gui_input.push(looper_command(looper.id, DisableRecord));
//
//                        if looper.play_mode == PlayMode::Paused {
//                            self.gui_input.push(looper_command(looper.id, EnablePlay));
//                        } else {
//                            self.gui_input.push(looper_command(looper.id, DisablePlay));
//                        }
//
//                        self.gui_input.push(global_command(ResetTime));
//                    },
//                    64 => {
//                        self.gui_input.push(global_command(AddLooper))
//                    }
//                    _ => {}
//                }
//            } else {}
//        }
//    }

    fn handle_commands(&mut self) {
        loop {
            let c = self.gui_input.pop();
            if c.is_err() {
                return;
            }
            let c = c.unwrap();
            if c.command_oneof.is_none() {
                continue;
            }

            match c.command_oneof.unwrap() {
                CommandOneof::LooperCommand(lc) => {
                    for looper_id in lc.loopers {
                        if let Some(looper) = self.looper_by_id_mut(looper_id) {
                            looper.process_event(self, lc);
                        } else {
                            // TODO: log this
                        }
                    }
                },
                CommandOneof::GlobalCommand(gc) => {
                    if let Some(typ) = GlobalCommandType::from_i32(gc.command) {
                        match typ as GlobalCommandType {
                            GlobalCommandType::ResetTime => {
                                for l in &mut self.loopers {
                                    l.time = 0;
                                }
                            },
                            GlobalCommandType::AddLooper => {
                                self.loopers.push(Looper::new(self.id_counter));
                                self.active = self.id_counter;
                                self.id_counter += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn samples_to_time(time: usize) -> u64 {
        ((time as f64) / SAMPLE_RATE) as u64
    }

    fn get_beat(&self) -> u64 {
        let bps = self.tempo.bpm() as f32 / 60.0;
        let mspb = 1000.0 / bps;
        (Engine::samples_to_time(self.time as usize) as f32 / mspb) as u64
    }

    pub fn process(&mut self, _ : &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let prev_beat = self.get_beat();
        {
            self.time += ps.n_frames() as u64;
        }
        let cur_beat = self.get_beat();

        // self.commands_from_midi(ps);

        self.handle_commands();

        let out_a_p = self.out_a.as_mut_slice(ps);
        let out_b_p = self.out_b.as_mut_slice(ps);
        let in_a_p = self.in_a.as_slice(ps);
        let in_b_p = self.in_b.as_slice(ps);

        let buffer_len = in_a_p.len();

        let active = self.active;

        let mut l: Vec<f64> = in_a_p.iter().map(|v| *v as f64).collect();
        let mut r: Vec<f64> = in_b_p.iter().map(|v| *v as f64).collect();


        for looper in &mut self.loopers {
            if looper.deleted {
                continue;
            }
            if looper.mode == LooperMode::Playing {
                let mut time = looper.time;
                if !looper.samples.is_empty() {
                    for i in 0..buffer_len {
                        for b in &looper.buffers {
                            assert_eq!(b[0].len(), b[1].len());
                            assert_eq!(b[0].len(), looper.buffers[0][0].len());
                            if b[0].len() > 0 {
                                l[i] += b[0][time % b[0].len()] as f64;
                                r[i] += b[1][time % b[1].len()] as f64;
                            }
                        }
                        time += 1;
                    }
                }
            }
        }

        let looper = self.loopers.iter_mut().find(|l| l.id == active).unwrap();

        if looper.mode == LooperMode::Ready && (max_abs(in_a_p) > THRESHOLD || max_abs(in_b_p) > THRESHOLD) {
            looper.buffers.clear();
            looper.buffers.push([vec![], vec![]]);
            looper.transition_to(LooperMode::Record);
        }

        let l32: Vec<f32> = l.into_iter().map(|f| f as f32).collect();
        let r32: Vec<f32> = r.into_iter().map(|f| f as f32).collect();
        out_a_p.clone_from_slice(&l32);
        out_b_p.clone_from_slice(&r32);

        // in overdub mode, we add the new samples to our existing buffer
        if looper.mode == LooperMode::Overdub {
            let len = looper.buffers[0][0].len();
            for i in 0..buffer_len {
                let time = looper.time + i;
                looper.buffers.last_mut().unwrap()[0][time % len] += in_a_p[i];
                looper.buffers.last_mut().unwrap()[1][time % len] += in_b_p[i];
            }
        }

        // in record mode, we extend the current buffer with the new samples
        if looper.mode == LooperMode::Record {
            looper.buffers[0][0].extend_from_slice(&in_a_p);
            looper.buffers[0][1].extend_from_slice(&in_b_p);
        }

        self.loopers.iter_mut().filter(|l|
            l.mode == LooperMode::Playing || l.mode == LooperMode::Overdub)
            .for_each(|looper| looper.time += buffer_len);

        // TODO: make this non-allocating
        let gui_output = &mut self.gui_output;
        let loop_states: Vec<LoopState> = self.loopers.iter()
            .filter(|l| !l.deleted)
            .enumerate().map(|(i, l)| {
            let time = l.time;
            let len = l.buffers.get(0).map(|l| l[0].len())
                .unwrap_or(0);

            let t = if len > 0 && l.mode == LooperMode::Playing || l.mode == LooperMode::Overdub {
                time % len
            } else {
                0
            };

            LoopState {
                id: l.id,
                mode: l.mode as i32,
                time: Engine::samples_to_time(t) as i64,
                length: Engine::samples_to_time(len) as i64,
                active: l.id == active,
            }
        }).collect();

        gui_output.push(State{
            loops: loop_states,
            time: Engine::samples_to_time(self.time as usize) as i64,
            length: 0,
            beat: cur_beat,
            bpm: self.tempo.bpm(),
            time_signature_upper: self.time_signature.upper as u64,
            time_signature_lower: self.time_signature.lower as u64,
        });

        jack::Control::Continue
    }
}
