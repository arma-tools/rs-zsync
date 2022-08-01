use md4::{Digest, Md4};

use super::rsum::Rsum;

const CONFIG_BLOCK_LENGTH: i32 = 1024;
const CONFIG_CHUNK_SIZE: i32 = 32768;

pub struct Configuration {
    pub(crate) block_length: i32,
    pub(crate) strong_sum_length: i32,
    pub(crate) do_run_length: bool,
    pub(crate) checksum_seed: Vec<u8>,
    pub(crate) chunk_size: i32,

    pub(crate) weak_sum: Rsum,
    pub(crate) strong_sum: Md4,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration {
            block_length: CONFIG_BLOCK_LENGTH,
            strong_sum_length: 0,
            do_run_length: false,
            checksum_seed: Vec::new(),
            chunk_size: CONFIG_CHUNK_SIZE,
            weak_sum: Rsum::new(),
            strong_sum: Md4::new(),
        }
    }
}
