use crate::engine::{last_session_path, MetricStructure};
use crate::error::SaveLoadError;
use crate::looper;
use crate::looper::Looper;
use crate::prost::Message;
use crate::protos::SavedSession;
use bytes::BytesMut;
use chrono::Local;
use crossbeam_channel::{bounded, Sender, TrySendError};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

const LOOPER_SAVE_TIMEOUT: Duration = Duration::from_secs(10);

pub enum Command {
    SaveSession(MetricStructure, PathBuf),
    AddLooper(u32, Sender<looper::ControlMessage>),
    RemoveLooper(u32),
}

#[derive(Clone)]
pub struct SessionSaver {
    channel: Sender<Command>,
}

impl SessionSaver {
    pub fn new() -> SessionSaver {
        let (tx, rx) = bounded(10);

        thread::spawn(move || {
            loop {
                let mut loopers: HashMap<u32, Sender<looper::ControlMessage>> = HashMap::new();
                match rx.recv() {
                    Ok(Command::SaveSession(ms, path)) => {
                        Self::execute_save_session(ms, path, &loopers)
                            // TODO: handle this properly
                            .unwrap();
                    }
                    Ok(Command::AddLooper(id, tx)) => {
                        loopers.insert(id, tx);
                    }
                    Ok(Command::RemoveLooper(id)) => {
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
        metric_structure: MetricStructure,
        path: PathBuf,
        loopers: &HashMap<u32, Sender<looper::ControlMessage>>,
    ) -> Result<(), SaveLoadError> {
        let now = Local::now();
        let mut path = PathBuf::from(&path);
        path.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());

        create_dir_all(&path)?;

        let mut session = SavedSession {
            save_time: now.timestamp_millis(),
            time_signature_upper: metric_structure.time_signature.upper as u64,
            time_signature_lower: metric_structure.time_signature.lower as u64,
            tempo_mbpm: metric_structure.tempo.mbpm,
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

        let mut buf = BytesMut::with_capacity(session.encoded_len());
        session.encode(&mut buf)?;
        file.write_all(&buf)?;

        // save our last session
        let config_path = last_session_path()?;
        let mut last_session = File::create(config_path)?;
        write!(last_session, "{}", path.to_string_lossy())?;

        Ok(())
    }

    pub fn add_looper(&mut self, looper: &Looper) {
        self.channel
            .send(Command::AddLooper(looper.id, looper.channel()))
            .expect("channel closed!");
    }

    pub fn remove_looper(&mut self, id: u32) {
        self.channel
            .send(Command::RemoveLooper(id))
            .expect("channel closed");
    }

    pub fn save_session(
        &mut self,
        metric_structure: MetricStructure,
        path: PathBuf,
    ) -> Result<(), SaveLoadError> {
        self.channel
            .try_send(Command::SaveSession(metric_structure, path))
            .map_err(|err| match err {
                TrySendError::Full(_) => SaveLoadError::ChannelFull,
                TrySendError::Disconnected(_) => SaveLoadError::ChannelClosed,
            })
    }
}
