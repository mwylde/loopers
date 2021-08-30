use crate::gui_channel::WAVEFORM_DOWNSAMPLE;
use crate::music::MetricStructure;
use derive_more::{Add, Div, Mul, Sub};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            Command::Start,
            Command::from_str("Start", &[][..]).unwrap()(CommandData { data: 0 })
        );

        assert_eq!(
            Command::SetTime(FrameTime(100)),
            Command::from_str("SetTime", &["100"][..]).unwrap()(CommandData { data: 0 })
        );

        assert_eq!(
            Command::Looper(LooperCommand::Record, LooperTarget::All),
            Command::from_str("Record", &["All"][..]).unwrap()(CommandData { data: 0 })
        );

        assert_eq!(
            Command::Looper(LooperCommand::Overdub, LooperTarget::Selected),
            Command::from_str("Overdub", &["Selected"][..]).unwrap()(CommandData { data: 0 })
        );

        assert_eq!(
            Command::Looper(LooperCommand::Mute, LooperTarget::Index(13)),
            Command::from_str("Mute", &["13"][..]).unwrap()(CommandData { data: 0 })
        );
    }
}

static SAMPLE_RATE: AtomicUsize = AtomicUsize::new(44100);

pub fn set_sample_rate(sample_rate: usize) {
    SAMPLE_RATE.store(sample_rate, Ordering::SeqCst);
}

pub fn get_sample_rate() -> usize {
    SAMPLE_RATE.load(Ordering::SeqCst)
}

pub fn get_sample_rate_ms() -> f64 {
    get_sample_rate() as f64 / 1000.0
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Add,
    Mul,
    Sub,
    Div,
)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub fn from_ms(ms: f64) -> FrameTime {
        FrameTime((ms * get_sample_rate_ms()) as i64)
    }

    pub fn to_ms(&self) -> f64 {
        (self.0 as f64) / get_sample_rate_ms()
    }

    pub fn to_waveform(&self) -> i64 {
        self.0 / WAVEFORM_DOWNSAMPLE as i64
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandData {
    pub data: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperTarget {
    Id(u32),
    Index(u8),
    All,
    Selected,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum LooperCommand {
    // Basic commands
    Record,
    Overdub,
    Play,
    Mute,
    Solo,
    Clear,

    SetSpeed(LooperSpeed),

    // [-1.0, 1.0]
    SetPan(f32),

    // [0.0, 1.0]
    SetLevel(f32),

    // Composite commands
    RecordOverdubPlay,

    // not currently usable from midi
    AddToPart(Part),
    RemoveFromPart(Part),

    // delete
    Delete,

    Undo,
    Redo,
}

impl LooperCommand {
    pub fn from_str(
        command: &str,
        args: &[&str],
    ) -> Result<Box<dyn Fn(CommandData) -> Command + Send>, String> {
        use Command::Looper;
        use LooperCommand::*;

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

        Ok(match command {
            "Record" => Box::new(move |_| Looper(Record, target)),
            "Overdub" => Box::new(move |_| Looper(Overdub, target)),
            "Play" => Box::new(move |_| Looper(Play, target)),
            "Mute" => Box::new(move |_| Looper(Mute, target)),
            "Solo" => Box::new(move |_| Looper(Solo, target)),
            "RecordOverdubPlay" => Box::new(move |_| Looper(RecordOverdubPlay, target)),
            "Delete" => Box::new(move |_| Looper(Delete, target)),
            "Clear" => Box::new(move |_| Looper(Clear, target)),

            "SetPan" => {
                let v = args.get(1).ok_or(
                    "SetPan expects a target and a pan value between -1 and 1".to_string(),
                )?;

                let arg = if *v == "$data" {
                    None
                } else {
                    let f = f32::from_str(v)
                        .map_err(|_| format!("Invalid value for SetPan: '{}'", v))?;
                    if f < -1.0 || f > 1.0 {
                        return Err("Value for SetPan must be between -1 and 1".to_string());
                    }
                    Some(f)
                };

                Box::new(move |d| {
                    Looper(
                        SetPan(arg.unwrap_or(d.data as f32 / 127.0 * 2.0 - 1.0)),
                        target,
                    )
                })
            },

            "SetLevel" => {
                let v = args.get(1).ok_or(
                    "SetLevel expects a target and a level value between 0 and 1".to_string(),
                )?;

                let arg = if *v == "$data" {
                    None
                } else {
                    let f = f32::from_str(v)
                        .map_err(|_| format!("Invalid value for SetLevel: '{}'", v))?;
                    if f < 0.0 || f > 1.0 {
                        return Err("Value for SetLevel must be between 0 and 1".to_string());
                    }
                    Some(f)
                };

                Box::new(move |d| {
                    Looper(
                        SetLevel(arg.unwrap_or(d.data as f32 / 127.0)),
                        target,
                    )
                })
            }


            "1/2x" => Box::new(move |_| Looper(SetSpeed(LooperSpeed::Half), target)),
            "1x" => Box::new(move |_| Looper(SetSpeed(LooperSpeed::One), target)),
            "2x" => Box::new(move |_| Looper(SetSpeed(LooperSpeed::Double), target)),

            _ => return Err(format!("{} is not a valid command", command)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Looper(LooperCommand, LooperTarget),

    Start,
    Stop,
    Pause,

    StartStop,
    PlayPause,

    Reset,
    SetTime(FrameTime),

    AddLooper,
    SelectLooperById(u32),
    SelectLooperByIndex(u8),

    SelectPreviousLooper,
    SelectNextLooper,

    PreviousPart,
    NextPart,
    GoToPart(Part),

    SetQuantizationMode(QuantizationMode),

    SaveSession(Arc<PathBuf>),
    LoadSession(Arc<PathBuf>),

    SetMetronomeLevel(u8),

    SetTempoBPM(f32),
    SetTimeSignature(u8, u8),

    Undo,
    Redo,
}

impl Command {
    pub fn from_str(
        command: &str,
        args: &[&str],
    ) -> Result<Box<dyn Fn(CommandData) -> Command + Send>, String> {
        Ok(match command {
            "Start" => Box::new(|_| Command::Start),
            "Stop" => Box::new(|_| Command::Stop),
            "Pause" => Box::new(|_| Command::Pause),
            "StartStop" => Box::new(|_| Command::StartStop),
            "PlayPause" => Box::new(|_| Command::PlayPause),
            "Reset" => Box::new(|_| Command::Reset),

            "SetTime" => {
                let arg = args
                    .get(0)
                    .and_then(|s| i64::from_str(s).ok())
                    .map(|t| FrameTime(t))
                    .ok_or("SetTime expects a single numeric argument, time".to_string())?;
                Box::new(move |_| Command::SetTime(arg))
            }

            "AddLooper" => Box::new(|_| Command::AddLooper),
            "SelectLooperById" => {
                let arg = args
                    .get(0)
                    .and_then(|s| u32::from_str(s).ok())
                    .ok_or(
                        "SelectLooperById expects a single numeric argument, the looper id"
                            .to_string(),
                    )?
                    .to_owned();

                Box::new(move |_| Command::SelectLooperById(arg))
            }

            "SelectLooperByIndex" => {
                let arg = args.get(0).and_then(|s| u8::from_str(s).ok()).ok_or(
                    "SelectLooperByIndex expects a single numeric argument, the looper index"
                        .to_string(),
                )?;
                Box::new(move |_| Command::SelectLooperByIndex(arg))
            }

            "SelectPreviousLooper" => Box::new(|_| Command::SelectPreviousLooper),
            "SelectNextLooper" => Box::new(|_| Command::SelectNextLooper),

            "PreviousPart" => Box::new(|_| Command::PreviousPart),
            "NextPart" => Box::new(|_| Command::NextPart),
            "GoToPart" => {
                let arg = args
                    .get(0)
                    .and_then(|s| match s.as_ref() {
                        "A" => Some(Part::A),
                        "B" => Some(Part::B),
                        "C" => Some(Part::C),
                        "D" => Some(Part::D),
                        _ => None,
                    })
                    .ok_or("GoToPart expects a part name (one of A, B, C, or D)".to_string())?;

                Box::new(move |_| Command::GoToPart(arg))
            }

            "SetQuantizationMode" => {
                let arg = args
                    .get(0)
                    .and_then(|s| match s.as_ref() {
                        "Free" => Some(QuantizationMode::Free),
                        "Beat" => Some(QuantizationMode::Beat),
                        "Measure" => Some(QuantizationMode::Measure),
                        _ => None,
                    })
                    .ok_or(
                        "SetQuantizationMode expects a sync mode (one of Free, Beat, or Measure)"
                            .to_string(),
                    )?;
                Box::new(move |_| Command::SetQuantizationMode(arg))
            }

            "SetMetronomeLevel" => {
                let arg = args.get(0).and_then(|s| u8::from_str(s).ok()).ok_or(
                    "SetMetronomeLevel expects a single numeric argument, the level between 0-100"
                        .to_string(),
                )?;
                Box::new(move |_| Command::SetMetronomeLevel(arg))
            }

            _ => {
                return LooperCommand::from_str(command, args);
            }
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Hash)]
pub enum Part {
    A,
    B,
    C,
    D,
}

impl Part {
    pub fn name(&self) -> &'static str {
        match self {
            Part::A => "A",
            Part::B => "B",
            Part::C => "C",
            Part::D => "D",
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PartSet {
    a: bool,
    b: bool,
    c: bool,
    d: bool,
}

impl PartSet {
    pub fn new() -> PartSet {
        PartSet {
            a: true,
            b: false,
            c: false,
            d: false,
        }
    }

    pub fn with(part: Part) -> PartSet {
        let mut parts = PartSet {
            a: false,
            b: false,
            c: false,
            d: false,
        };
        parts[part] = true;
        parts
    }

    pub fn is_empty(&self) -> bool {
        !(self.a || self.b || self.c || self.c)
    }
}

impl Default for PartSet {
    fn default() -> Self {
        PartSet::new()
    }
}

impl Index<Part> for PartSet {
    type Output = bool;

    fn index(&self, index: Part) -> &Self::Output {
        match index {
            Part::A => &self.a,
            Part::B => &self.b,
            Part::C => &self.c,
            Part::D => &self.d,
        }
    }
}

impl IndexMut<Part> for PartSet {
    fn index_mut(&mut self, index: Part) -> &mut Self::Output {
        match index {
            Part::A => &mut self.a,
            Part::B => &mut self.b,
            Part::C => &mut self.c,
            Part::D => &mut self.d,
        }
    }
}

pub static PARTS: [Part; 4] = [Part::A, Part::B, Part::C, Part::D];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperMode {
    Recording,
    Overdubbing,
    Muted,
    Playing,
    Soloed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LooperSpeed {
    Half,
    One,
    Double,
}

fn looper_speed_default() -> LooperSpeed {
    LooperSpeed::One
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QuantizationMode {
    Free,
    Beat,
    Measure,
}

fn sync_mode_default() -> QuantizationMode {
    QuantizationMode::Measure
}

fn level_default() -> f32 {
    1.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedLooper {
    pub id: u32,
    pub mode: LooperMode,
    #[serde(default = "looper_speed_default")]
    pub speed: LooperSpeed,
    #[serde(default)]
    pub pan: f32,
    #[serde(default = "level_default")]
    pub level: f32,
    #[serde(default)]
    pub parts: PartSet,
    pub samples: Vec<PathBuf>,
    #[serde(default)]
    pub offset_samples: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedSession {
    pub save_time: i64,
    #[serde(default)]
    pub metronome_volume: u8,
    pub metric_structure: MetricStructure,
    #[serde(default = "sync_mode_default")]
    pub sync_mode: QuantizationMode,
    #[serde(default)]
    pub sample_rate: usize,
    pub loopers: Vec<SavedLooper>,
}
