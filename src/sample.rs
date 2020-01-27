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

    pub fn length(&self) -> u64 {
        self.len
    }

    pub fn record(&mut self, window_size: u64, time_in_samples: u64, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        let new_len = time_in_samples + data[0].len() as u64;
        if (self.buffer[0].len() as u64) < new_len {
            println!("window_size: {}, size: {}, time: {} last_idx: {}", window_size, self.length(), time_in_samples, new_len);

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
