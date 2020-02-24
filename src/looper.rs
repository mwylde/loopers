use crate::sample::Sample;
use crate::protos::{LooperMode, LooperCommand, LooperCommandType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let looper = Looper::new(1);
        assert_eq!(LooperMode::None, looper.mode);
        assert_eq!(1, looper.id);
        assert_eq!(0, looper.length_in_samples());
    }

    #[test]
    fn test_transitions() {
        let mut looper = Looper::new(1);

        assert_eq!(LooperMode::None, looper.mode);

        looper.transition_to(LooperMode::Record);
        assert_eq!(LooperMode::Record, looper.mode);
        assert_eq!(1, looper.samples.len());

        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        looper.process_input(0, &[&data[0], &data[1]]);

        looper.transition_to(LooperMode::Overdub);
        assert_eq!(2, looper.samples.len());
        for s in &looper.samples {
            assert_eq!(2, s.length());
        }

        looper.transition_to(LooperMode::Playing);
        assert_eq!(LooperMode::Playing, looper.mode);

        looper.transition_to(LooperMode::Record);
        assert_eq!(1, looper.samples.len());
        assert_eq!(0, looper.length_in_samples());
    }
}


// The Looper struct encapsulates behavior similar to a single hardware looper. Internally, it is
// driven by a state machine, which controls how it responds to input buffers (e.g., by recording
// or overdubbing to its internal buffers) and output buffers (e.g., by playing).
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
                self.samples.push(Sample::new());
            },
            (_, LooperMode::Overdub) => {
                let len = self.length_in_samples();
                if len == 0 {
                    panic!("moving to overdub with 0-length looper")
                }
                self.samples.push(Sample::with_size(len as usize));
            },
            (_, LooperMode::Playing) => {},
            (_, LooperMode::Stopping) => {},
        }

        self.mode = mode;
    }

    // In process_output, we modify the specified output buffers according to our internal state. In
    // Playing or Overdub mode, we will add our buffer to the output. Otherwise, we do nothing.
    pub fn process_output(&self, time: u64, outputs: &mut [Vec<f64>; 2]) {
        if self.mode == LooperMode::Playing || self.mode == LooperMode::Overdub {
            let mut time = time as usize;
            if !self.samples.is_empty() {
                for i in 0..outputs[0].len() {
                    for sample in &self.samples {
                        let b = &sample.buffer;
                        assert_eq!(b[0].len(), b[1].len());
                        if b[0].len() > 0 {
                            for (j, o) in outputs.iter_mut().enumerate() {
                                o[i] += b[j][time & b[j].len()] as f64;
                            }
                        }
                    }
                    time += 1;
                }
            }
        }
    }

    // In process_input, we modify our internal buffers based on the input. In Record mode, we
    // append the data in the input buffers to our current sample. In Overdub mode, we sum the data
    // with whatever is currently in our buffer at the point of time_in_samples.
    pub fn process_input(&mut self, time_in_samples: u64, inputs: &[&[f32]]) {
        let len = self.length_in_samples();
        if self.mode == LooperMode::Overdub {
            // in overdub mode, we add the new samples to our existing buffer
            let s = self.samples.last_mut().expect("No samples for looper in overdub mode");
            s.overdub(time_in_samples % len, inputs);
        } else if self.mode == LooperMode::Record {
            // in record mode, we extend the current buffer with the new samples
            let s = self.samples.last_mut().expect("No samples for looper in record mode");
            s.record(inputs);
        }
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
