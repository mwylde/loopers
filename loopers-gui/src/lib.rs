#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod app;
mod skia;
mod widgets;

use skia_safe::{Canvas, Size};

use crate::app::MainPage;
use crossbeam_channel::{Sender, TryRecvError, TrySendError};
use loopers_common::api::{
    Command, FrameTime, LooperCommand, LooperMode, LooperSpeed, Part, PartSet, SyncMode,
};
use loopers_common::gui_channel::{
    EngineState, EngineStateSnapshot, GuiCommand, GuiReceiver, GuiSender, LogMessage, Waveform,
    WAVEFORM_DOWNSAMPLE,
};
use loopers_common::music::{MetricStructure, Tempo, TimeSignature};
use sdl2::mouse::MouseButton;
use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::time::{Duration, Instant};

const SHOW_BUTTONS: bool = true;

pub const MESSAGE_DISPLAY_TIME_SECS: u64 = 4;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MouseEventType {
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Moved,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyEventType {
    Pressed,
    Released,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyEventKey {
    Char(char),
    Esc,
    Backspace,
    Enter,
}

#[derive(Copy, Clone, Debug)]
pub enum GuiEvent {
    MouseEvent(MouseEventType, (i32, i32)),
    KeyEvent(KeyEventType, KeyEventKey),
}

#[derive(Clone)]
pub struct LooperData {
    id: u32,
    length: u64,
    offset: FrameTime,
    last_time: FrameTime,
    mode: LooperMode,
    parts: PartSet,
    speed: LooperSpeed,
    waveform: Waveform,
    trigger: Option<(FrameTime, LooperCommand)>,
}

impl LooperData {
    fn mode_with_solo(&self, data: &AppData) -> LooperMode {
        let solo = data
            .loopers
            .iter()
            .any(|(_, l)| l.mode == LooperMode::Soloed);
        if solo && self.mode != LooperMode::Soloed {
            LooperMode::Muted
        } else {
            self.mode
        }
    }
}

#[derive(Clone)]
pub struct Log {
    cur: Option<(Instant, LogMessage)>,
    queue: VecDeque<LogMessage>,
}

impl Log {
    fn new() -> Self {
        Log {
            cur: None,
            queue: VecDeque::new(),
        }
    }

    fn add(&mut self, message: LogMessage) {
        if self.cur.is_none() {
            self.cur = Some((Instant::now(), message));
        } else {
            self.queue.push_back(message);
        }
    }

    fn update(&mut self) {
        if let Some((t, _)) = &self.cur {
            if Instant::now() - *t > Duration::from_secs(MESSAGE_DISPLAY_TIME_SECS) {
                self.cur = self.queue.pop_front().map(|m| (Instant::now(), m));
            }
        }
    }
}

#[derive(Clone)]
pub struct Controller {
    command_sender: Sender<Command>,
    gui_sender: GuiSender,
}

impl Controller {
    pub fn send_command(&mut self, command: Command, err: &str) {
        match self.command_sender.try_send(command) {
            Ok(_) => {}
            Err(TrySendError::Full(_)) => {
                self.log(err);
            }
            Err(TrySendError::Disconnected(_)) => {
                // TODO: handle these cases better
                panic!("lost connection to engine");
            }
        }
    }

    pub fn log(&mut self, msg: &str) {
        if let Err(_) = write!(self.gui_sender, "{}", msg).and_then(|_| self.gui_sender.flush()) {
            error!("Failed to write message to gui");
        }
    }
}

#[derive(Clone)]
pub struct AppData {
    engine_state: EngineStateSnapshot,
    loopers: HashMap<u32, LooperData>,
    show_buttons: bool,
    messages: Log,
    global_triggers: Vec<(
        FrameTime, // time created
        FrameTime, // trigger time
        Command,
    )>,
}

pub struct Gui {
    state: AppData,
    receiver: GuiReceiver,
    controller: Controller,
    initialized: bool,

    root: MainPage,
}

impl Gui {
    pub fn new(
        receiver: GuiReceiver,
        command_sender: Sender<Command>,
        gui_sender: GuiSender,
    ) -> Gui {
        Gui {
            state: AppData {
                engine_state: EngineStateSnapshot {
                    engine_state: EngineState::Stopped,
                    time: FrameTime(0),
                    metric_structure: MetricStructure {
                        time_signature: TimeSignature { upper: 4, lower: 4 },
                        tempo: Tempo::from_bpm(120.0),
                    },
                    active_looper: 0,
                    looper_count: 0,
                    part: Part::A,
                    sync_mode: SyncMode::Measure,
                    input_levels: [0.0, 0.0],
                    metronome_volume: 1.0,
                },
                loopers: HashMap::new(),
                show_buttons: SHOW_BUTTONS,
                messages: Log::new(),
                global_triggers: Vec::new(),
            },
            receiver,

            controller: Controller {
                command_sender,
                gui_sender,
            },

            initialized: false,
            root: MainPage::new(),
        }
    }

    pub fn start(self) {
        skia::skia_main(self);
    }

    pub fn update(&mut self) {
        loop {
            match self.receiver.cmd_channel.try_recv() {
                Ok(GuiCommand::StateSnapshot(state)) => {
                    self.state.engine_state = state;
                    self.initialized = true;

                    // clear past triggers
                    for l in self.state.loopers.values_mut() {
                        if let Some((time, _)) = l.trigger {
                            if time < state.time {
                                l.trigger = None;
                            }
                        }
                    }
                }
                Ok(GuiCommand::AddLooper(id, state)) => {
                    self.state.loopers.insert(
                        id,
                        LooperData {
                            id,
                            length: 0,
                            offset: FrameTime(0),
                            last_time: FrameTime(0),
                            mode: state.mode,
                            parts: state.parts,
                            speed: state.speed,
                            waveform: [vec![], vec![]],
                            trigger: None,
                        },
                    );
                }
                Ok(GuiCommand::AddLooperWithSamples(id, length, waveform, state)) => {
                    self.state.loopers.insert(
                        id,
                        LooperData {
                            id,
                            length,
                            offset: state.offset,
                            last_time: FrameTime(length as i64 - 1),
                            mode: state.mode,
                            parts: state.parts,
                            speed: state.speed,
                            waveform: *waveform,
                            trigger: None,
                        },
                    );
                }
                Ok(GuiCommand::RemoveLooper(id)) => {
                    self.state.loopers.remove(&id);
                }
                Ok(GuiCommand::ClearLooper(id)) => {
                    if let Some(looper) = self.state.loopers.get_mut(&id) {
                        looper.waveform = [vec![], vec![]];
                        looper.length = 0;
                    }
                }
                Ok(GuiCommand::LooperStateChange(id, state)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.mode = state.mode;
                        l.parts = state.parts;
                        l.speed = state.speed;
                    } else {
                        warn!("Got looper state change for unknown looper {}", id);
                    }
                }
                Ok(GuiCommand::AddNewSample(id, time, sample, new_len)) => {
                    // TODO: use time to ensure we're synced
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.waveform[0].push(sample[0]);
                        l.waveform[1].push(sample[1]);
                        l.length = new_len;
                        l.last_time = time;
                    }
                }
                Ok(GuiCommand::AddOverdubSample(id, time, sample)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        let time = time + l.offset;
                        if time.0 >= 0 && l.waveform[0].len() > 0 && l.length > 0 {
                            let i =
                                ((time.0 as u64 % l.length) / WAVEFORM_DOWNSAMPLE as u64) as usize;
                            if i < l.waveform[0].len() - 1 {
                                l.waveform[0][i] = sample[0];
                                l.waveform[1][i] = sample[1];
                            } else {
                                l.waveform[0].push(sample[0]);
                                l.waveform[1].push(sample[1]);
                            }
                            l.last_time = time;
                        }
                    }
                }
                Ok(GuiCommand::SetLoopLengthAndOffset(id, len, offset)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.length = len;
                        l.offset = offset;
                    }
                }
                Ok(GuiCommand::AddGlobalTrigger(time, command)) => {
                    self.state
                        .global_triggers
                        .push((self.state.engine_state.time, time, command));
                }
                Ok(GuiCommand::AddLoopTrigger(id, time, command)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.trigger = Some((time, command))
                    }
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Channel disconnected");
                }
            }
        }

        // clear out old global triggers
        let time = self.state.engine_state.time;
        self.state.global_triggers.retain(|(_, t, _)| *t > time);

        // clear out old log messages
        self.state.messages.update();

        // read log messages
        match self.receiver.log_channel.try_recv() {
            Ok(log) => {
                self.state.messages.add(log);
            }
            Err(TryRecvError::Empty) => {
                // do nothing
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Channel disconnected");
            }
        }
    }

    pub fn min_size(&self) -> Size {
        self.root.min_size(&self.state)
    }

    pub fn draw(&mut self, canvas: &mut Canvas, w: f32, h: f32, last_event: Option<GuiEvent>) {
        if self.initialized {
            self.root
                .draw(canvas, &self.state, w, h, &mut self.controller, last_event);
        }
    }
}
