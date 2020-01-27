use num::Float;

pub struct Sample<T : Float> {
    pub buffer: [Vec<T>; 2],
    len: u64,
}

impl <T : Float> Sample<T> {
    pub fn new(len: usize) -> Sample<T> {
        Sample {
            buffer: [vec![T::zero(); len], vec![T::zero(); len]],
            len: len as u64
        }
    }

    pub fn from_buf(buffer: [Vec<T>; 2]) -> Sample<T> {
        assert_eq!(buffer[0].len(), buffer[1].len());
        Sample {
            buffer,
            len: buffer[0].len() as u64,
        }
    }

    pub fn from_mono_f32(buffer: &[f32]) -> Sample<f32> {
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

    pub fn record(&mut self, window_size: u64, time_in_samples: u64, data: &[&[T]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        let new_len = time_in_samples + data[0].len() as u64;
        if (self.buffer[0].len() as u64) < new_len {
            println!("window_size: {}, size: {}, time: {} last_idx: {}", window_size, self.length(), time_in_samples, new_len);

            for b in &mut self.buffer {
                b.resize((time_in_samples + window_size) as usize, T::zero());
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
