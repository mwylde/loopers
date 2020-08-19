use crate::api::{FrameTime, LooperCommand, LooperMode};
use crate::music::MetricStructure;
use arrayvec::ArrayVec;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::io::{Write, ErrorKind};
use std::borrow::Cow;
use std::io;

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
    pub metronome_volume: f32,
}

pub type Waveform = [Vec<f32>; 2];

#[derive(Clone, Debug)]
pub enum GuiCommand {
    StateSnapshot(EngineStateSnapshot),
    AddLooper(u32),
    AddLooperWithSamples(u32, u64, Box<Waveform>),
    RemoveLooper(u32),
    ClearLooper(u32),

    LooperStateChange(u32, LooperMode),

    AddNewSample(u32, FrameTime, [f32; 2], u64),
    AddOverdubSample(u32, FrameTime, [f32; 2]),
    SetLoopLength(u32, u64),

    AddTrigger(u32, FrameTime, LooperCommand),
}

#[derive(Clone)]
pub struct LogMessage {
    buffer: ArrayVec<[u8; 256]>,
    len: usize,
}

impl LogMessage {
    pub fn new() -> Self {
        LogMessage {
            buffer: ArrayVec::new(),
            len: 0,
        }
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.buffer[0..self.len])
    }
}

pub struct GuiSender {
    cmd_channel: Option<Sender<GuiCommand>>,
    cur_message: LogMessage,
    log_channel: Option<Sender<LogMessage>>,
}

pub struct GuiReceiver {
    pub cmd_channel: Receiver<GuiCommand>,
    pub log_channel: Receiver<LogMessage>,
}

impl GuiSender {
    pub fn new() -> (GuiSender, GuiReceiver) {
        let (tx, rx) = bounded(100);
        let (log_tx, log_rx) = bounded(10);

        let sender = GuiSender {
            cmd_channel: Some(tx),
            cur_message: LogMessage::new(),
            log_channel: Some(log_tx),
        };

        let receiver = GuiReceiver {
            cmd_channel: rx,
            log_channel: log_rx,
        };

        (sender, receiver)
    }

    pub fn disconnected() -> GuiSender {
        GuiSender {
            cmd_channel: None,
            cur_message: LogMessage::new(),
            log_channel: None,
        }
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
            cur_message: self.cur_message.clone(),
            log_channel: self.log_channel.clone(),
        }
    }
}

impl Write for GuiSender {
    fn write(&mut self, s: &[u8]) -> io::Result<usize> {
        self.cur_message.len += s.len();
        self.cur_message.buffer.write(s)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(log_channel) = &self.log_channel {
            let message = self.cur_message.clone();
            self.cur_message.len = 0;
            self.cur_message.buffer.clear();
            log_channel.try_send(message).map_err(|e| {
                match e {
                    TrySendError::Full(_) => {
                        io::Error::new(ErrorKind::WouldBlock, "queue full")
                    },
                    TrySendError::Disconnected(_) => {
                        io::Error::new(ErrorKind::BrokenPipe, "queue disconnected")
                    }
                }
            })?;
        }
        Ok(())
    }
}
