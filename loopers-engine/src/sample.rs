use std::sync::Arc;
use std::fmt::{Debug, Formatter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sample = Sample::with_size(100);
        assert_eq!(100, sample.length());
        assert_eq!(100, sample.buffer[0].len());
        assert_eq!(100, sample.buffer[1].len());
    }

    #[test]
    fn test_from_mono() {
        let buf = vec![1.0f32, 1.0, 2.0, 2.0];
        let sample = Sample::from_mono(&buf);
        assert_eq!(4, sample.length());
        assert_eq!(vec![0.5f32, 0.5, 1.0, 1.0], sample.buffer[0]);
        assert_eq!(vec![0.5f32, 0.5, 1.0, 1.0], sample.buffer[1]);
    }

    #[test]
    fn test_record() {
        let mut sample = Sample::new();
        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];

        sample.record(&[&data[0], &data[1]]);
        assert_eq!(2, sample.length());
        assert_eq!(vec![1.0f32, 1.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0], sample.buffer[1]);

        sample.record(&[&data[1], &data[0]]);
        assert_eq!(4, sample.length());
        assert_eq!(vec![1.0f32, 1.0, -1.0, -1.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0, 1.0, 1.0], sample.buffer[1]);
    }

    #[test]
    fn test_overdub() {
        let mut sample = Sample::with_size(8);
        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        sample.overdub(0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(
            vec![1.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            sample.buffer[0]
        );
        assert_eq!(
            vec![-1.0f32, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            sample.buffer[1]
        );

        sample.overdub(0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(
            vec![2.0f32, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            sample.buffer[0]
        );
        assert_eq!(
            vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            sample.buffer[1]
        );

        sample.overdub(6, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(
            vec![2.0f32, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
            sample.buffer[0]
        );
        assert_eq!(
            vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0],
            sample.buffer[1]
        );
    }

    #[test]
    fn test_xfade() {
        let mut sample = Sample::with_size(0);
        let data = [vec![1.0f32; 8], vec![-1.0f32; 8]];
        sample.record(&[&data[0], &data[1]]);

        let xfade = [vec![3.0f32; 3], vec![-3.0f32; 3]];
        sample.xfade(
            3,
            0,
            0,
            &[&xfade[0][0..2], &xfade[1][0..2]],
            XfadeDirection::OUT,
            linear,
        );
        sample.xfade(
            3,
            2,
            2,
            &[&xfade[0][2..], &xfade[1][2..]],
            XfadeDirection::OUT,
            linear,
        );

        let l: Vec<i64> = sample.buffer[0]
            .iter()
            .map(|f| (*f * 1000f32).floor() as i64)
            .collect();
        let r: Vec<i64> = sample.buffer[1]
            .iter()
            .map(|f| (*f * 1000f32).ceil() as i64)
            .collect();

        assert_eq!(8, sample.length());
        assert_eq!(vec![3000i64, 2333, 1666, 1000, 1000, 1000, 1000, 1000], l);
        assert_eq!(
            vec![-3000i64, -2333, -1666, -1000, -1000, -1000, -1000, -1000],
            r
        );
    }
}

#[allow(dead_code)]
pub fn linear(x: f32) -> f32 {
    x
}

pub fn norm(x: f32) -> f32 {
    x / (x * x + (1.0 - x) * (1.0 - x)).sqrt()
}

#[derive(Clone)]
pub struct Sample {
    pub buffer: [Vec<f32>; 2],
}

pub enum XfadeDirection {
    IN,
    OUT,
}

impl Debug for Sample {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<sample [{}]>", self.buffer[0].len())
    }
}

impl Sample {
    pub fn new() -> Sample {
        Sample::with_size(0)
    }

    pub fn with_size(len: usize) -> Sample {
        Sample {
            buffer: [vec![0f32; len], vec![0f32; len]],
        }
    }

    pub fn from_mono(buffer: &[f32]) -> Sample {
        let half: Vec<f32> = buffer.iter().map(|x| *x / 2f32).collect();
        Sample {
            buffer: [half.clone(), half],
        }
    }

    pub fn length(&self) -> u64 {
        self.buffer[0].len() as u64
    }

    // Records data onto this sample, expanding the buffer as necessary
    pub fn record(&mut self, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        self.buffer[0].extend_from_slice(data[0]);
        self.buffer[1].extend_from_slice(data[1]);
    }

    // Overdubs the buffer, starting at the give time. len(data[{0, 1}]) + time_in_samples must
    // be < than self.len().
    pub fn overdub(&mut self, time_in_samples: u64, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());
        let len = self.length() as usize;

        for (i, channel) in data.iter().enumerate() {
            for (t, v) in channel.iter().enumerate() {
                self.buffer[i][(time_in_samples as usize + t) % len] += *v;
            }
        }
    }

    pub fn replace(&mut self, time_in_samples: u64, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());
        let len = self.length() as usize;

        for (i, channel) in data.iter().enumerate() {
            for (t, v) in channel.iter().enumerate() {
                self.buffer[i][(time_in_samples as usize + t) % len] = *v;
            }
        }
    }

    pub fn clear(&mut self) {
        for b in self.buffer.iter_mut() {
            b.iter_mut().for_each(|m| *m = 0.0);
        }
    }

    /// Performs a crossfade with the existing buffer using the given function
    /// The fade direction refers to the given sample -- i.e., a fade in starts
    /// with 100% of the existing sample, and ends at 100% of the new sample.
    pub fn xfade(
        &mut self,
        xfade_size: usize,
        start_time_in_fade: u64,
        time_in_samples: u64,
        data: &[&[f32]],
        direction: XfadeDirection,
        f: fn(f32) -> f32,
    ) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        let len = self.length();

        // let end_time = time_in_samples % len + data[0].len() as u64;
        // assert!(end_time <= xfade_size as u64,
        //         format!("expected {} <= {}", end_time, xfade_size));

        for i in 0..data.len() {
            for j in 0..data[i].len() {
                let idx = ((time_in_samples + j as u64) % len) as usize;
                let q = (start_time_in_fade + j as u64) as f32 / xfade_size as f32;
                self.buffer[i][idx] = match direction {
                    XfadeDirection::IN => self.buffer[i][idx] * f(1.0 - q) + data[i][j] * f(q),
                    XfadeDirection::OUT => self.buffer[i][idx] * f(q) + data[i][j] * f(1.0 - q),
                }
            }
        }
    }
}

pub struct SamplePlayer {
    pub sample: Arc<Sample>,
    pub time: usize,
}

#[derive(PartialOrd, PartialEq)]
pub enum PlayOutput {
    Done,
    NotDone,
}

impl SamplePlayer {
    pub fn new(sample: Arc<Sample>) -> SamplePlayer {
        SamplePlayer { sample, time: 0 }
    }

    pub fn play(&mut self, out: &mut [&mut [f32]; 2], volume: f32) -> PlayOutput {
        for i in 0..out[0].len() {
            let t = self.time + i;

            if t >= self.sample.length() as usize {
                return PlayOutput::Done;
            }

            out[0][i] += self.sample.buffer[0][t] * volume;
            out[1][i] += self.sample.buffer[1][t] * volume;
        }

        self.time += out[0].len();

        if self.sample.length() <= self.time as u64 {
            PlayOutput::Done
        } else {
            PlayOutput::NotDone
        }
    }
}
