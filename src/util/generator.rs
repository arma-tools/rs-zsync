use md4::{Digest, Md4};

use super::{configuration::Configuration, copy::arr_copy};

pub struct Generator {
    config: Configuration,
}

impl Generator {
    pub fn new(config: Configuration) -> Self {
        Generator { config }
    }

    pub fn generate_weak_sum(&mut self, buf: &mut [u8], offset: i32) -> i32 {
        self.config
            .weak_sum
            .first(buf, offset, self.config.block_length);
        self.config.weak_sum.get_value()
    }

    pub fn generate_strong_sum(&mut self, buf: &mut [u8], off: usize, len: usize) -> Vec<u8> {
        self.config.strong_sum.update(&buf[off..(off + len)]);
        let hasher = self.config.strong_sum.clone();
        let hash: Vec<u8> = hasher.finalize().to_vec();
        self.config.strong_sum = Md4::new();

        let mut strong_sum = vec![0_u8; self.config.strong_sum_length as usize];
        arr_copy(
            &hash,
            0,
            &mut strong_sum,
            0,
            self.config.strong_sum_length as usize,
        );
        strong_sum
    }

    pub fn generate_roll_sum(&mut self, b: u8) -> i32 {
        self.config.weak_sum.roll(b);
        self.config.weak_sum.get_value()
    }
}
