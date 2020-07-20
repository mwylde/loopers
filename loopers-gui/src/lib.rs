mod app;
mod protos;
mod skia;

use skia_safe::{Color, Canvas};

use loopers_common::music::{FrameTime, MetricStructure, TimeSignature, Tempo};
use crossbeam_channel::{TryRecvError};
use crate::app::MainPage;
use loopers_common::gui_channel::{EngineStateSnapshot, GuiCommand, GuiReceiver};
use std::collections::HashMap;


#[allow(unused)]
#[derive(Clone, PartialEq)]
pub enum LoopState {
    Record,
    Overdub,
    Play,
    Stop,
}

impl LoopState {
    fn color(&self) -> Color {
        match self {
            LoopState::Record => Color::from_rgb(255, 0, 0),
            LoopState::Overdub => Color::from_rgb(0, 255, 255),
            LoopState::Play => Color::from_rgb(0, 255, 0),
            LoopState::Stop => Color::from_rgb(135, 135, 135),
        }
    }

    fn dark_color(&self) -> Color {
        match self {
            LoopState::Record => Color::from_rgb(210, 45, 45),
            LoopState::Overdub => Color::from_rgb(0, 255, 255),
            LoopState::Play => Color::from_rgb(0, 213, 0),
            LoopState::Stop => Color::from_rgb(65, 65, 65),
        }
    }
}

type Waveform = [Vec<f32>; 2];

#[derive(Clone)]
pub struct LooperData {
    id: u32,
    length: FrameTime,
    state: LoopState,
    waveform: Waveform,
}

#[derive(Clone)]
pub struct AppData {
    engine_state: EngineStateSnapshot,
    loopers: HashMap<u32, LooperData>,
}

pub struct Gui {
    state: AppData,
    receiver: GuiReceiver,
    initialized: bool,

    root: MainPage,
}

impl Gui {
    pub fn new(receiver: GuiReceiver) -> Gui {
        Gui {
            state: AppData {
                engine_state: EngineStateSnapshot {
                    time: FrameTime(0),
                    metric_structure: MetricStructure {
                        time_signature: TimeSignature {
                            upper: 4,
                            lower: 4,
                        },
                        tempo: Tempo::from_bpm(120.0),
                    },
                    active_looper: 0,
                    looper_count: 0
                },
                loopers: HashMap::new(),
            },
            receiver,

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
                },
                Ok(GuiCommand::AddLooper(id)) => {
                    self.state.loopers.entry(id)
                        .or_insert_with(|| LooperData {
                            id,
                            length: FrameTime(0),
                            state: LoopState::Stop,
                            waveform: [vec![], vec![]],
                        });
                }
                Ok(GuiCommand::RemoveLooper(id)) => {
                    self.state.loopers.remove(&id);
                }
                Ok(GuiCommand::LooperStateChange(_id, _state)) => {

                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Channel disconnected");
                },
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas) {
        if self.initialized {
            self.root.draw(canvas, &self.state);
        }
    }
}


