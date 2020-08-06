use crate::gui_channel::WAVEFORM_DOWNSAMPLE;
use crate::music::MetricStructure;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use derive_more::{Add, Mul, Sub, Div};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Ok(Command::Start), Command::from_str("Start", &[][..]));

        assert_eq!(
            Ok(Command::SetTime(FrameTime(100))),
            Command::from_str("SetTime", &["100"][..])
        );

        assert_eq!(
            Ok(Command::Looper(LooperCommand::Record, LooperTarget::All)),
            Command::from_str("Record", &["All"][..])
        );

        assert_eq!(
            Ok(Command::Looper(
                LooperCommand::Overdub,
                LooperTarget::Selected
            )),
            Command::from_str("Overdub", &["Selected"][..])
        );

        assert_eq!(
            Ok(Command::Looper(
                LooperCommand::Mute,
                LooperTarget::Index(13)
            )),
            Command::from_str("Mute", &["13"][..])
        );
    }
}

pub const SAMPLE_RATE: f64 = 44.100;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd,
         Add, Mul, Sub, Div)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub fn from_ms(ms: f64) -> FrameTime {
        FrameTime((ms * SAMPLE_RATE) as i64)
    }

    pub fn to_ms(&self) -> f64 {
        (self.0 as f64) / SAMPLE_RATE
    }

    pub fn to_waveform(&self) -> i64 {
        self.0 / WAVEFORM_DOWNSAMPLE as i64
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperTarget {
    Id(u32),
    Index(u8),
    All,
    Selected,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperCommand {
    // Basic commands
    Record,
    Overdub,
    Play,
    Mute,

    // Composite commands
    RecordOverdubPlay,

    // delete
    Delete,
}

impl LooperCommand {
    pub fn from_str(command: &str) -> Option<LooperCommand> {
        use LooperCommand::*;
        match command {
            "Record" => Some(Record),
            "Overdub" => Some(Overdub),
            "Play" => Some(Play),
            "Mute" => Some(Mute),
            "RecordOverdubPlay" => Some(RecordOverdubPlay),
            "Delete" => Some(Delete),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    Looper(LooperCommand, LooperTarget),

    Start,
    Stop,

    StartStop,

    Reset,
    SetTime(FrameTime),

    AddLooper,
    SelectLooperById(u32),
    SelectLooperByIndex(u8),

    SaveSession(Arc<PathBuf>),
    LoadSession(Arc<PathBuf>),

    SetMetronomeLevel(u8),

    SetTempoBPM(f32),
    SetTimeSignature(u8, u8),
}

impl Command {
    pub fn from_str(command: &str, args: &[&str]) -> Result<Command, String> {
        match command {
            "Start" => Ok(Command::Start),
            "Stop" => Ok(Command::Stop),
            "StartStop" => Ok(Command::StartStop),
            "Reset" => Ok(Command::Reset),

            "SetTime" => args
                .get(0)
                .and_then(|s| i64::from_str(s).ok())
                .map(|t| Command::SetTime(FrameTime(t)))
                .ok_or("SetTime expects a single numeric argument, time".to_string()),

            "AddLooper" => Ok(Command::AddLooper),
            "SelectLooperById" => args
                .get(0)
                .and_then(|s| u32::from_str(s).ok())
                .map(|id| Command::SelectLooperById(id))
                .ok_or(
                    "SelectLooperById expects a single numeric argument, the looper id".to_string(),
                ),

            "SelectLooperByIndex" => args
                .get(0)
                .and_then(|s| u8::from_str(s).ok())
                .map(|id| Command::SelectLooperByIndex(id))
                .ok_or(
                    "SelectLooperByIndex expects a single numeric argument, the looper index"
                        .to_string(),
                ),

            "SetMetronomeLevel" => args
                .get(0)
                .and_then(|s| u8::from_str(s).ok())
                .map(|v| Command::SetMetronomeLevel(v))
                .ok_or(
                    "SetTime expects a single numeric argument, the level between 0-100"
                        .to_string(),
                ),

            _ => {
                let lc = LooperCommand::from_str(command)
                    .ok_or(format!("{} is not a valid command", command))?;

                let target_type = args.get(0).ok_or(format!("{} expects a target", command))?;

                let target = match *target_type {
                    "All" => LooperTarget::All,
                    "Selected" => LooperTarget::Selected,
                    i => LooperTarget::Index(u8::from_str(i).map_err(|_| {
                        format!(
                            "{} expects a target (All, Selected, or a looper index)",
                            command
                        )
                    })?),
                };

                Ok(Command::Looper(lc, target))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperMode {
    Recording,
    Overdubbing,
    Muted,
    Playing,
    Soloed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedLooper {
    pub id: u32,
    pub mode: LooperMode,
    pub samples: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedSession {
    pub save_time: i64,
    #[serde(default)]
    pub metronome_volume: u8,
    pub metric_structure: MetricStructure,
    pub loopers: Vec<SavedLooper>,
}
