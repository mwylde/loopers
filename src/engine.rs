use jack::{AudioIn, Port, AudioOut, MidiIn};
use crossbeam_queue::SegQueue;
use std::sync::{Arc, Mutex};
use std::f32::NEG_INFINITY;
use crate::protos::*;
use crate::protos::command::CommandOneof;
use crate::sample::Sample;
use std::f64::consts::PI as PI64;
use crate::protos::looper_command::TargetOneof;
use std::ops::Rem;

const SAMPLE_RATE: f64 = 44.100;

struct Looper {
    id: u32,
    samples: Vec<Sample>,
    mode: LooperMode,
    deleted: bool,
}

impl Looper {
    fn new(id: u32) -> Looper {
        let looper = Looper {
            id,
            samples: vec![],
            mode: LooperMode::None,
            deleted: false,
        };

        looper
    }
}

impl Looper {
    fn transition_to(&mut self, mode: LooperMode) {
        println!("Transition {:?} to {:?}", self.mode, mode);
        match (self.mode, mode) {
            (x, y) if x == y => {},
            (_, LooperMode::None) => {},
            (_, LooperMode::Ready) => {},
            (_, LooperMode::Record) => {
                self.samples.clear();
                self.samples.push(Sample::new(0));
            },
            (_, LooperMode::Overdub) => {
                let len = self.length_in_samples();
                self.samples.push(Sample::new(self.samples.last().unwrap().length() as usize));
                println!("len: {}\t{}", len, self.length_in_samples());
            },
            (_, LooperMode::Playing) => {},
            (_, LooperMode::Stopping) => {},
        }

        self.mode = mode;
    }

    fn process_event(&mut self, looper_event: &LooperCommand) {
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
                    if self.mode == LooperMode::Record {
                        self.transition_to(LooperMode::Stopping);
                    } else {
                        self.transition_to(LooperMode::Playing);
                    }
                },
                LooperCommandType::Select => {
                    // TODO: handle
                },
                LooperCommandType::Delete => {
                    self.deleted = true;
                },

                LooperCommandType::ReadyOverdubPlay => {
                    if self.samples.is_empty() {
                        self.transition_to(LooperMode::Ready);
                    } else if self.mode == LooperMode::Record || self.mode == LooperMode::Playing {
                        self.transition_to(LooperMode::Overdub);
                    } else {
                        self.transition_to(LooperMode::Playing);
                    }
                }
            }
        } else {
            // TODO: log this
        }        
    }

    fn length_in_samples(&self) -> u64 {
        self.samples.get(0).map(|s| s.length()).unwrap_or(0)
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
    config: Config,

    in_a: Port<AudioIn>,
    in_b: Port<AudioIn>,
    out_a: Port<AudioOut>,
    out_b: Port<AudioOut>,

    met_out_a: Port<AudioOut>,
    met_out_b: Port<AudioOut>,

    midi_in: Port<MidiIn>,

    time: i64,
    time_signature: TimeSignature,
    tempo: Tempo,

    gui_output: Arc<SegQueue<State>>,
    gui_input: Arc<SegQueue<Command>>,

    loopers: Vec<Looper>,
    active: u32,

    beat_normal: Vec<f32>,
    beat_emphasis: Vec<f32>,
    metronome_time: usize,
    id_counter: u32,

    is_learning: bool,
    last_midi: Option<Vec<u8>>,
}

const THRESHOLD: f32 = 0.05;

fn max_abs(b: &[f32]) -> f32 {
    b.iter().map(|v| v.abs())
        .fold(NEG_INFINITY, |a, b| a.max(b))
}

impl Engine {
    pub fn new(config: Config,
               in_a: Port<AudioIn>, in_b: Port<AudioIn>,
               out_a: Port<AudioOut>, out_b: Port<AudioOut>,
               met_out_a: Port<AudioOut>, met_out_b: Port<AudioOut>,
               midi_in: Port<MidiIn>,
               gui_output: Arc<SegQueue<State>>,
               gui_input: Arc<SegQueue<Command>>,
               beat_normal: Vec<f32>,
               beat_emphasis: Vec<f32>) -> Engine {
        Engine {
            config,

            in_a,
            in_b,
            out_a,
            out_b,
            met_out_a,
            met_out_b,
            midi_in,

            time: 0,
            time_signature: TimeSignature::new(4, 4).unwrap(),
            tempo: Tempo::from_bpm(120.0),

            gui_output,
            gui_input,
            loopers: vec![Looper::new(0)],
            active: 0,
            id_counter: 1,
            metronome_time: beat_emphasis.len(),
            beat_normal,
            beat_emphasis,

            is_learning: false,
            last_midi: None,
        }
    }

    fn looper_by_id_mut(&mut self, id: u32) -> Option<&mut Looper> {
        self.loopers.iter_mut().find(|l| l.id == id)
    }

    fn looper_by_id(&self, id: u32) -> Option<&Looper> {
        self.loopers.iter().find(|l| l.id == id)
    }


    fn commands_from_midi(&self, ps: &jack::ProcessScope) {
        let midi_in = &self.midi_in;

        for e in midi_in.iter(ps) {
            println!("midi {:?}", e);

            for m in &self.config.midi_mappings {
                if e.bytes.get(1).map(|b| *b as u32 == m.controller_number).unwrap_or(false) &&
                    e.bytes.get(2).map(|b| *b as u32 == m.data).unwrap_or(false) {
                    if let Some(c) = &m.command {
                        self.gui_input.push(c.clone());
                    }
                }
            }
        }
    }

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

            match &c.command_oneof.unwrap() {
                CommandOneof::LooperCommand(lc) => {
                    match lc.target_oneof.as_ref().unwrap() {
                        TargetOneof::TargetAll(_) => {
                            for looper in &mut self.loopers {
                                looper.process_event(lc);
                            }
                        }
                        TargetOneof::TargetSelected(_) => {
                            if let Some(looper) = self.looper_by_id_mut(self.active) {
                                looper.process_event(lc);
                            } else {
                                // TODO: log this
                            }
                        }
                        TargetOneof::TargetNumber(t) => {
                            if let Some(looper) = self.loopers.get_mut(t.looper_number as usize) {
                                if lc.command_type == LooperCommandType::Select as i32 {
                                    self.active = looper.id;
                                } else {
                                    looper.process_event(lc);
                                }
                            } else {
                                // TODO: log this
                            }
                        }
                    }
                },
                CommandOneof::GlobalCommand(gc) => {
                    if let Some(typ) = GlobalCommandType::from_i32(gc.command) {
                        match typ as GlobalCommandType {
                            GlobalCommandType::ResetTime => {
                                self.time = 0;
                            },
                            GlobalCommandType::AddLooper => {
                                self.loopers.push(Looper::new(self.id_counter));
                                self.active = self.id_counter;
                                self.id_counter += 1;
                            }
                            GlobalCommandType::EnableLearnMode => {
                                self.is_learning = true;
                            }
                            GlobalCommandType::DisableLearnMode => {
                                self.is_learning = false;
                            }
                        }
                    }
                }
            }
        }
    }

    fn samples_to_time(samples: isize) -> i64 {
        ((samples as f64) / SAMPLE_RATE) as i64
    }

    fn time_to_samples(time_ms: f64) -> u64 {
        (time_ms * SAMPLE_RATE) as u64
    }

    fn play_sample(sample: &[f32], sample_time: usize, left: &mut [f32], right: &mut [f32]) {
        for i in 0..left.len() {
            let t = sample_time + i;
            if t >= sample.len() {
                return;
            }
            left[i] += sample[t] / 2f32;
            right[i] += sample[t] / 2f32;
        }
    }

    fn get_beat(&self) -> i64 {
        let bps = self.tempo.bpm() as f32 / 60.0;
        let mspb = 1000.0 / bps;
        (Engine::samples_to_time(self.time as isize) as f32 / mspb) as i64
    }

    fn measure_len(&self) -> u64 {
        let bps = self.tempo.bpm() as f32 / 60.0;
        let mspb = 1000.0 / bps;

        let mspm = mspb * self.time_signature.upper as f32;

        Engine::time_to_samples(mspm as f64)
    }

    pub fn process(&mut self, _ : &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        if !self.is_learning {
            self.commands_from_midi(ps);
            self.last_midi = None;
        } else {
            let midi_in = &self.midi_in;
            let new_last = midi_in.iter(ps).last().map(|m| m.bytes.to_vec());
            if new_last.is_some() {
                self.last_midi = new_last;
            }
        }

        let measure_len = self.measure_len();
        let prev_beat = self.get_beat().rem_euclid(self.time_signature.upper as i64);
        let prev_time = self.time;
        {
            if !self.loopers.iter().all(|l| l.mode == LooperMode::None) {
                self.time += ps.n_frames() as i64;
            } else {
                self.time = -(measure_len as i64);
            }
        }
        let cur_beat = self.get_beat().rem_euclid(self.time_signature.upper as i64);
        // self.commands_from_midi(ps);

        self.handle_commands();

        let out_a_p = self.out_a.as_mut_slice(ps);
        let out_b_p = self.out_b.as_mut_slice(ps);
        let met_a_p = self.met_out_a.as_mut_slice(ps);
        let met_b_p = self.met_out_b.as_mut_slice(ps);
        let in_a_p = self.in_a.as_slice(ps);
        let in_b_p = self.in_b.as_slice(ps);

        let buffer_len = in_a_p.len();

        let active = self.active;

        let mut l: Vec<f64> = in_a_p.iter().map(|v| *v as f64).collect();
        let mut r: Vec<f64> = in_b_p.iter().map(|v| *v as f64).collect();

        let mut met_l: Vec<f32> = out_a_p.to_vec();
        let mut met_r: Vec<f32> = out_b_p.to_vec();

        if self.time >= 0 {
            for looper in &mut self.loopers {
                if looper.deleted {
                    continue;
                }
                if looper.mode == LooperMode::Playing || looper.mode == LooperMode::Overdub {
                    let mut time = self.time as usize;
                    if !looper.samples.is_empty() {
                        for i in 0..buffer_len {
                            for sample in &looper.samples {
                                let b = &sample.buffer;
                                assert_eq!(b[0].len(), b[1].len());
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
        }

        if prev_beat != cur_beat || (prev_time == -(measure_len as i64) && self.time != prev_time) {
            self.metronome_time = 0;
        }

        let sample = if cur_beat == 0 {
            &self.beat_emphasis
        } else {
            &self.beat_normal
        };

//        let met_l = self.met_out_a.as_mut_slice(ps);
//        let met_r = self.met_out_b.as_mut_slice(ps);

        Engine::play_sample(sample, self.metronome_time, &mut met_l, &mut met_r);
        self.metronome_time += met_l.len();

        let looper = self.loopers.iter_mut().find(|l| l.id == active).unwrap();

        if prev_beat != cur_beat &&
            cur_beat == 0 &&
            looper.mode == LooperMode::Stopping{
            looper.transition_to(LooperMode::Playing);
        }

        if looper.mode == LooperMode::Ready &&
            //(max_abs(in_a_p) > THRESHOLD || max_abs(in_b_p) > THRESHOLD
            self.time >= 0 {
            looper.transition_to(LooperMode::Record);
        }

        let l32: Vec<f32> = l.into_iter().map(|f| f as f32).collect();
        let r32: Vec<f32> = r.into_iter().map(|f| f as f32).collect();
        out_a_p.clone_from_slice(&l32);
        out_b_p.clone_from_slice(&r32);

        met_a_p.clone_from_slice(&met_l);
        met_b_p.clone_from_slice(&met_r);

        // in overdub mode, we add the new samples to our existing buffer
        if looper.mode == LooperMode::Overdub && self.time >= 0 {
            let length = looper.length_in_samples();
            looper.samples.iter_mut().last().unwrap().record(
                measure_len, (self.time as u64) % length, &[in_a_p, in_b_p]);
        }

        // in record mode, we extend the current buffer with the new samples
        if (looper.mode == LooperMode::Record || looper.mode == LooperMode::Stopping) && self.time >= 0 {
            let len = looper.length_in_samples();
            looper.samples.iter_mut().last().unwrap().record(
                measure_len, len, &[in_a_p, &in_b_p])
        }

        // TODO: make this non-allocating
        let gui_output = &mut self.gui_output;
        let time = self.time as usize;
        let loop_states: Vec<LoopState> = self.loopers.iter()
            .filter(|l| !l.deleted)
            .map(|l| {
            let len = l.length_in_samples() as usize;

            let t = if len > 0 && (l.mode == LooperMode::Playing || l.mode == LooperMode::Overdub) {
                time % len
            } else {
                0
            };

            LoopState {
                id: l.id,
                mode: l.mode as i32,
                time: Engine::samples_to_time(t as isize) as i64,
                length: Engine::samples_to_time(len as isize) as i64,
                active: l.id == active,
            }
        }).collect();

        gui_output.push(State{
            loops: loop_states,
            time: Engine::samples_to_time(self.time as isize) as i64,
            length: 0,
            beat: cur_beat,
            bpm: self.tempo.bpm(),
            time_signature_upper: self.time_signature.upper as u64,
            time_signature_lower: self.time_signature.lower as u64,
            learn_mode: self.is_learning,
            last_midi: self.last_midi.as_ref().map(|b| b.clone()).unwrap_or_else(|| vec![]),
        });

        jack::Control::Continue
    }
}
