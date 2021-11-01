use crate::looper;
use crate::looper::Looper;
use crate::{last_session_path, MetricStructure};
use chrono::Local;
use crossbeam_channel::{bounded, Sender, TrySendError};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::SaveLoadError;
use loopers_common::api::{QuantizationMode, SavedSession};
use loopers_common::gui_channel::{GuiSender, LogMessage};
use std::sync::Arc;

const LOOPER_SAVE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct SaveSessionData {
    pub metric_structure: MetricStructure,
    pub metronome_volume: u8,
    pub sync_mode: QuantizationMode,
    pub path: Arc<PathBuf>,
    pub sample_rate: usize,
}

pub enum SessionCommand {
    SaveSession(SaveSessionData),
    AddLooper(u32, Sender<looper::ControlMessage>),
    RemoveLooper(u32),
}

#[derive(Clone)]
pub struct SessionSaver {
    channel: Sender<SessionCommand>,
}

impl SessionSaver {
    pub fn new(mut gui_channel: GuiSender) -> SessionSaver {
        let (tx, rx) = bounded(10);

        thread::spawn(move || {
            let mut loopers: HashMap<u32, Sender<looper::ControlMessage>> = HashMap::new();

            loop {
                match rx.recv() {
                    Ok(SessionCommand::SaveSession(sd)) => {
                        if let Err(e) = Self::execute_save_session(sd, &loopers, &mut gui_channel) {
                            let mut log = LogMessage::error();
                            if let Err(e) = write!(log, "Failed to save session: {:?}", e) {
                                error!("Failed to write error message: {}", e);
                            } else {
                                gui_channel.send_log(log);
                            }
                        }
                    }
                    Ok(SessionCommand::AddLooper(id, tx)) => {
                        loopers.insert(id, tx);
                    }
                    Ok(SessionCommand::RemoveLooper(id)) => {
                        loopers.remove(&id);
                    }
                    Err(_) => {
                        debug!("channel closed, stopping");
                        break;
                    }
                }
            }
        });

        SessionSaver { channel: tx }
    }

    fn execute_save_session(
        sd: SaveSessionData,
        loopers: &HashMap<u32, Sender<looper::ControlMessage>>,
        gui_channel: &mut GuiSender,
    ) -> Result<(), SaveLoadError> {
        let now = Local::now();
        let mut path = (&*sd.path).clone();
        path.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());

        create_dir_all(&path)?;

        let mut session = SavedSession {
            save_time: now.timestamp_millis(),
            metric_structure: sd.metric_structure.to_saved(),
            metronome_volume: sd.metronome_volume,
            sync_mode: sd.sync_mode,
            sample_rate: sd.sample_rate,
            loopers: Vec::with_capacity(loopers.len()),
        };

        let mut channels = vec![];
        for (id, l) in loopers.iter() {
            let (tx, rx) = bounded(1);

            l.send(looper::ControlMessage::Serialize(path.clone(), tx))
                .map_err(|_f| SaveLoadError::LooperSaveError(*id))?;

            channels.push((*id, rx));
        }

        let start = Instant::now();
        let mut timeout = LOOPER_SAVE_TIMEOUT;
        for (id, c) in channels {
            session.loopers.push(
                c.recv_timeout(timeout)
                    .map_err(|_| SaveLoadError::LooperSaveError(id))??,
            );

            timeout = timeout
                .checked_sub(start.elapsed())
                .ok_or(SaveLoadError::LooperTimeoutError)?;
        }

        path.push("project.loopers");
        let mut file = File::create(&path)?;

        match serde_json::to_string_pretty(&session) {
            Ok(v) => {
                writeln!(file, "{}", v)?;

                // save our last session
                let config_path = last_session_path()?;
                let mut last_session = File::create(config_path)?;
                write!(last_session, "{}", path.to_string_lossy())?;
            }
            Err(e) => {
                return Err(SaveLoadError::OtherError(format!(
                    "Failed to serialize session: {:?}",
                    e
                )));
            }
        }

        if let Err(_) = write!(gui_channel, "Session saved to {}", path.to_string_lossy())
            .and_then(|_| gui_channel.flush())
        {
            warn!("failed to write gui message");
        }

        Ok(())
    }

    pub fn add_looper(&mut self, looper: &Looper) {
        self.channel
            .send(SessionCommand::AddLooper(looper.id, looper.channel()))
            .expect("channel closed!");
    }

    pub fn remove_looper(&mut self, id: u32) {
        self.channel
            .send(SessionCommand::RemoveLooper(id))
            .expect("channel closed");
    }

    pub fn save_session(&mut self, data: SaveSessionData) -> Result<(), SaveLoadError> {
        self.channel
            .try_send(SessionCommand::SaveSession(data))
            .map_err(|err| match err {
                TrySendError::Full(_) => SaveLoadError::ChannelFull,
                TrySendError::Disconnected(_) => SaveLoadError::ChannelClosed,
            })
    }
}
