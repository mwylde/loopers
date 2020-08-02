use crate::api::{FrameTime, LooperMode};
use crate::music::MetricStructure;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

pub const WAVEFORM_DOWNSAMPLE: usize = 2048;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EngineState {
    Stopped,
    Active,
}

#[derive(Copy, Clone, Debug)]
pub struct EngineStateSnapshot {
    pub engine_state: EngineState,
    pub time: FrameTime,
    pub metric_structure: MetricStructure,
    pub active_looper: u32,
    pub looper_count: usize,
    pub input_levels: [f32; 2],
}

pub type Waveform = [Vec<f32>; 2];

#[derive(Clone, Debug)]
pub enum GuiCommand {
    StateSnapshot(EngineStateSnapshot),
    AddLooper(u32),
    AddLooperWithSamples(u32, u64, Box<Waveform>),
    RemoveLooper(u32),

    LooperStateChange(u32, LooperMode),

    AddNewSample(u32, FrameTime, [f32; 2], u64),
    AddOverdubSample(u32, FrameTime, [f32; 2]),
    SetLoopLength(u32, u64),
}

pub struct GuiSender {
    cmd_channel: Option<Sender<GuiCommand>>,
}

pub struct GuiReceiver {
    pub cmd_channel: Receiver<GuiCommand>,
}

impl GuiSender {
    pub fn new() -> (GuiSender, GuiReceiver) {
        let (tx, rx) = bounded(100);

        let sender = GuiSender {
            cmd_channel: Some(tx),
        };

        let receiver = GuiReceiver { cmd_channel: rx };

        (sender, receiver)
    }

    pub fn disconnected() -> GuiSender {
        GuiSender { cmd_channel: None }
    }

    pub fn send_update(&mut self, cmd: GuiCommand) {
        if let Some(gui_sender) = &self.cmd_channel {
            match gui_sender.try_send(cmd) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {
                    warn!("GUI message queue is full");
                }
                Err(TrySendError::Disconnected(_)) => {
                    // TODO: think more about the correct behavior here
                    panic!("GUI message queue is disconnected");
                }
            }
        }
    }
}

impl Clone for GuiSender {
    fn clone(&self) -> Self {
        GuiSender {
            cmd_channel: self.cmd_channel.clone(),
        }
    }
}
