use crate::sample::Sample;
use crate::protos::{LooperMode, LooperCommand, LooperCommandType};

pub struct Looper {
    pub id: u32,
    pub samples: Vec<Sample>,
    pub mode: LooperMode,
    pub deleted: bool,
}

impl Looper {
    pub fn new(id: u32) -> Looper {
        let looper = Looper {
            id,
            samples: vec![],
            mode: LooperMode::None,
            deleted: false,
        };

        looper
    }
}

impl Looper {
    pub fn transition_to(&mut self, mode: LooperMode) {
        println!("Transition {:?} to {:?}", self.mode, mode);
        match (self.mode, mode) {
            (x, y) if x == y => {},
            (_, LooperMode::None) => {},
            (_, LooperMode::Ready) => {},
            (_, LooperMode::Record) => {
                self.samples.clear();
                self.samples.push(Sample::new(0));
            },
            (_, LooperMode::Overdub) => {
                let len = self.length_in_samples();
                self.samples.push(Sample::new(self.samples.last().unwrap().length() as usize));
                println!("len: {}\t{}", len, self.length_in_samples());
            },
            (_, LooperMode::Playing) => {},
            (_, LooperMode::Stopping) => {},
        }

        self.mode = mode;
    }

    pub fn process_event(&mut self, looper_event: &LooperCommand) {
        if let Some(typ) = LooperCommandType::from_i32(looper_event.command_type) {
            match typ as LooperCommandType {
                LooperCommandType::EnableReady => {
                    self.transition_to(LooperMode::Ready);
                }
                LooperCommandType::EnableRecord => {
                    self.transition_to(LooperMode::Record);
                },
                LooperCommandType::EnableOverdub => {
                    self.transition_to(LooperMode::Overdub);
                },
                LooperCommandType::EnableMutiply => {
                    // TODO
                },
                LooperCommandType::Stop => {
                    self.transition_to(LooperMode::None);
                }

                LooperCommandType::EnablePlay => {
                    if self.mode == LooperMode::Record {
                        self.transition_to(LooperMode::Stopping);
                    } else {
                        self.transition_to(LooperMode::Playing);
                    }
                },
                LooperCommandType::Select => {
                    // TODO: handle
                },
                LooperCommandType::Delete => {
                    self.deleted = true;
                },

                LooperCommandType::ReadyOverdubPlay => {
                    if self.samples.is_empty() {
                        self.transition_to(LooperMode::Ready);
                    } else if self.mode == LooperMode::Record || self.mode == LooperMode::Playing {
                        self.transition_to(LooperMode::Overdub);
                    } else {
                        self.transition_to(LooperMode::Playing);
                    }
                }
            }
        } else {
            // TODO: log this
        }
    }

    pub fn length_in_samples(&self) -> u64 {
        self.samples.get(0).map(|s| s.length()).unwrap_or(0)
    }
}
