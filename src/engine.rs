use crate::error::SaveLoadError;
use crate::looper::Looper;
use crate::metronome::Metronome;
use crate::midi::MidiEvent;
use crate::music::*;
use crate::protos::command::CommandOneof;
use crate::protos::looper_command::TargetOneof;
use crate::protos::*;
use crate::sample::Sample;
use bytes::BytesMut;
use chrono::Local;
use crossbeam_queue::SegQueue;
use prost::Message;
use std::f32::NEG_INFINITY;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Eq, PartialEq)]
enum TriggerCondition {
    BEAT0,
}

struct Trigger {
    condition: TriggerCondition,
    command: Command,
}

#[derive(Copy, Clone)]
pub struct MetricStructure {
    pub time_signature: TimeSignature,
    pub tempo: Tempo,
}

impl MetricStructure {
    pub fn new(upper: u8, lower: u8, bpm: f32) -> Option<MetricStructure> {
        let time_signature = TimeSignature::new(upper, lower)?;
        Some(MetricStructure {
            time_signature,
            tempo: Tempo::from_bpm(bpm),
        })
    }
}

pub struct Engine {
    config: Config,

    time: i64,

    metric_structure: MetricStructure,

    gui_output: Arc<SegQueue<State>>,
    gui_input: Arc<SegQueue<Command>>,

    loopers: Vec<Looper>,
    active: u32,

    metronome: Option<Metronome>,

    triggers: Vec<Trigger>,

    id_counter: u32,

    is_learning: bool,
    last_midi: Option<Vec<u8>>,
}

#[allow(dead_code)]
const THRESHOLD: f32 = 0.05;

#[allow(dead_code)]
fn max_abs(b: &[f32]) -> f32 {
    b.iter()
        .map(|v| v.abs())
        .fold(NEG_INFINITY, |a, b| a.max(b))
}

fn last_session_path() -> io::Result<PathBuf> {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("loopers");
    create_dir_all(&config_path)?;
    config_path.push(".last-session");
    Ok(config_path)
}

impl Engine {
    pub fn new(
        config: Config,
        gui_output: Arc<SegQueue<State>>,
        gui_input: Arc<SegQueue<Command>>,
        beat_normal: Vec<f32>,
        beat_emphasis: Vec<f32>,
        restore: bool,
    ) -> Engine {
        let metric_structure = MetricStructure::new(4, 4, 120.0).unwrap();
        let mut engine = Engine {
            config,

            time: 0,

            metric_structure,

            gui_output,
            gui_input,
            loopers: vec![Looper::new(0)],
            active: 0,
            id_counter: 1,

            metronome: Some(Metronome::new(
                metric_structure,
                Sample::from_mono(&beat_normal),
                Sample::from_mono(&beat_emphasis),
            )),

            triggers: vec![],

            is_learning: false,
            last_midi: None,
        };

        if restore {
            let mut restore_fn = || {
                let config_path = last_session_path()?;
                let restore_path = read_to_string(config_path)?;
                println!("Restoring from {}", restore_path);
                engine.load_session(&LoadSessionCommand { path: restore_path })
            };

            if let Err(err) = restore_fn() {
                println!("Failed to restore existing session {:?}", err);
            }
        }

        engine
    }

    fn looper_by_id_mut(&mut self, id: u32) -> Option<&mut Looper> {
        self.loopers.iter_mut().find(|l| l.id == id)
    }

    fn commands_from_midi(&self, events: &[MidiEvent]) {
        for e in events {
            println!("midi {:?}", e);

            for m in &self.config.midi_mappings {
                if e.bytes
                    .get(1)
                    .map(|b| *b as u32 == m.controller_number)
                    .unwrap_or(false)
                    && e.bytes.get(2).map(|b| *b as u32 == m.data).unwrap_or(false)
                {
                    if let Some(c) = &m.command {
                        self.gui_input.push(c.clone());
                    }
                }
            }
        }
    }

    // possibly convert a loop command into a trigger
    fn trigger_from_command(lc: &LooperCommand) -> Option<Trigger> {
        match LooperCommandType::from_i32(lc.command_type) {
            Some(LooperCommandType::EnableRecord)
            | Some(LooperCommandType::EnableOverdub)
            | Some(LooperCommandType::RecordOverdubPlay) => Some(Trigger {
                condition: TriggerCondition::BEAT0,
                command: Command {
                    command_oneof: Some(CommandOneof::LooperCommand(lc.clone())),
                },
            }),
            _ => None,
        }
    }

    fn handle_loop_command(&mut self, lc: &LooperCommand, triggered: bool) {
        println!("Handling loop command: {:?}", lc);

        if !triggered {
            if let Some(trigger) = Engine::trigger_from_command(lc) {
                self.triggers.push(trigger);
                return;
            }
        }

        let loopers: Vec<&mut Looper> = match lc.target_oneof.as_ref().unwrap() {
            TargetOneof::TargetAll(_) => self.loopers.iter_mut().collect(),
            TargetOneof::TargetSelected(_) => {
                if let Some(l) = self.looper_by_id_mut(self.active) {
                    vec![l]
                } else {
                    vec![]
                }
            }
            TargetOneof::TargetNumber(t) => {
                if let Some(l) = self.loopers.get_mut(t.looper_number as usize) {
                    vec![l]
                } else {
                    vec![]
                }
            }
        };

        let mut selected = None;

        // TODO: warn if loopers is empty (indicating an invalid selection)
        for l in loopers {
            if let Some(typ) = LooperCommandType::from_i32(lc.command_type) {
                match typ as LooperCommandType {
                    LooperCommandType::EnableReady => {
                        l.transition_to(LooperMode::Ready);
                    }
                    LooperCommandType::EnableRecord => {
                        l.transition_to(LooperMode::Record);
                    }
                    LooperCommandType::EnableOverdub => {
                        l.transition_to(LooperMode::Overdub);
                    }
                    LooperCommandType::EnableMutiply => {
                        // TODO
                    }
                    LooperCommandType::Stop => {
                        l.transition_to(LooperMode::None);
                    }

                    LooperCommandType::EnablePlay => {
                        l.transition_to(LooperMode::Playing);
                    }
                    LooperCommandType::Select => {
                        selected = Some(l.id);
                    }
                    LooperCommandType::Delete => {
                        l.deleted = true;
                    }

                    LooperCommandType::RecordOverdubPlay => {
                        selected = Some(l.id);
                        if l.length_in_samples() == 0 {
                            l.transition_to(LooperMode::Record);
                        } else if l.mode == LooperMode::Record || l.mode == LooperMode::Playing {
                            l.transition_to(LooperMode::Overdub);
                        } else {
                            l.transition_to(LooperMode::Playing);
                        }
                    }
                }
            } else {
                // TODO: log this
            }
        }

        if let Some(id) = selected {
            self.active = id;
        }
    }

    fn save_session(&self, command: &SaveSessionCommand) -> Result<(), SaveLoadError> {
        let now = Local::now();
        let mut path = PathBuf::from(&command.path);
        path.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());

        create_dir_all(&path)?;

        let mut session = SavedSession {
            save_time: now.timestamp_millis(),
            time_signature_upper: self.metric_structure.time_signature.upper as u64,
            time_signature_lower: self.metric_structure.time_signature.lower as u64,
            tempo_mbpm: self.metric_structure.tempo.mbpm,
            loopers: Vec::with_capacity(self.loopers.len()),
        };

        for l in &self.loopers {
            let state = l.serialize(&path)?;
            session.loopers.push(state);
        }

        path.push("project.loopers");
        let mut file = File::create(&path)?;

        let mut buf = BytesMut::with_capacity(session.encoded_len());
        session.encode(&mut buf)?;
        file.write_all(&buf)?;

        // save our last session
        let config_path = last_session_path()?;
        let mut last_session = File::create(config_path)?;
        write!(last_session, "{}", path.to_string_lossy())?;

        Ok(())
    }

    fn load_session(&mut self, command: &LoadSessionCommand) -> Result<(), SaveLoadError> {
        let mut file = File::open(&command.path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let path = Path::new(&command.path);
        let dir = path.parent().unwrap();

        let session: SavedSession = SavedSession::decode(&buf)?;
        self.metric_structure.time_signature = TimeSignature::new(
            session.time_signature_upper as u8,
            session.time_signature_lower as u8,
        )
        .expect(&format!(
            "Invalid time signature: {}/{}",
            session.time_signature_upper, session.time_signature_lower
        ));

        self.metric_structure.tempo = Tempo {
            mbpm: session.tempo_mbpm,
        };

        self.loopers.clear();
        for l in session.loopers {
            self.loopers.push(Looper::from_serialized(&l, dir)?);
        }

        Ok(())
    }

    fn handle_command(&mut self, command: &Command, triggered: bool) {
        if let Some(oneof) = &command.command_oneof {
            match oneof {
                CommandOneof::LooperCommand(lc) => {
                    self.handle_loop_command(lc, triggered);
                }
                CommandOneof::GlobalCommand(gc) => {
                    if let Some(typ) = GlobalCommandType::from_i32(gc.command) {
                        match typ as GlobalCommandType {
                            GlobalCommandType::ResetTime => {
                                self.time = 0;
                            }
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
                CommandOneof::SaveSessionCommand(command) => {
                    if let Err(e) = self.save_session(command) {
                        println!("Failed to save session {:?}", e);
                    }
                }
                CommandOneof::LoadSessionCommand(command) => {
                    if let Err(e) = self.load_session(command) {
                        println!("Failed to load session {:?}", e);
                    }
                }
                CommandOneof::MetronomeVolumeCommand(command) => {
                    if command.volume >= 0.0 && command.volume <= 1.0 {
                        if let Some(metronome) = &mut self.metronome {
                            metronome.set_volume(command.volume);
                        }
                    } else {
                        println!("Invalid metronome volume; must be between 0 and 1");
                    }
                }
            }
        }
    }

    fn play_loops(&self, outputs: &mut [Vec<f64>; 2]) {
        if self.time >= 0 {
            for looper in &self.loopers {
                if !looper.deleted
                    && (looper.mode == LooperMode::Playing || looper.mode == LooperMode::Overdub)
                {
                    looper.process_output(FrameTime(self.time as i64), outputs)
                }
            }
        }
    }

    // returns length
    fn measure_len(&self) -> FrameTime {
        let bps = self.metric_structure.tempo.bpm() as f32 / 60.0;
        let mspb = 1000.0 / bps;
        let mspm = mspb * self.metric_structure.time_signature.upper as f32;

        FrameTime::from_ms(mspm as f64)
    }

    // Step 1: Convert midi events to commands
    // Step 2: Handle commands
    // Step 3: Play current samples
    // Step 4: Record
    // Step 5: (async) Update GUI
    pub fn process(
        &mut self,
        in_bufs: [&[f32]; 2],
        out_bufs: &mut [&mut [f32]; 2],
        met_bufs: &mut [&mut [f32]; 2],
        frames: u64,
        midi_events: &[MidiEvent],
    ) {
        // Convert midi events to commands
        if !self.is_learning {
            self.commands_from_midi(midi_events);
            self.last_midi = None;
        } else {
            let new_last = midi_events.last().map(|m| m.bytes.to_vec());
            if new_last.is_some() {
                self.last_midi = new_last;
            }
        }

        // Update our state based on commands
        loop {
            let c = self.gui_input.pop();
            if let Ok(c) = c {
                self.handle_command(&c, false);
            } else {
                break;
            }
        }

        let buf_len = out_bufs[0].len();

        // create new output bufs from the input
        let mut out_64_vec: [Vec<f64>; 2] = [
            in_bufs[0].iter().map(|v| *v as f64).collect(),
            in_bufs[1].iter().map(|v| *v as f64).collect(),
        ];

        // Update our time
        let stopped = self.loopers.iter().all(|l| l.mode == LooperMode::None);
        let measure_len = self.measure_len();
        {
            if stopped && self.triggers.is_empty() {
                if let Some(m) = &mut self.metronome {
                    m.reset();
                }
                self.time = -(measure_len.0 as i64);
            } else {
                // process triggers
                let prev_beat_of_measure = self
                    .metric_structure
                    .time_signature
                    .beat_of_measure(self.metric_structure.tempo.beat(FrameTime(self.time)));

                let beat_of_measure = self.metric_structure.time_signature.beat_of_measure(
                    self.metric_structure
                        .tempo
                        .beat(FrameTime(self.time + frames as i64)),
                );

                let old_triggers: Vec<Trigger> = self.triggers.drain(..).collect();
                let mut beat0_triggers = vec![];
                self.triggers = vec![];

                for t in old_triggers {
                    let did_match = match t.condition {
                        TriggerCondition::BEAT0 => {
                            (stopped || prev_beat_of_measure != 0)
                                && beat_of_measure == 0
                                && self.time >= 0
                        }
                    };

                    if did_match && t.condition == TriggerCondition::BEAT0 {
                        beat0_triggers.push(t);
                    } else if did_match {
                        self.handle_command(&t.command, true);
                    } else {
                        self.triggers.push(t);
                    }
                }

                let active = self.active;

                let mut buf_index = 0;

                // We need to handle "beat 0" triggers specially, as the input buffer may not line
                // up exactly with our beats. Since this trigger is used to stop recording, we need
                // to ensure that we end up with exactly the right number of samples, no matter what
                // our buffer size is. We do that by splitting up the input into two pieces: the part
                // before beat 0 and the part after.
                if self.time >= 0 && !beat0_triggers.is_empty() {
                    if stopped {
                        // if we were previously stopped, we will cheat a bit and reset the time to
                        // *exactly* 0 (it will be something between 0 and frame_size)
                        self.time = 0;
                    }

                    let next_beat_time = self
                        .metric_structure
                        .tempo
                        .next_full_beat(FrameTime(self.time));
                    assert!(
                        next_beat_time.0 < self.time + frames as i64,
                        format!(
                            "{} >= {} (time = {})",
                            next_beat_time.0,
                            self.time + frames as i64,
                            self.time
                        )
                    );

                    let pre_size = (next_beat_time.0 - self.time) as usize;

                    if pre_size > 0 {
                        let looper = self.loopers.iter_mut().find(|l| l.id == active).unwrap();

                        // Record input to active loop
                        looper.process_input(
                            self.time as u64 + pre_size as u64,
                            &[&in_bufs[0][0..pre_size], &in_bufs[1][0..pre_size]],
                        );

                        buf_index = pre_size;
                    }

                    for t in beat0_triggers {
                        self.handle_command(&t.command, true);
                    }
                }

                self.time += frames as i64;

                // Play the metronome
                if let Some(metronome) = &mut self.metronome {
                    metronome.advance(met_bufs);
                }

                // Play our loops

                if self.time >= 0 {
                    self.play_loops(&mut out_64_vec);
                    let looper = self.loopers.iter_mut().find(|l| l.id == active).unwrap();

                    // Record input to active loop
                    looper.process_input(
                        self.time as u64,
                        &[
                            &in_bufs[0][buf_index..frames as usize],
                            &in_bufs[1][buf_index..frames as usize],
                        ],
                    );
                }
            }
        }

        for i in 0..buf_len {
            for j in 0..out_64_vec.len() {
                out_bufs[j][i] = out_64_vec[j][i] as f32
            }
        }

        // Update GUI

        // TODO: make this async or non-allocating
        let gui_output = &mut self.gui_output;
        let time = self.time as usize;
        let active = self.active;
        let loop_states: Vec<LoopState> = self
            .loopers
            .iter()
            .filter(|l| !l.deleted)
            .map(|l| {
                let len = l.length_in_samples() as usize;

                let t = if len > 0
                    && (l.mode == LooperMode::Playing || l.mode == LooperMode::Overdub)
                {
                    time % len
                } else {
                    0
                };

                LoopState {
                    id: l.id,
                    mode: l.mode as i32,
                    time: FrameTime(t as i64).to_ms() as i64,
                    length: FrameTime(len as i64).to_ms() as i64,
                    active: l.id == active,
                }
            })
            .collect();

        gui_output.push(State {
            loops: loop_states,
            time: FrameTime(self.time).to_ms() as i64,
            length: 0,
            beat: self
                .metric_structure
                .time_signature
                .beat_of_measure(self.metric_structure.tempo.beat(FrameTime(self.time)))
                as i64,
            bpm: self.metric_structure.tempo.bpm(),
            time_signature_upper: self.metric_structure.time_signature.upper as u64,
            time_signature_lower: self.metric_structure.time_signature.lower as u64,
            learn_mode: self.is_learning,
            last_midi: self
                .last_midi
                .as_ref()
                .map(|b| b.clone())
                .unwrap_or_else(|| vec![]),
            metronome_volume: self.metronome.as_ref().map_or(0.0, |m| m.get_volume()),
        });
    }
}
