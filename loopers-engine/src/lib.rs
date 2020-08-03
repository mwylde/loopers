#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use crate::error::SaveLoadError;
use crate::looper::Looper;
use crate::metronome::Metronome;
use crate::midi::MidiEvent;
use crate::sample::Sample;
use crate::session::SessionSaver;
use crate::trigger::{Trigger, TriggerCondition};
use crossbeam_channel::Receiver;
use loopers_common::api::{
    Command, FrameTime, LooperCommand, LooperMode, LooperTarget, SavedSession,
};
use loopers_common::config::Config;
use loopers_common::gui_channel::{EngineState, EngineStateSnapshot, GuiCommand, GuiSender};
use loopers_common::music::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::f32::NEG_INFINITY;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::Read;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod error;
pub mod looper;
pub mod metronome;
pub mod midi;
pub mod sample;
pub mod session;
mod trigger;

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

    triggers: BinaryHeap<Reverse<Trigger>>,

    id_counter: u32,

    is_learning: bool,
    last_midi: Option<Vec<u8>>,

    session_saver: SessionSaver,

    output_left: Vec<f64>,
    output_right: Vec<f64>,
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

            triggers: BinaryHeap::with_capacity(128),

            is_learning: false,
            last_midi: None,

            session_saver: SessionSaver::new(),

            output_left: vec![0f64; 2048],
            output_right: vec![0f64; 2048],
        };

        engine.reset();

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

    fn add_trigger(triggers: &mut BinaryHeap<Reverse<Trigger>>, t: Trigger) {
        while triggers.len() >= triggers.capacity() {
            triggers.pop();
        }

        triggers.push(Reverse(t));
    }

    fn reset(&mut self) {
        if let Some(m) = &mut self.metronome {
            m.reset();
        }
        self.triggers.clear();
        self.set_time(FrameTime(-(self.measure_len().0 as i64)));
    }

    fn looper_by_index_mut(&mut self, idx: u8) -> Option<&mut Looper> {
        self.loopers
            .iter_mut()
            .filter(|l| !l.deleted)
            .skip(idx as usize)
            .next()
    }

    fn commands_from_midi(&mut self, events: &[MidiEvent]) {
        for e in events {
            debug!("midi {:?}", e);
            if e.bytes.len() >= 3 {
                let command = self
                    .config
                    .midi_mappings
                    .iter()
                    .find(|m| e.bytes[1] == m.channel as u8 && e.bytes[2] == m.data as u8)
                    .map(|m| m.command.clone());

                if let Some(c) = command {
                    self.handle_command(&c, false);
                }
            }
        }
    }

    // possibly convert a loop command into a trigger
    fn trigger_from_command(
        ms: MetricStructure,
        time: FrameTime,
        lc: LooperCommand,
        target: LooperTarget,
        looper: &Looper,
    ) -> Option<Trigger> {
        use LooperCommand::*;
        match (looper.length_in_samples() == 0, looper.mode, lc) {
            (_, _, Record)
            | (_, LooperMode::Recording, _)
            | (true, _, RecordOverdubPlay)
            | (_, LooperMode::Overdubbing, _) => Some(Trigger::new(
                TriggerCondition::Measure,
                Command::Looper(lc, target),
                ms,
                time,
            ))
            .unwrap(),
            _ => None,
        }
    }

    fn handle_loop_command(&mut self, lc: LooperCommand, target: LooperTarget, triggered: bool) {
        debug!("Handling loop command: {:?} for {:?}", lc, target);

        let ms = self.metric_structure;
        let time = FrameTime(self.time);
        let triggers = &mut self.triggers;

        fn handle_or_trigger(
            triggered: bool,
            ms: MetricStructure,
            time: FrameTime,
            lc: LooperCommand,
            target: LooperTarget,
            looper: &mut Looper,
            triggers: &mut BinaryHeap<Reverse<Trigger>>,
        ) {
            if triggered {
                looper.handle_command(lc);
            } else if let Some(trigger) = Engine::trigger_from_command(ms, time, lc, target, looper)
            {
                Engine::add_trigger(triggers, trigger);
            } else {
                looper.handle_command(lc);
            }
        }

        let mut selected = None;
        match target {
            LooperTarget::Id(id) => {
                if let Some(l) = self.loopers.iter_mut().find(|l| l.id == id) {
                    selected = Some(l.id);
                    handle_or_trigger(triggered, ms, time, lc, target, l, triggers);
                } else {
                    warn!(
                        "Could not find looper with id {} while handling command {:?}",
                        id, lc
                    );
                }
            }
            LooperTarget::Index(idx) => {
                if let Some(l) = self.loopers.get_mut(idx as usize) {
                    selected = Some(l.id);
                    handle_or_trigger(triggered, ms, time, lc, target, l, triggers);
                } else {
                    warn!("No looper at index {} while handling command {:?}", idx, lc);
                }
            }
            LooperTarget::All => {
                for l in &mut self.loopers {
                    handle_or_trigger(triggered, ms, time, lc, target, l, triggers);
                }
            }
            LooperTarget::Selected => {
                let active = self.active;
                if let Some(l) = self.loopers.iter_mut().find(|l| l.id == active) {
                    handle_or_trigger(triggered, ms, time, lc, target, l, triggers);
                } else {
                    error!(
                        "selected looper {} not found while handling command {:?}",
                        self.active, lc
                    );
                }
            }
        };

        if let Some(id) = selected {
            self.active = id;
        }
    }

    fn load_session(&mut self, path: &Path) -> Result<(), SaveLoadError> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let dir = path.parent().unwrap();

        let mut session: SavedSession = toml::from_str(&contents).map_err(|err| {
            debug!("Found invalid SavedSession during load: {:?}", err);
            SaveLoadError::OtherError("Failed to restore session; file is invalid".to_string())
        })?;

        debug!("Restoring session: {:?}", session);

        self.metric_structure = session.metric_structure;

        if let Some(metronome) = &mut self.metronome {
            metronome.set_volume((session.metronome_volume as f32 / 100.0).min(1.0).max(0.0));
        }

        for l in &self.loopers {
            self.session_saver.remove_looper(l.id);
        }
        self.loopers.clear();

        session.loopers.sort_by_key(|l| l.id);

        for l in session.loopers {
            let looper = Looper::from_serialized(&l, dir, self.gui_sender.clone())?.start();
            self.session_saver.add_looper(&looper);
            self.loopers.push(looper);
        }

        self.id_counter = self.loopers.iter().map(|l| l.id).max().unwrap_or(0) + 1;

        Ok(())
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
                self.reset();
            }
            StartStop => {
                self.state = match self.state {
                    EngineState::Stopped => EngineState::Active,
                    EngineState::Active => {
                        self.reset();
                        EngineState::Stopped
                    }
                };
            }
            Reset => {
                self.reset();
            }
            SetTime(time) => self.set_time(*time),
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
                    self.metric_structure,
                    self.metronome
                        .as_ref()
                        .map(|m| (m.get_volume() * 100.0) as u8)
                        .unwrap_or(100),
                    Arc::clone(path),
                ) {
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
            SetTempoBPM(bpm) => {
                self.metric_structure.tempo = Tempo::from_bpm(*bpm);
                if let Some(met) = &mut self.metronome {
                    met.set_metric_structure(self.metric_structure);
                }
                self.reset();
            }
            SetTimeSignature(upper, lower) => {
                if let Some(ts) = TimeSignature::new(*upper, *lower) {
                    self.metric_structure.time_signature = ts;
                    if let Some(met) = &mut self.metronome {
                        met.set_metric_structure(self.metric_structure);
                    }
                    self.reset();
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

    fn perform_looper_io(&mut self, in_bufs: &[&[f32]], time: FrameTime, idx_range: Range<usize>) {
        if time.0 >= 0 {
            // play the loops
            for looper in self.loopers.iter_mut() {
                if !looper.deleted {
                    let mut o = [
                        &mut self.output_left[idx_range.clone()],
                        &mut self.output_right[idx_range.clone()],
                    ];

                    looper.process_output(time, &mut o);

                    looper.process_input(
                        time.0 as u64,
                        &[
                            &in_bufs[0][idx_range.clone()],
                            &in_bufs[1][idx_range.clone()],
                        ],
                    );
                }
            }
        } else {
            error!("perform_looper_io called with negative time {}", time.0);
        }
    }

    fn process_loopers(&mut self, in_bufs: &[&[f32]], frames: u64) {
        let mut time = self.time;
        let mut idx = 0usize;

        if time < 0 {
            time = (self.time + frames as i64).min(0);
            if time < 0 {
                return;
            }
            idx = (time - self.time) as usize;
        }

        let mut time = time as u64;

        let next_time = (self.time + frames as i64) as u64;
        while time < next_time {
            if let Some(_) = self
                .triggers
                .peek()
                .filter(|t| t.0.triggered_at().0 < next_time as i64)
            {
                let trigger = self.triggers.pop().unwrap();

                let trigger_at = trigger.0.triggered_at();
                // we'll process up to this time, then trigger the trigger

                if trigger_at.0 < time as i64 {
                    // we failed to trigger, but don't know if it's safe to trigger late. so we'll
                    // just ignore it. there might be better solutions for specific triggers, but
                    // hopefully this is rare.
                    error!(
                        "missed trigger for time {} (cur time = {})",
                        trigger_at.0, time
                    );
                    continue;
                }

                // we know that trigger_at is non-negative from the previous condition
                let trigger_at = trigger_at.0 as u64;

                // if we're exactly on the trigger time, just trigger it immediately and continue
                if trigger_at > time {
                    // otherwise, we need to process the stuff before the trigger time, then trigger
                    // the command, then continue processing the rest
                    let idx_range = idx..(trigger_at as i64 - self.time) as usize;
                    assert_eq!(
                        idx_range.end - idx_range.start,
                        (trigger_at - time) as usize
                    );

                    self.perform_looper_io(&in_bufs, FrameTime(time as i64), idx_range.clone());
                    time = trigger_at;
                    idx = idx_range.end;
                }

                self.handle_command(&trigger.0.command, true);
            } else {
                // there are no more triggers for this period, so just process the rest and finish
                self.perform_looper_io(&in_bufs, FrameTime(time as i64), idx..frames as usize);
                time = next_time;
            }
        }
    }

    fn compute_peaks(in_bufs: &[&[f32]]) -> [f32; 2] {
        let mut peaks = [0f32; 2];
        for c in 0..2 {
            let mut peak = 0f32;
            for v in in_bufs[c] {
                let v_abs = v.abs();
                if v_abs > peak {
                    peak = v_abs;
                }
            }

            peaks[c] = 20.0 * peak.log10();
        }

        peaks
    }

    // Step 1: Convert midi events to commands
    // Step 2: Handle commands
    // Step 3: Play current samples
    // Step 4: Record
    // Step 5: Update GUI
    pub fn process(
        &mut self,
        in_bufs: [&[f32]; 2],
        out_l: &mut [f32],
        out_r: &mut [f32],
        mut met_bufs: [&mut [f32]; 2],
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
                }
                Err(_) => break,
            }
        }

        // ensure out internal output buffer is big enough (this should only allocate when the
        // frame size is increased)
        while self.output_left.len() < frames as usize {
            self.output_left.push(0.0);
        }
        while self.output_right.len() < frames as usize {
            self.output_right.push(0.0);
        }

        // copy the input to the output for monitoring
        // TODO: should probably make this behavior configurable
        for (i, (l, r)) in in_bufs[0].iter().zip(in_bufs[1]).enumerate() {
            self.output_left[i] = *l as f64;
            self.output_right[i] = *r as f64;
        }

        if !self.triggers.is_empty() {
            self.state = EngineState::Active;
        }

        if self.state == EngineState::Active {
            // process the loopers
            self.process_loopers(&in_bufs, frames);

            // Play the metronome
            if let Some(metronome) = &mut self.metronome {
                metronome.advance(&mut met_bufs);
            }

            self.time += frames as i64;
        }

        for i in 0..frames as usize {
            out_l[i] = self.output_left[i] as f32;
        }
        for i in 0..frames as usize {
            out_r[i] = self.output_right[i] as f32;
        }

        // Update GUI
        self.gui_sender
            .send_update(GuiCommand::StateSnapshot(EngineStateSnapshot {
                engine_state: self.state,
                time: FrameTime(self.time),
                metric_structure: self.metric_structure,
                active_looper: self.active,
                looper_count: self.loopers.len(),
                input_levels: Self::compute_peaks(&in_bufs),
                metronome_volume: self
                    .metronome
                    .as_ref()
                    .map(|m| m.get_volume())
                    .unwrap_or(0.0),
            }));
    }
}
