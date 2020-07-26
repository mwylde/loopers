#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use crate::looper::Looper;
use crate::metronome::Metronome;
use crate::midi::MidiEvent;
use crate::sample::Sample;
use crate::session::SessionSaver;
use loopers_common::music::*;
use std::f32::NEG_INFINITY;
use std::fs::{create_dir_all, read_to_string};
use std::io;
use std::path::{Path, PathBuf};
use loopers_common::gui_channel::{GuiSender, GuiCommand, EngineStateSnapshot};
use crossbeam_channel::{Receiver};
use loopers_common::api::{Command, LooperCommand, LooperTarget, LooperMode, FrameTime};
use loopers_common::config::Config;
use crate::error::SaveLoadError;
use std::sync::Arc;

pub mod looper;
pub mod metronome;
pub mod midi;
pub mod sample;
pub mod session;
mod error;

#[derive(Eq, PartialEq)]
enum TriggerCondition {
    BEAT0,
}

struct Trigger {
    condition: TriggerCondition,
    command: LooperCommand,
    target: LooperTarget,
}

#[derive(Eq, PartialEq)]
enum EngineState {
    Stopped,
    Active,
}

pub struct Engine {
    config: Config,

    state: EngineState,

    time: i64,

    metric_structure: MetricStructure,

    command_input: Receiver<Command>,
    
    gui_sender: GuiSender,

    loopers: Vec<Looper>,
    active: u32,

    metronome: Option<Metronome>,

    triggers: Vec<Trigger>,

    id_counter: u32,

    is_learning: bool,
    last_midi: Option<Vec<u8>>,

    session_saver: SessionSaver,
}

#[allow(dead_code)]
const THRESHOLD: f32 = 0.05;

#[allow(dead_code)]
fn max_abs(b: &[f32]) -> f32 {
    b.iter()
        .map(|v| v.abs())
        .fold(NEG_INFINITY, |a, b| a.max(b))
}

pub fn last_session_path() -> io::Result<PathBuf> {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("loopers");
    create_dir_all(&config_path)?;
    config_path.push(".last-session");
    Ok(config_path)
}

impl Engine {
    pub fn new(
        config: Config,
        gui_sender: GuiSender,
        command_input: Receiver<Command>,
        beat_normal: Vec<f32>,
        beat_emphasis: Vec<f32>,
        restore: bool,
    ) -> Engine {
        let metric_structure = MetricStructure::new(4, 4, 120.0).unwrap();
        let mut engine = Engine {
            config,

            state: EngineState::Stopped,
            time: 0,

            metric_structure,

            gui_sender: gui_sender.clone(),
            command_input,

            loopers: vec![Looper::new(0, gui_sender).start()],
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

            session_saver: SessionSaver::new(),
        };

        for l in &engine.loopers {
            engine.session_saver.add_looper(l);
        }

        if restore {
            let mut restore_fn = || {
                let config_path = last_session_path()?;
                let restore_path = read_to_string(config_path)?;
                info!("Restoring from {}", restore_path);
                engine.load_session(Path::new(&restore_path))
            };

            if let Err(err) = restore_fn() {
                warn!("Failed to restore existing session {:?}", err);
            }
        }

        engine
    }

    fn looper_by_id_mut(&mut self, id: u32) -> Option<&mut Looper> {
        self.loopers.iter_mut().find(|l| l.id == id)
    }

    fn looper_by_index_mut(&mut self, idx: u8) -> Option<&mut Looper> {
        self.loopers.iter_mut().filter(|l| !l.deleted)
            .skip(idx as usize)
            .next()
    }

    fn commands_from_midi(&mut self, events: &[MidiEvent]) {
        for e in events {
            debug!("midi {:?}", e);
            if e.bytes.len() >= 3 {
                let command = self.config.midi_mappings.iter().find(|m|
                    e.bytes[1] == m.channel as u8 && e.bytes[2] == m.data as u8)
                    .map(|m| m.command.clone());

                if let Some(c) = command {
                    self.handle_command(&c, false);
                }
            }
        }
    }

    // possibly convert a loop command into a trigger
    fn trigger_from_command(lc: LooperCommand, target: LooperTarget) -> Option<Trigger> {
        use LooperCommand::*;
        match lc {
            Record | Overdub | RecordOverdubPlay => Some(Trigger {
                condition: TriggerCondition::BEAT0,
                command: lc,
                target,
            }),
            _ => None,
        }
    }

    fn handle_loop_command(&mut self, lc: LooperCommand, target: LooperTarget, triggered: bool) {
        debug!("Handling loop command: {:?} for {:?}", lc, target);

        if !triggered {
            if let Some(trigger) = Engine::trigger_from_command(lc, target) {
                self.triggers.push(trigger);
                return;
            }
        }

        match target {
            LooperTarget::Id(id) => {
                if let Some(l) = self.looper_by_id_mut(id) {
                    l.handle_command(lc);
                } else {
                    warn!("Could not find looper with id {} while handling command {:?}", id, lc);
                }
            }
            LooperTarget::Index(idx) => {
                if let Some(l) = self.looper_by_index_mut(idx) {
                    l.handle_command(lc);
                } else {
                    warn!("No looper at index {} while handling command {:?}", idx, lc);
                }
            }
            LooperTarget::All => {
                for l in &mut self.loopers {
                    l.handle_command(lc);
                }
            }
            LooperTarget::Selected => {
                if let Some(l) = self.looper_by_id_mut(self.active) {
                    l.handle_command(lc);
                } else {
                    error!("selected looper {} not found while handling command {:?}", self.active, lc);
                }
            }
        };
    }

    fn load_session(&mut self, _path: &Path) -> Result<(), SaveLoadError> {
        unimplemented!()
        // let mut file = File::open(&path)?;
        // let mut buf = Vec::new();
        // file.read_to_end(&mut buf)?;
        //
        // let path = Path::new(&command.path);
        // let dir = path.parent().unwrap();
        //
        // // let session: SavedSession = SavedSession::decode(&buf)?;
        // // self.metric_structure.time_signature = TimeSignature::new(
        // //     session.time_signature_upper as u8,
        // //     session.time_signature_lower as u8,
        // // )
        // // .expect(&format!(
        // //     "Invalid time signature: {}/{}",
        // //     session.time_signature_upper, session.time_signature_lower
        // // ));
        // //
        // // self.metric_structure.tempo = Tempo {
        // //     mbpm: session.tempo_mbpm,
        // // };
        // //
        // // for l in &self.loopers {
        // //     self.session_saver.remove_looper(l.id);
        // // }
        // // self.loopers.clear();
        // //
        // // for l in session.loopers {
        // //     let looper = Looper::from_serialized(
        // //         &l, dir, self.gui_sender.clone())?.start();
        // //     self.session_saver.add_looper(&looper);
        // //     self.loopers.push(looper);
        // // }
        //
        // Ok(())
    }

    fn handle_command(&mut self, command: &Command, triggered: bool) {
        use Command::*;
        match command {
            Looper(lc, target) => {
                self.handle_loop_command(*lc, *target, triggered);
            }
            Start => {
                self.state = EngineState::Active;
            }
            Stop => {
                self.state = EngineState::Stopped;
            }
            Reset => {
                if let Some(m) = &mut self.metronome {
                    m.reset();
                }
                self.set_time(FrameTime(-(self.measure_len().0 as i64)));
            }
            SetTime(time) => {
                self.set_time(*time)
            }
            AddLooper => {
                // TODO: make this non-allocating
                let looper = crate::Looper::new(self.id_counter, self.gui_sender.clone()).start();
                self.session_saver.add_looper(&looper);
                self.loopers.push(looper);
                self.active = self.id_counter;
                self.id_counter += 1;
            }
            SelectLooperById(id) => {
                if self.loopers.iter().any(|l| l.id == *id) {
                    self.active = *id;
                } else {
                    warn!("tried to select non-existent looper id {}", id);
                }
            }
            SelectLooperByIndex(idx) => {
                if let Some(l) = self.looper_by_index_mut(*idx) {
                    self.active = l.id;
                } else {
                    warn!("tried to select non-existent looper index {}", idx);
                }
            }
            SaveSession(path) => {
                if let Err(e) = self.session_saver.save_session(
                    self.metric_structure, Arc::clone(path)) {
                    error!("Failed to save session {:?}", e);
                }
            }
            LoadSession(path) => {
                if let Err(e) = self.load_session(path) {
                    error!("Failed to load session {:?}", e);
                }
            }
            SetMetronomeLevel(l) => {
                if *l <= 100 {
                    if let Some(metronome) = &mut self.metronome {
                        metronome.set_volume(*l as f32 / 100.0);
                    }
                } else {
                    error!("Invalid metronome volume; must be between 0 and 100");
                }
            }
        }
    }

    fn play_loops(&mut self, outputs: &mut [Vec<f64>; 2]) {
        if self.time >= 0 {
            for looper in self.loopers.iter_mut() {
                if !looper.deleted
                    && (looper.mode == LooperMode::Playing || looper.mode == LooperMode::Overdubbing)
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

    fn set_time(&mut self, time: FrameTime) {
        self.time = time.0;
        for l in &mut self.loopers {
            l.set_time(time);
        }
    }

    // Step 1: Convert midi events to commands
    // Step 2: Handle commands
    // Step 3: Play current samples
    // Step 4: Record
    // Step 5: Update GUI
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

        // Handle commands from the gui
        loop {
            match self.command_input.try_recv() {
                Ok(c) => {
                    self.handle_command(&c, false);
                },
                Err(_) => break,
            }
        }

        let buf_len = out_bufs[0].len();

        // create new output bufs from the input
        let mut out_64_vec: [Vec<f64>; 2] = [
            in_bufs[0].iter().map(|v| *v as f64).collect(),
            in_bufs[1].iter().map(|v| *v as f64).collect(),
        ];

        {
            if self.state == EngineState::Active {
                // process triggers
                let prev_beat_of_measure = self
                    .metric_structure
                    .time_signature
                    .beat_of_measure(self.metric_structure.tempo.beat(FrameTime(self.time)));

                let beat_of_measure = self.metric_structure.time_signature.beat_of_measure(
                    self.metric_structure
                        .tempo
                        .beat(FrameTime(self.time + frames as i64)));

                let old_triggers: Vec<Trigger> = self.triggers.drain(..).collect();
                let mut beat0_triggers = vec![];
                self.triggers = vec![];

                for t in old_triggers {
                    let did_match = match t.condition {
                        TriggerCondition::BEAT0 => {
                            (self.state == EngineState::Stopped || prev_beat_of_measure != 0)
                                && beat_of_measure == 0
                                && self.time >= 0
                        }
                    };

                    if did_match && t.condition == TriggerCondition::BEAT0 {
                        beat0_triggers.push(t);
                    } else if did_match {
                        self.handle_loop_command(t.command, t.target, true);
                    } else {
                        self.triggers.push(t);
                    }
                }

                let active = self.active;

                let mut pre_size = 0;

                // We need to handle "beat 0" triggers specially, as the input buffer may not line
                // up exactly with our beats. Since this trigger is used to stop recording, we need
                // to ensure that we end up with exactly the right number of samples, no matter what
                // our buffer size is. We do that by splitting up the input into two pieces: the part
                // before beat 0 and the part after.
                if self.time >= 0 && !beat0_triggers.is_empty() {
                    // if stopped {
                    //     // if we were previously stopped, we will cheat a bit and reset the time to
                    //     // *exactly* 0 (it will be something between 0 and frame_size)
                    //     self.set_time(FrameTime(0));
                    // }

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

                    pre_size = (next_beat_time.0 - self.time) as usize;

                    if pre_size > 0 {
                        let time = self.time as u64;
                        if let Some(looper) = self.looper_by_id_mut(active) {
                            // Record input to active loop
                            looper.process_input(
                                time,
                                &[&in_bufs[0][0..pre_size], &in_bufs[1][0..pre_size]],
                            );

                        }
                    }

                    for t in beat0_triggers {
                        self.handle_loop_command(t.command, t.target, true);
                    }
                }

                if self.time >= 0 {
                    // record rest of input
                    let time = self.time as u64;
                    if let Some(looper) = self.looper_by_id_mut(active) {
                        looper.process_input(
                            time + pre_size as u64,
                            &[
                                &in_bufs[0][pre_size..frames as usize],
                                &in_bufs[1][pre_size..frames as usize],
                            ],
                        );
                    }

                    // Play our loops
                    self.play_loops(&mut out_64_vec);
                }

                // Play the metronome
                if let Some(metronome) = &mut self.metronome {
                    metronome.advance(met_bufs);
                }

                self.time += frames as i64;
            }
        }

        for i in 0..buf_len {
            for j in 0..out_64_vec.len() {
                out_bufs[j][i] = out_64_vec[j][i] as f32
            }
        }

        // Update GUI
        self.gui_sender.send_update(GuiCommand::StateSnapshot(EngineStateSnapshot {
                time: FrameTime(self.time),
                metric_structure: self.metric_structure,
                active_looper: self.active,
                looper_count: self.loopers.len(),
        }));
    }
}
