mod app;
mod protos;
mod skia;

use crate::protos::SavedSession;
use bytes::Bytes;
use hound::SampleFormat;
use prost::Message;
use skia_safe::Color;
use std::fs::File;
use std::io::Read;

const SAMPLE_RATE: f32 = 44.100;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub fn from_ms(ms: f64) -> FrameTime {
        FrameTime((ms * SAMPLE_RATE as f64) as i64)
    }

    pub fn to_ms(&self) -> f64 {
        (self.0 as f64) / SAMPLE_RATE as f64
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeSignature {
    pub upper: u8,
    pub lower: u8,
}

impl TimeSignature {
    pub fn new(upper: u8, lower: u8) -> Option<TimeSignature> {
        if lower == 0 || (lower & (lower - 1)) != 0 {
            // lower must be a power of 2
            return None;
        }
        Some(TimeSignature { upper, lower })
    }

    // converts from (possibly negative) beat-of-song to always positive beat-of-measure
    pub fn beat_of_measure(&self, beat: i64) -> u8 {
        beat.rem_euclid(self.upper as i64) as u8
    }

    pub fn measure(&self, beat: i64) -> i64 {
        beat / self.lower as i64
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tempo {
    pub mbpm: u64,
}

impl Tempo {
    pub fn from_bpm(bpm: f32) -> Tempo {
        Tempo {
            mbpm: (bpm * 1000f32).round() as u64,
        }
    }

    pub fn bpm(&self) -> f32 {
        (self.mbpm as f32) / 1000f32
    }

    pub fn beat(&self, time: FrameTime) -> i64 {
        let bps = self.bpm() / 60.0;
        let mspb = 1000.0 / bps;
        ((time.to_ms() as f32) / mspb).floor() as i64
    }
}

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
    time: FrameTime,
    loopers: Vec<LooperData>,

    time_signature: TimeSignature,
    tempo: Tempo,
}

fn load_waveform(path: &str) -> [Vec<f32>; 2] {
    let mut reader = hound::WavReader::open(path).unwrap();
    let bits_per_sample = reader.spec().bits_per_sample;
    println!(
        "{} = {:?} / len = {}",
        path,
        reader.spec(),
        reader.len() / reader.spec().channels as u32
    );

    let mut r = [Vec::new(), Vec::new()];

    match reader.spec().sample_format {
        SampleFormat::Float => {
            for (i, s) in reader.samples().enumerate() {
                if i % 2 == 0 {
                    r[0].push(s.unwrap())
                } else {
                    r[1].push(s.unwrap())
                }
            }
        }
        SampleFormat::Int => {
            for (i, s) in reader.samples().enumerate() {
                let x: i32 = s.unwrap();

                let max = if bits_per_sample == 16 {
                    i16::max_value() as f32
                } else {
                    i32::max_value() as f32
                };

                let f = x as f32 / max;

                if i % 2 == 0 {
                    r[0].push(f)
                } else {
                    r[1].push(f)
                }
            }
        }
    }
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut file = File::open(&format!("{}/project.loopers", &args[1])).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let buf = Bytes::from(buf);

    let session: SavedSession = Message::decode(buf).unwrap();

    let loopers: Vec<LooperData> = session
        .loopers
        .iter()
        .filter(|l| !l.samples.is_empty())
        .enumerate()
        .map(|(i, l)| {
            let waveform = load_waveform(&l.samples[0]);
            LooperData {
                length: FrameTime(waveform[0].len() as i64),
                state: match i % 4 {
                    0 => LoopState::Record,
                    1 => LoopState::Play,
                    2 => LoopState::Overdub,
                    3 => LoopState::Stop,
                    _ => unreachable!(),
                },
                waveform,
            }
        })
        .collect();

    let data = AppData {
        time: FrameTime(0),
        time_signature: TimeSignature::new(
            session.time_signature_upper as u8,
            session.time_signature_lower as u8,
        )
        .unwrap(),
        tempo: Tempo {
            mbpm: session.tempo_mbpm,
        },
        loopers,
    };

    skia::skia_main(data);
}
