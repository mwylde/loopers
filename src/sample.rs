pub struct Sample {
    pub buffer: [Vec<f32>; 2],
}

impl Sample {
    pub fn new() -> Sample {
        Sample {
            buffer: [vec![], vec![]],
        }
    }

    pub fn length(&self) -> u64 {
        assert_eq!(self.buffer[0].len(), self.buffer[1].len());
        self.buffer[0].len() as u64
    }

    pub fn record(&mut self, time_in_samples: u64, data: &[&[f32]]) {
        assert_eq!(2, data.len());
        assert_eq!(data[0].len(), data[1].len());

        let new_len = time_in_samples + data[0].len() as u64;
        if self.length() < new_len {
            for b in &mut self.buffer {
                b.resize(new_len as usize, 0f32);
            }
        }

        for (i, channel) in data.iter().enumerate() {
            for (t, v) in channel.iter().enumerate() {
                self.buffer[i][time_in_samples as usize + t] += *v;
            }
        }
    }
}
