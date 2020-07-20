#[macro_use]
extern crate log;

mod app;
mod protos;
mod skia;

use skia_safe::Canvas;

use loopers_common::music::{FrameTime, MetricStructure, TimeSignature, Tempo};
use crossbeam_channel::{TryRecvError};
use crate::app::MainPage;
use loopers_common::gui_channel::{EngineStateSnapshot, GuiCommand, GuiReceiver, Waveform};
use std::collections::HashMap;
use loopers_common::protos::LooperMode;

#[derive(Clone)]
pub struct LooperData {
    id: u32,
    length: u64,
    state: LooperMode,
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
                    self.state.loopers.insert(id, LooperData {
                        id,
                        length: 0,
                        state: LooperMode::None,
                        waveform: [vec![], vec![]],
                    });
                }
                Ok(GuiCommand::AddLooperWithSamples(id, length, waveform)) => {
                    self.state.loopers.insert(id, LooperData {
                        id,
                        length,
                        state: LooperMode::None,
                        waveform: *waveform,
                    });
                }
                Ok(GuiCommand::RemoveLooper(id)) => {
                    self.state.loopers.remove(&id);
                }
                Ok(GuiCommand::LooperStateChange(id, mode)) => {
                    if let Some(l) = self.state.loopers.get_mut(&id) {
                        l.state = mode;
                    } else {
                        warn!("Got looper state change for unknown looper {}", id);
                    }
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


