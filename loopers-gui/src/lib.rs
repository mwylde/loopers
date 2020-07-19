mod app;
mod protos;
mod skia;





use skia_safe::{Color, Canvas};


use loopers_common::music::{FrameTime};
use crossbeam_channel::{Receiver, bounded, Sender, TryRecvError};
use loopers_common::{EngineStateSnapshot, GuiCommand};

use crate::app::MainPage;


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
    length: FrameTime,
    state: LoopState,
    waveform: Waveform,
}

#[derive(Clone)]
pub struct AppData {
    engine_state: EngineStateSnapshot,
    loopers: Vec<LooperData>,
}

pub struct Gui {
    state: Option<AppData>,
    sender: Sender<GuiCommand>,
    receiver: Receiver<GuiCommand>,

    root: MainPage,
}

impl Gui {
    pub fn new() -> Gui {
        let (tx, rx) = bounded(10);

        Gui {
            state: None,
            sender: tx,
            receiver: rx,

            root: MainPage::new(),
        }
    }

    pub fn get_sender(&self) -> Sender<GuiCommand> {
        self.sender.clone()
    }

    pub fn start(self) {
        skia::skia_main(self);
    }

    pub fn update(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(GuiCommand::StateSnapshot(state)) => {
                    if self.state.is_none() {
                        self.state = Some(AppData {
                            engine_state: state,
                            loopers: vec![],
                        })
                    } else {
                        self.state.as_mut().unwrap().engine_state = state;
                    }
                },
                Ok(GuiCommand::AddLooper(_id)) => {
                   // todo
                }
                Ok(GuiCommand::RemoveLooper(_id)) => {
                    // todo
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
        if let Some(state) = &self.state {
            self.root.draw(canvas, state);
        }
    }
}


