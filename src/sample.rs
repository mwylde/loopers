use std::sync::Arc;

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
        assert_eq!(vec![ 1.0f32,  1.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0], sample.buffer[1]);

        sample.record(&[&data[1], &data[0]]);
        assert_eq!(4, sample.length());
        assert_eq!(vec![ 1.0f32,  1.0, -1.0, -1.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0,  1.0,  1.0], sample.buffer[1]);
    }

    #[test]
    fn test_overdub() {
        let mut sample = Sample::with_size(8);
        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        sample.overdub(0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 1.0f32,  1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[1]);

        sample.overdub(0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 2.0f32,  2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[0]);
        assert_eq!(vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[1]);


        sample.overdub(6, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 2.0f32,  2.0, 0.0, 0.0, 0.0, 0.0,  1.0,  1.0], sample.buffer[0]);
        assert_eq!(vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0], sample.buffer[1]);
    }

}

#[derive(Clone)]
pub struct Sample {
    pub buffer: [Vec<f32>; 2],
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
        let half: Vec<f32> = buffer.iter().map(|x|  *x / 2f32).collect();
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
}


pub struct SamplePlayer {
    pub sample: Arc<Sample>,
    pub time: usize,
}

#[derive(PartialOrd, PartialEq)]
pub enum PlayOutput {
    Done, NotDone
}

impl SamplePlayer {
    pub fn new(sample: Arc<Sample>) -> SamplePlayer {
        SamplePlayer { sample, time: 0}
    }

    pub fn play(&mut self, out: &mut [&mut [f32]; 2]) -> PlayOutput {
        for i in 0..out[0].len() {
            let t = self.time + i;


            if t >= self.sample.length() as usize {
                return PlayOutput::Done;
            }

            out[0][i] += self.sample.buffer[0][t];
            out[1][i] += self.sample.buffer[1][t];
        }

        self.time += out[0].len();

        if self.sample.length() <= self.time as u64 {
            PlayOutput::Done
        } else {
            PlayOutput::NotDone
        }
    }
}
