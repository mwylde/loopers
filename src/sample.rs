#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sample = Sample::new(100);
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
        let mut sample = Sample::new(8);
        let data = [vec![1.0f32, 1.0], vec![-1.0, -1.0]];
        sample.record(4, 0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 1.0f32,  1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[0]);
        assert_eq!(vec![-1.0f32, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[1]);

        sample.record(4, 0, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 2.0f32,  2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[0]);
        assert_eq!(vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], sample.buffer[1]);

        sample.record(4, 6, &[&data[0], &data[1]]);
        assert_eq!(8, sample.length());
        assert_eq!(vec![ 2.0f32,  2.0, 0.0, 0.0, 0.0, 0.0,  1.0,  1.0], sample.buffer[0]);
        assert_eq!(vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0], sample.buffer[1]);

        sample.record(4, 8, &[&data[0], &data[1]]);
        assert_eq!(10, sample.length());
        assert_eq!(vec![ 2.0f32,  2.0, 0.0, 0.0, 0.0, 0.0,  1.0,  1.0,  1.0,  1.0, 0.0, 0.0], sample.buffer[0]);
        assert_eq!(vec![-2.0f32, -2.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, -1.0, -1.0, 0.0, 0.0], sample.buffer[1]);
    }
}

#[derive(Clone)]
pub struct Sample {
    pub buffer: [Vec<f32>; 2],
    len: u64,
}

impl Sample {
    pub fn new(len: usize) -> Sample {
        Sample {
            buffer: [vec![0f32; len], vec![0f32; len]],
            len: len as u64
        }
    }

//    pub fn from_buf(buffer: [Vec<f32>; 2]) -> Sample {
//        assert_eq!(buffer[0].len(), buffer[1].len());
//        let len = buffer[0].len();
//        Sample {
//            buffer,
//            len: len as u64,
//        }
//    }

    pub fn from_mono(buffer: &[f32]) -> Sample {
        let half: Vec<f32> = buffer.iter().map(|x|  *x / 2f32).collect();
        let len = half.len() as u64;
        Sample {
            buffer: [half.clone(), half],
            len,
        }
    }

    pub fn length(&self) -> u64 {
        self.len
    }

    // Records data onto this sample, expanding the buffer as necessary
    //
    // The `window_size` is the unit of growing the buffer; typically this will be the size of a
    // measure (in samples). Data will be rewritten to `time_in_samples`. If the buffer is too
    // small, it will be resized. If there is already data at that location, it will be added to
    // the new data.
    pub fn record(&mut self, window_size: u64, time_in_samples: u64, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        let new_len = time_in_samples + data[0].len() as u64;
        if (self.buffer[0].len() as u64) < new_len {
            for b in &mut self.buffer {
                b.resize((time_in_samples + window_size) as usize, 0f32);
            }
        }

        self.len = (time_in_samples + data[0].len() as u64).max(self.len);

        for (i, channel) in data.iter().enumerate() {
            for (t, v) in channel.iter().enumerate() {
                self.buffer[i][time_in_samples as usize + t] += *v;
            }
        }
    }
}
