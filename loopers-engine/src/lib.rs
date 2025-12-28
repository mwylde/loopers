#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::collections::VecDeque;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::Receiver;

use loopers_common::api::QuantizationMode::Free;
use loopers_common::api::{
    get_sample_rate, set_sample_rate, Command, FrameTime, LooperCommand, LooperMode, LooperTarget,
    Part, PartSet, QuantizationMode, SavedSession,
};
use loopers_common::config::{Config, MidiMapping, FILE_HEADER};
use loopers_common::gui_channel::{
    EngineState, EngineStateSnapshot, GuiCommand, GuiSender, LogMessage,
};
use loopers_common::midi::MidiEvent;
use loopers_common::music::*;
use loopers_common::Host;

use crate::error::SaveLoadError;
use crate::looper::Looper;
use crate::metronome::Metronome;
use crate::sample::Sample;
use crate::session::{SaveSessionData, SessionSaver};
use crate::trigger::{Trigger, TriggerCondition};

mod error;
pub mod looper;
pub mod metronome;
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

    current_part: Part,

    sync_mode: QuantizationMode,

    metronome: Option<Metronome>,

    triggers: VecDeque<Trigger>,

    id_counter: u32,

    session_saver: SessionSaver,

    tmp_left: Vec<f64>,
    tmp_right: Vec<f64>,
    output_left: Vec<f64>,
    output_right: Vec<f64>,

    looper_peaks: [[f32; 2]; 64],
}

#[allow(dead_code)]
const THRESHOLD: f32 = 0.05;

#[allow(dead_code)]
fn max_abs(b: &[f32]) -> f32 {
    b.iter()
        .map(|v| v.abs())
        .fold(f32::NEG_INFINITY, |a, b| a.max(b))
}

pub fn last_session_path() -> io::Result<PathBuf> {
    let mut config_path = dirs::config_dir().unwrap_or_default();
    config_path.push("loopers");
    create_dir_all(&config_path)?;
    config_path.push(".last-session");
    Ok(config_path)
}

pub fn read_config() -> Result<Config, String> {
    let mut mapping_path = dirs::config_dir().unwrap_or_default();
    mapping_path.push("loopers/midi_mappings.tsv");

    let mut config = Config::default();

    match File::open(&mapping_path) {
        Ok(file) => match MidiMapping::from_file(&mapping_path.to_string_lossy(), &file) {
            Ok(mms) => config.midi_mappings.extend(mms),
            Err(e) => {
                return Err(format!("Failed to load midi mappings: {:?}", e));
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // try to create an empty config file if it doesn't exist
            if let Ok(ref mut file) = File::create(&mapping_path) {
                writeln!(file, "{}", FILE_HEADER).unwrap();
            }
        }
        Err(_) => {}
    }

    Ok(config)
}

impl Engine {
    pub fn new<'a, H: Host<'a>>(
        host: &mut H,
        mut gui_sender: GuiSender,
        command_input: Receiver<Command>,
        beat_normal: Vec<f32>,
        beat_emphasis: Vec<f32>,
        restore: bool,
        sample_rate: usize,
    ) -> Engine {
        let metric_structure = MetricStructure::new(4, 4, Tempo::from_bpm(120.0)).unwrap();

        let config = match read_config() {
            Ok(config) => config,
            Err(err) => {
                let mut error = LogMessage::error();
                if let Err(e) = write!(error, "{}", err) {
                    error!("Failed to report config error: {}", e);
                } else {
                    gui_sender.send_log(error);
                }

                Config::default()
            }
        };

        let mut engine = Engine {
            config,

            state: EngineState::Stopped,
            time: 0,

            metric_structure,

            gui_sender: gui_sender.clone(),
            command_input,

            loopers: vec![Looper::new(0, PartSet::new(), gui_sender.clone()).start()],
            active: 0,
            current_part: Part::A,

            sync_mode: QuantizationMode::Measure,

            id_counter: 1,

            metronome: Some(Metronome::new(
                metric_structure,
                Sample::from_mono(&beat_normal),
                Sample::from_mono(&beat_emphasis),
            )),

            triggers: VecDeque::with_capacity(128),

            session_saver: SessionSaver::new(gui_sender),

            tmp_left: vec![0f64; 2048],
            tmp_right: vec![0f64; 2048],

            output_left: vec![0f64; 2048],
            output_right: vec![0f64; 2048],

            looper_peaks: [[0.0; 2]; 64],
        };

        set_sample_rate(sample_rate);

        engine.reset();

        for l in &engine.loopers {
            engine.session_saver.add_looper(l);
            if let Err(e) = host.add_looper(l.id) {
                error!("Failed to add host port for looper {}: {}", l.id, e);
            }
        }

        if restore {
            let mut restore_fn = || {
                let config_path = last_session_path()?;
                let restore_path = read_to_string(config_path)?;
                info!("Restoring from {}", restore_path);
                engine.load_session(host, Path::new(&restore_path))
            };

            if let Err(err) = restore_fn() {
                warn!("Failed to restore existing session {:?}", err);
            }
        }

        engine
    }

    fn add_trigger(triggers: &mut VecDeque<Trigger>, t: Trigger) {
        while triggers.len() >= triggers.capacity() {
            triggers.pop_front();
        }

        triggers.push_back(t);
    }

    fn reset(&mut self) {
        if let Some(m) = &mut self.metronome {
            m.reset();
        }
        self.triggers.clear();
        self.set_time(FrameTime(-self.measure_len().0));
        for l in &mut self.loopers {
            l.handle_command(LooperCommand::Play);
        }
    }

    fn looper_by_index_mut(&mut self, idx: u8) -> Option<&mut Looper> {
        self.loopers
            .iter_mut()
            .filter(|l| !l.deleted)
            .nth(idx as usize)
    }

    fn commands_from_midi<'a, H: Host<'a>>(&mut self, host: &mut H, events: &[MidiEvent]) {
        for e in events {
            debug!("midi {:?}", e);
            for i in 0..self.config.midi_mappings.len() {
                let mm = &self.config.midi_mappings[i];
                if let Some(c) = mm.command_for_event(e) {
                    self.handle_command(host, &c, false);
                }
            }
        }
    }

    // possibly convert a loop command into a trigger
    fn trigger_from_command(
        ms: MetricStructure,
        sync_mode: QuantizationMode,
        time: FrameTime,
        lc: LooperCommand,
        target: LooperTarget,
        looper: &Looper,
    ) -> Option<Trigger> {
        let trigger_condition = match sync_mode {
            Free => {
                if time.0 < 0 {
                    Some(TriggerCondition::Beat)
                } else {
                    None
                }
            }
            QuantizationMode::Beat => Some(TriggerCondition::Beat),
            QuantizationMode::Measure => Some(TriggerCondition::Measure),
        }?;

        use LooperCommand::*;
        match (looper.length() == 0, looper.mode(), lc) {
            // SetLevel and SetPan should apply immediately
            (_, _, SetLevel(_)) => None,
            (_, _, SetPan(_)) => None,

            (_, _, Record)
            | (_, LooperMode::Recording, _)
            | (true, _, RecordOverdubPlay)
            | (_, LooperMode::Overdubbing, _) => Some(Trigger::new(
                trigger_condition,
                Command::Looper(lc, target),
                ms,
                time,
            )),
            (_, _, RecordOverdubPlay) => Some(Trigger::new(
                TriggerCondition::Immediate,
                Command::Looper(lc, target),
                ms,
                time,
            )),
            _ => None,
        }
    }

    fn handle_loop_command(&mut self, lc: LooperCommand, target: LooperTarget, triggered: bool) {
        debug!("Handling loop command: {:?} for {:?}", lc, target);

        let ms = self.metric_structure;
        let sync_mode = self.sync_mode;
        let time = FrameTime(self.time);
        let triggers = &mut self.triggers;
        let gui_sender = &mut self.gui_sender;

        #[allow(clippy::too_many_arguments)]
        fn handle_or_trigger(
            triggered: bool,
            ms: MetricStructure,
            sync_mode: QuantizationMode,
            time: FrameTime,
            lc: LooperCommand,
            target: LooperTarget,
            looper: &mut Looper,
            triggers: &mut VecDeque<Trigger>,
            gui_sender: &mut GuiSender,
        ) {
            if triggered {
                looper.handle_command(lc);
            } else if let Some(trigger) =
                Engine::trigger_from_command(ms, sync_mode, time, lc, target, looper)
            {
                Engine::add_trigger(triggers, trigger.clone());

                gui_sender.send_update(GuiCommand::AddLoopTrigger(
                    looper.id,
                    trigger.triggered_at(),
                    lc,
                ));
            } else {
                looper.handle_command(lc);
            }
        }

        let mut selected = None;
        match target {
            LooperTarget::Id(id) => {
                if let Some(l) = self.loopers.iter_mut().find(|l| l.id == id) {
                    handle_or_trigger(
                        triggered, ms, sync_mode, time, lc, target, l, triggers, gui_sender,
                    );
                } else {
                    warn!(
                        "Could not find looper with id {} while handling command {:?}",
                        id, lc
                    );
                }
            }
            LooperTarget::Index(idx) => {
                if let Some(l) = self
                    .loopers
                    .iter_mut()
                    .filter(|l| !l.deleted)
                    .nth(idx as usize)
                {
                    selected = Some(l.id);
                    handle_or_trigger(
                        triggered, ms, sync_mode, time, lc, target, l, triggers, gui_sender,
                    );
                } else {
                    warn!("No looper at index {} while handling command {:?}", idx, lc);
                }
            }
            LooperTarget::All => {
                for l in &mut self.loopers {
                    handle_or_trigger(
                        triggered, ms, sync_mode, time, lc, target, l, triggers, gui_sender,
                    );
                }
            }
            LooperTarget::Selected => {
                let active = self.active;
                if let Some(l) = self.loopers.iter_mut().find(|l| l.id == active) {
                    handle_or_trigger(
                        triggered, ms, sync_mode, time, lc, target, l, triggers, gui_sender,
                    );
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

    fn load_session<'a, H: Host<'a>>(
        &mut self,
        host: &mut H,
        path: &Path,
    ) -> Result<(), SaveLoadError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let dir = path.parent().unwrap();

        let mut session: SavedSession = serde_json::from_str(&contents).map_err(|err| {
            warn!("Found invalid SavedSession during load: {:?}", err);
            // TODO: improve these error messages
            SaveLoadError::OtherError("Failed to restore session; file is invalid".to_string())
        })?;

        if session.sample_rate != get_sample_rate() {
            let mut error = LogMessage::error();
            if write!(
                &mut error,
                "Session was saved with sample rate {}, but system rate \
            is set to {}, playback will be affected",
                session.sample_rate,
                get_sample_rate()
            )
            .is_err()
            {
                error!("Different sample rate");
            };
            self.gui_sender.send_log(error);
        }

        debug!("Restoring session: {:?}", session);

        self.metric_structure = session
            .metric_structure
            .to_ms()
            .map_err(SaveLoadError::OtherError)?;
        self.sync_mode = session.sync_mode;

        if let Some(metronome) = &mut self.metronome {
            metronome.set_volume((session.metronome_volume as f32 / 100.0).clamp(0.0, 1.0));
        }

        for l in &self.loopers {
            self.session_saver.remove_looper(l.id);
            self.gui_sender.send_update(GuiCommand::RemoveLooper(l.id));
        }
        self.loopers.clear();

        session.loopers.sort_by_key(|l| l.id);

        for l in session.loopers {
            debug!("Restoring looper {}", l.id);
            let looper = Looper::from_serialized(&l, dir, self.gui_sender.clone())?.start();
            self.session_saver.add_looper(&looper);
            if let Err(e) = host.add_looper(looper.id) {
                error!("Failed to create host port for looper {}: {}", looper.id, e);
            }
            self.loopers.push(looper);
        }

        self.id_counter = self.loopers.iter().map(|l| l.id).max().unwrap_or(0) + 1;

        self.reset();

        Ok(())
    }

    fn handle_command<'a, H: Host<'a>>(
        &mut self,
        host: &mut H,
        command: &Command,
        triggered: bool,
    ) {
        fn trigger_or_run<F>(
            engine: &mut Engine,
            command: &Command,
            triggered: bool,
            queued: bool,
            f: F,
        ) where
            F: FnOnce(&mut Engine),
        {
            if engine.state == EngineState::Stopped || triggered {
                f(engine);
                return;
            }

            let trigger_condition = match (queued, engine.sync_mode) {
                (true, _) => TriggerCondition::Immediate,
                (false, QuantizationMode::Free) => TriggerCondition::Immediate,
                (false, QuantizationMode::Beat) => TriggerCondition::Beat,
                (false, QuantizationMode::Measure) => TriggerCondition::Measure,
            };

            let trigger = Trigger::new(
                trigger_condition,
                command.clone(),
                engine.metric_structure,
                FrameTime(engine.time),
            );

            if trigger.triggered_at() != FrameTime(0)
                && trigger.triggered_at() < FrameTime(engine.time)
            {
                f(engine);
                return;
            }

            Engine::add_trigger(&mut engine.triggers, trigger.clone());
            engine.gui_sender.send_update(GuiCommand::AddGlobalTrigger(
                trigger.triggered_at(),
                trigger.command,
            ));
        }

        use Command::*;
        match command {
            Looper(lc, target) => {
                self.handle_loop_command(*lc, *target, triggered);
            }
            Start => {
                self.state = EngineState::Active;
            }
            Pause => {
                self.state = EngineState::Paused;
            }
            Stop => {
                self.state = EngineState::Stopped;
                self.reset();
            }
            StartStop => {
                self.state = match self.state {
                    EngineState::Stopped | EngineState::Paused => EngineState::Active,
                    EngineState::Active => {
                        self.reset();
                        EngineState::Stopped
                    }
                };
            }
            PlayPause => {
                self.state = match self.state {
                    EngineState::Stopped | EngineState::Paused => EngineState::Active,
                    EngineState::Active => EngineState::Paused,
                }
            }
            Reset => {
                self.reset();
            }
            SetTime(time) => self.set_time(*time),
            AddLooper => {
                // TODO: make this non-allocating
                let looper = crate::Looper::new(
                    self.id_counter,
                    PartSet::with(self.current_part),
                    self.gui_sender.clone(),
                )
                .start();
                self.session_saver.add_looper(&looper);
                self.loopers.push(looper);
                self.active = self.id_counter;
                // TODO: better error handling
                if let Err(e) = host.add_looper(self.id_counter) {
                    error!(
                        "failed to create host port for looper {}: {}",
                        self.id_counter, e
                    );
                }
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
            SelectNextLooper | SelectPreviousLooper => {
                trigger_or_run(self, command, triggered, true, |engine| {
                    if let Some((i, _)) = engine
                        .loopers
                        .iter()
                        .filter(|l| !l.deleted)
                        .filter(|l| l.parts[engine.current_part])
                        .enumerate()
                        .find(|(_, l)| l.id == engine.active)
                    {
                        let count = engine
                            .loopers
                            .iter()
                            .filter(|l| l.parts[engine.current_part])
                            .filter(|l| !l.deleted)
                            .count();

                        let next = if *command == SelectNextLooper {
                            (i + 1) % count
                        } else {
                            (i as isize - 1).rem_euclid(count as isize) as usize
                        };

                        if let Some(l) = engine
                            .loopers
                            .iter()
                            .filter(|l| l.parts[engine.current_part])
                            .filter(|l| !l.deleted)
                            .nth(next)
                        {
                            engine.active = l.id;
                        }
                    } else if let Some(l) = engine
                        .loopers
                        .iter()
                        .filter(|l| !l.deleted)
                        .find(|l| l.parts[engine.current_part])
                    {
                        engine.active = l.id;
                    }
                });
            }
            PreviousPart => {
                trigger_or_run(self, command, triggered, false, |engine| {
                    let original = engine.current_part;
                    loop {
                        engine.current_part = match engine.current_part {
                            Part::A => Part::D,
                            Part::B => Part::A,
                            Part::C => Part::B,
                            Part::D => Part::C,
                        };
                        if engine
                            .loopers
                            .iter()
                            .any(|l| !l.deleted && l.parts[engine.current_part])
                            || engine.current_part == original
                        {
                            break;
                        }
                    }
                    engine.select_first_in_part();
                });
            }
            NextPart => {
                trigger_or_run(self, command, triggered, false, |engine| {
                    let original = engine.current_part;
                    loop {
                        engine.current_part = match engine.current_part {
                            Part::A => Part::B,
                            Part::B => Part::C,
                            Part::C => Part::D,
                            Part::D => Part::A,
                        };
                        if engine
                            .loopers
                            .iter()
                            .any(|l| !l.deleted && l.parts[engine.current_part])
                            || engine.current_part == original
                        {
                            break;
                        }
                    }
                    engine.select_first_in_part();
                });
            }
            GoToPart(part) => {
                trigger_or_run(self, command, triggered, false, |engine| {
                    engine.current_part = *part;
                    engine.select_first_in_part();
                });
            }
            SetQuantizationMode(sync_mode) => {
                self.sync_mode = *sync_mode;
            }
            SaveSession(path) => {
                if let Err(e) = self.session_saver.save_session(SaveSessionData {
                    metric_structure: self.metric_structure,
                    metronome_volume: self
                        .metronome
                        .as_ref()
                        .map(|m| (m.get_volume() * 100.0) as u8)
                        .unwrap_or(100),
                    sync_mode: self.sync_mode,
                    path: Arc::clone(path),
                    sample_rate: get_sample_rate(),
                }) {
                    error!("Failed to save session {:?}", e);
                }
            }
            LoadSession(path) => {
                if let Err(e) = self.load_session(host, path) {
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

    // selects the first looper in the part, unless the current selection is already in the part
    fn select_first_in_part(&mut self) {
        if let Some(l) = self
            .loopers
            .iter()
            .find(|l| l.id == self.active && l.parts[self.current_part])
            .or(self
                .loopers
                .iter()
                .find(|l| !l.deleted && l.parts[self.current_part]))
        {
            self.active = l.id;
        }
    }

    // returns length
    fn measure_len(&self) -> FrameTime {
        let bps = self.metric_structure.tempo.bpm() / 60.0;
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

    fn perform_looper_io<'a, H: Host<'a>>(
        &mut self,
        host: &mut H,
        in_bufs: &[&[f32]],
        time: FrameTime,
        idx_range: Range<usize>,
        solo: bool,
    ) {
        if time.0 >= 0 {
            let mut looper_index = 0;
            for looper in self.loopers.iter_mut() {
                if !looper.deleted {
                    self.tmp_left.iter_mut().for_each(|i| *i = 0.0);
                    self.tmp_right.iter_mut().for_each(|i| *i = 0.0);

                    let mut o = [
                        &mut self.tmp_left[idx_range.clone()],
                        &mut self.tmp_right[idx_range.clone()],
                    ];

                    looper.process_output(time, &mut o, self.current_part, solo);

                    // copy the output to the looper input in the host, if we can find one
                    if let Some([l, r]) = host.output_for_looper(looper.id) {
                        l.iter_mut()
                            .zip(&self.tmp_left[idx_range.clone()])
                            .for_each(|(a, b)| *a = *b as f32);
                        r.iter_mut()
                            .zip(&self.tmp_right[idx_range.clone()])
                            .for_each(|(a, b)| *a = *b as f32);
                    }

                    // copy the output to the our main output
                    self.output_left[idx_range.clone()]
                        .iter_mut()
                        .zip(&self.tmp_left[idx_range.clone()])
                        .for_each(|(a, b)| *a += *b);
                    self.output_right[idx_range.clone()]
                        .iter_mut()
                        .zip(&self.tmp_right[idx_range.clone()])
                        .for_each(|(a, b)| *a += *b);

                    // update our peaks
                    let mut peaks = [0f32; 2];
                    for (i, vs) in [&self.tmp_left, &self.tmp_right].iter().enumerate() {
                        for v in *vs {
                            let v_abs = v.abs() as f32;
                            if v_abs > peaks[i] {
                                peaks[i] = v_abs;
                            }
                        }
                    }

                    if let Some(p) = self.looper_peaks.get_mut(looper_index) {
                        *p = peaks;
                    }
                    looper_index += 1;

                    looper.process_input(
                        time.0 as u64,
                        &[
                            &in_bufs[0][idx_range.clone()],
                            &in_bufs[1][idx_range.clone()],
                        ],
                        self.current_part,
                    );
                }
            }
        } else {
            error!("perform_looper_io called with negative time {}", time.0);
        }
    }

    fn process_loopers<'a, H: Host<'a>>(
        &mut self,
        host: &mut H,
        in_bufs: &[&[f32]],
        frames: u64,
        solo: bool,
    ) {
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
            if self
                .triggers
                .iter()
                .peekable()
                .peek()
                .filter(|t| t.triggered_at().0 < next_time as i64)
                .is_some()
            {
                // The unwrap is safe due to the preceding peek
                let trigger = self.triggers.pop_front().unwrap();

                let trigger_at = trigger.triggered_at();
                // we'll process up to this time, then trigger the trigger

                if trigger_at != FrameTime(0) && trigger_at.0 < time as i64 {
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

                    self.perform_looper_io(
                        host,
                        in_bufs,
                        FrameTime(time as i64),
                        idx_range.clone(),
                        solo,
                    );
                    time = trigger_at;
                    idx = idx_range.end;
                }

                self.handle_command(host, &trigger.command, true);
            } else {
                // there are no more triggers for this period, so just process the rest and finish
                self.perform_looper_io(
                    host,
                    in_bufs,
                    FrameTime(time as i64),
                    idx..frames as usize,
                    solo,
                );
                time = next_time;
            }
        }
    }

    fn compute_peaks(in_bufs: &[&[f32]]) -> [u8; 2] {
        let mut peaks = [0u8; 2];
        for c in 0..2 {
            let mut peak = 0f32;
            for v in in_bufs[c] {
                let v_abs = v.abs();
                if v_abs > peak {
                    peak = v_abs;
                }
            }

            peaks[c] = Self::iec_scale(peak);
        }

        peaks
    }

    fn iec_scale(amp: f32) -> u8 {
        let db = 20.0 * amp.log10();

        let d = if db < -70.0 {
            0.0
        } else if db < -60.0 {
            db + 70.0 * 0.25
        } else if db < -50.0 {
            db + 60.0 * 0.5 + 5.0
        } else if db < -40.0 {
            db + 50.0 * 0.75 + 7.5
        } else if db < -30.0 {
            db + 40.0 * 1.5 + 15.0
        } else if db < -20.0 {
            db + 30.0 * 2.0 + 30.0
        } else if db < 0.0 {
            db + 20.0 * 2.5 + 50.0
        } else {
            100.0
        };

        d as u8
    }

    // Step 1: Convert midi events to commands
    // Step 2: Handle commands
    // Step 3: Play current samples
    // Step 4: Record
    // Step 5: Update GUI
    #[allow(clippy::too_many_arguments)]
    pub fn process<'a, H: Host<'a>>(
        &mut self,
        host: &mut H,
        in_bufs: [&[f32]; 2],
        out_l: &mut [f32],
        out_r: &mut [f32],
        mut met_bufs: [&mut [f32]; 2],
        frames: u64,
        midi_events: &[MidiEvent],
    ) {
        // Convert midi events to commands
        self.commands_from_midi(host, midi_events);

        // Handle commands from the gui
        while let Ok(c) = self.command_input.try_recv() {
            self.handle_command(host, &c, false);
        }

        // Remove any deleted loopers
        for l in self.loopers.iter().filter(|l| l.deleted) {
            self.session_saver.remove_looper(l.id);
        }
        self.loopers.retain(|l| !l.deleted);

        // ensure out internal output buffer is big enough (this should only allocate when the
        // buffer size is increased)
        while self.output_left.len() < frames as usize {
            self.output_left.push(0.0);
        }
        while self.output_right.len() < frames as usize {
            self.output_right.push(0.0);
        }
        while self.tmp_left.len() < frames as usize {
            self.output_left.push(0.0);
        }
        while self.tmp_right.len() < frames as usize {
            self.output_right.push(0.0);
        }

        // copy the input to the output for monitoring
        // TODO: should probably make this behavior configurable
        for (i, (l, r)) in in_bufs[0].iter().zip(in_bufs[1]).enumerate() {
            self.output_left[i] = *l as f64;
            self.output_right[i] = *r as f64;
        }

        if (self.state != EngineState::Active && self.state != EngineState::Paused)
            && (!self.triggers.is_empty()
                || self.loopers.iter().any(|l| {
                    l.local_mode() == LooperMode::Recording
                        || l.local_mode() == LooperMode::Overdubbing
                }))
        {
            self.state = EngineState::Active;
        }

        let solo = self
            .loopers
            .iter()
            .any(|l| l.parts[self.current_part] && !l.deleted && l.mode() == LooperMode::Soloed);

        if self.state == EngineState::Active {
            // process the loopers
            self.process_loopers(host, &in_bufs, frames, solo);

            // Play the metronome
            if let Some(metronome) = &mut self.metronome {
                metronome.advance(&mut met_bufs);
            }

            self.time += frames as i64;
        }

        #[allow(clippy::needless_range_loop)]
        for i in 0..frames as usize {
            out_l[i] = self.output_left[i] as f32;
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..frames as usize {
            out_r[i] = self.output_right[i] as f32;
        }

        let mut peaks = [[0u8; 2]; 64];
        for (i, ps) in self.looper_peaks.iter().enumerate() {
            peaks[i][0] = Self::iec_scale(ps[0]);
            peaks[i][1] = Self::iec_scale(ps[1]);
        }

        // Update GUI
        self.gui_sender
            .send_update(GuiCommand::StateSnapshot(EngineStateSnapshot {
                engine_state: self.state,
                time: FrameTime(self.time),
                metric_structure: self.metric_structure,
                active_looper: self.active,
                looper_count: self.loopers.len(),
                part: self.current_part,
                solo,
                sync_mode: self.sync_mode,
                input_levels: Self::compute_peaks(&in_bufs),
                looper_levels: peaks,
                metronome_volume: self
                    .metronome
                    .as_ref()
                    .map(|m| m.get_volume())
                    .unwrap_or(0.0),
            }));
    }
}
