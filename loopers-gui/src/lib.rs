#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod app;
mod skia;
mod widgets;

use skia_safe::{Canvas, Size};

use crossbeam_channel::{Sender, TryRecvError};
use glutin::dpi::PhysicalPosition;
use loopers_common::gui_channel::{
    EngineState, EngineStateSnapshot, GuiCommand, GuiReceiver, Waveform, WAVEFORM_DOWNSAMPLE,
};
use loopers_common::music::{MetricStructure, Tempo, TimeSignature};
use std::collections::HashMap;
use winit::event::MouseButton;

use crate::app::MainPage;
use loopers_common::api::{Command, FrameTime, LooperMode, LooperCommand};

const SHOW_BUTTONS: bool = true;

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
    Backspace,
    Enter,
    Esc,
}

#[derive(Copy, Clone, Debug)]
pub enum GuiEvent {
    MouseEvent(MouseEventType, PhysicalPosition<f64>),
    KeyEvent(KeyEventType, KeyEventKey),
}

#[derive(Clone)]
pub struct LooperData {
    id: u32,
    length: u64,
    last_time: FrameTime,
    state: LooperMode,
    waveform: Waveform,
    trigger: Option<(FrameTime, LooperCommand)>
}


impl LooperData {
    fn mode_with_solo(&self, data: &AppData) -> LooperMode {
        let solo = data.loopers.iter().any(|(_, l)| l.state == LooperMode::Soloed);
        if solo && self.state != LooperMode::Soloed {
            LooperMode::Muted
        } else {
            self.state
        }
    }
}
#[derive(Clone)]
pub struct AppData {
    engine_state: EngineStateSnapshot,
    loopers: HashMap<u32, LooperData>,
    show_buttons: bool,
}

pub struct Gui {
    state: AppData,
    receiver: GuiReceiver,
    sender: Sender<Command>,
    initialized: bool,

    root: MainPage,
}

impl Gui {
    pub fn new(receiver: GuiReceiver, sender: Sender<Command>) -> Gui {
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
                    input_levels: [0.0, 0.0],
                    metronome_volume: 1.0,
                },
                loopers: HashMap::new(),
                show_buttons: SHOW_BUTTONS,
            },
            receiver,

            sender,

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
                Ok(GuiCommand::AddLooper(id)) => {
                    self.state.loopers.insert(
                        id,
                        LooperData {
                            id,
                            length: 0,
                            last_time: FrameTime(0),
                            state: LooperMode::Playing,
                            waveform: [vec![], vec![]],
                            trigger: None,
                        },
                    );
                }
                Ok(GuiCommand::AddLooperWithSamples(id, length, waveform)) => {
                    self.state.loopers.insert(
                        id,
                        LooperData {
                            id,
                            length,
                            last_time: FrameTime(length as i64 - 1),
                            state: LooperMode::Playing,
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
                Ok(GuiCommand::LooperStateChange(id, mode)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.state = mode;
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
                Ok(GuiCommand::SetLoopLength(id, len)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.length = len;
                    }
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Channel disconnected");
                }
                Ok(GuiCommand::AddTrigger(id, time, command)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.trigger = Some((time, command))
                    }
                }
            }
        }
    }

    pub fn min_size(&self) -> Size {
        self.root.min_size(&self.state)
    }

    pub fn draw(&mut self, canvas: &mut Canvas, w: f32, h: f32, last_event: Option<GuiEvent>) {
        if self.initialized {
            self.root
                .draw(canvas, &self.state, w, h, &mut self.sender, last_event);
        }
    }
}
