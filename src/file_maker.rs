use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::vec;

use crate::meta_file::MetaFile;
use crate::util::chaininghash::ChainingHash;
use crate::util::checksumpair::ChecksumPair;
use crate::util::configuration::Configuration;
use crate::util::copy::{arr_copy, arr_fill};
use crate::util::generator::Generator;

#[derive(Debug)]
pub struct FilePart {
    pub start_offset: usize,
    pub end_offset: usize,
    pub block_length: usize,
    pub offset: usize,
}

pub struct FileMaker {
    metafile: MetaFile,
    hashtable: ChainingHash,
    file_map: Vec<i64>,
    file_offset: i64,
}

impl FileMaker {
    pub const RANGES: i32 = 100;

    pub fn new(metafile: &MetaFile) -> Self {
        FileMaker {
            metafile: metafile.clone(),
            hashtable: metafile.hashtable.clone(),
            file_map: vec![-1; metafile.block_num as usize],
            file_offset: 0,
        }
    }

    pub fn file_maker(&self) -> Vec<FilePart> {
        let mut res = Vec::new();
        for i in 0..self.file_map.len() {
            let file_offset = self.file_map[i];

            if file_offset == -1 {
                let range_list = self.range_look_up(i);
                let range = range_list.len();

                let start_offset = range_list[0].0;
                let end_offset = range_list[range - 1].1;

                let block_length = FileMaker::calc_block_length(
                    i as i32,
                    self.metafile.blocksize as i32,
                    self.metafile.length as i32,
                );
                let offset = (range - range_list.len()) * self.metafile.blocksize;

                res.push(FilePart {
                    start_offset: start_offset as usize,
                    end_offset: end_offset as usize,
                    block_length: block_length as usize,
                    offset: offset as usize,
                });

                // dbg!("Range: {} - {}", start_offset, end_offset);
                // dbg!("Block Length: {} Offset: {}", block_length, offset);
            }
            // else {
            //     // normal copy
            // }
        }

        res
    }

    fn calc_block_length(i: i32, block_size: i32, length: i32) -> i32 {
        if (i + 1) * block_size < length {
            block_size
        } else {
            FileMaker::calc_block_length_b(i, block_size, length)
        }
    }

    fn calc_block_length_b(i: i32, block_size: i32, length: i32) -> i32 {
        block_size + (length - (i * block_size + block_size))
    }

    pub fn range_look_up(&self, index: usize) -> Vec<(i64, i64)> {
        let mut ranges: Vec<(i64, i64)> = Vec::new();

        for i in index..self.file_map.len() {
            if ranges.len() >= FileMaker::RANGES as usize {
                break;
            } else if self.file_map[i] == -1 {
                ranges.push((
                    (i * self.metafile.blocksize) as i64,
                    ((i * self.metafile.blocksize) + self.metafile.blocksize) as i64,
                ));
            } else {
                break;
            }
        }

        if !ranges.is_empty() {
            //rangeQueu = true;
        }

        ranges
    }

    pub fn remaining_size(&self, progress: f64) -> f64 {
        (self.metafile.length as f64 * (100.0 - progress)) / 100.0
    }

    pub fn map_matcher(&mut self, target_file: &Path) -> f64 {
        let mut buffer_offset: i32 = 0;

        let file_length = target_file.metadata().unwrap().len();

        let mebi_byte = 1048576;

        let mut config = Configuration::new();
        config.block_length = self.metafile.blocksize as i32;
        config.strong_sum_length = self.metafile.checksum_bytes as i32;

        let mut gen = Generator::new(config);

        let mut back_buffer = vec![0u8; self.metafile.blocksize as usize];
        let mut block_buffer = vec![0u8; self.metafile.blocksize as usize];

        let mut file_buffer: Vec<u8>;
        if self.metafile.length < mebi_byte && self.metafile.blocksize < self.metafile.length {
            file_buffer = vec![0u8; self.metafile.length];
        } else if self.metafile.blocksize > self.metafile.length
            || self.metafile.blocksize > mebi_byte
        {
            file_buffer = vec![0u8; self.metafile.blocksize];
        } else {
            file_buffer = vec![0_u8; mebi_byte];
        }

        let mut first_block = true;
        let len = file_buffer.len();
        let blocksize = self.metafile.blocksize;

        let mut new_byte: u8;

        let mut last_match: i64 = 0;

        let mut n: i32;
        let mut weak_sum = 0;
        let mut strong_sum: Vec<u8>;
        let mut end = false;

        let mut in_buf = File::open(target_file).expect("Unable to open file");

        while self.file_offset as u64 != file_length {
            file_buffer.resize(len, 0);
            n = in_buf.read(&mut file_buffer).expect("cant read") as i32; // maybe reads too much
                                                                          // n = file_buffer.len() as i32;

            if first_block {
                weak_sum = gen.generate_weak_sum(&mut file_buffer, 0);
                buffer_offset = self.metafile.blocksize as i32;
                let weak = self.update_weak_sum(weak_sum);

                if self.hash_look_up(weak, Vec::new()) {
                    strong_sum = gen.generate_strong_sum(&mut file_buffer, 0, blocksize);

                    let match_ = self.hash_look_up(weak_sum, strong_sum);
                    if match_ {
                        last_match = self.file_offset;
                    }
                }
                self.file_offset += 1;
                first_block = false;
            }

            while (buffer_offset as usize) < file_buffer.len() {
                new_byte = file_buffer[buffer_offset as usize];

                if (self.file_offset + self.metafile.blocksize as i64) as u64 > file_length {
                    new_byte = 0;
                }

                weak_sum = gen.generate_roll_sum(new_byte);

                let mut found = false;

                if self.file_offset >= last_match + blocksize as i64 {
                    let w_sum = self.update_weak_sum(weak_sum);
                    if self.hash_look_up(w_sum, Vec::new()) {
                        found = true;
                    }
                } else {
                }

                if found {
                    if (self.file_offset + self.metafile.blocksize as i64) as u64 > file_length {
                        if n > 0 {
                            for i in (n as usize)..file_buffer.len() {
                                file_buffer[i] = 0;
                            }
                        } else {
                            let offset = file_buffer.len() - self.metafile.blocksize
                                + buffer_offset as usize
                                + 1;
                            arr_copy(
                                &file_buffer,
                                offset as usize,
                                &mut block_buffer,
                                0,
                                file_buffer.len() - offset as usize,
                            );

                            let block_buffer_len = block_buffer.len();
                            arr_fill(
                                &mut block_buffer,
                                file_buffer.len() - offset as usize,
                                block_buffer_len,
                                0,
                            )
                        }
                    }
                    if (buffer_offset - self.metafile.blocksize as i32 + 1) < 0 {
                        if n > 0 {
                            arr_copy(
                                &back_buffer,
                                ((back_buffer.len() + buffer_offset as usize) as i32
                                    - self.metafile.blocksize as i32
                                    + 1) as usize,
                                &mut block_buffer,
                                0,
                                (self.metafile.blocksize as i32 - buffer_offset - 1) as usize,
                            );

                            arr_copy(
                                &file_buffer,
                                0,
                                &mut block_buffer,
                                (self.metafile.blocksize as i32 - buffer_offset - 1) as usize,
                                buffer_offset as usize + 1,
                            );
                        }
                        strong_sum =
                            gen.generate_strong_sum(&mut block_buffer, 0, blocksize as usize);

                        let temp_weak_sum = self.update_weak_sum(weak_sum);
                        let match_ = self.hash_look_up(temp_weak_sum, strong_sum);
                        if match_ {
                            last_match = self.file_offset;
                        }
                    } else {
                        strong_sum = gen.generate_strong_sum(
                            &mut file_buffer,
                            (buffer_offset - blocksize as i32 + 1) as usize,
                            blocksize,
                        );
                        let temp_weak_sum = self.update_weak_sum(weak_sum);
                        let match_ = self.hash_look_up(temp_weak_sum, strong_sum);
                        if match_ {
                            last_match = self.file_offset;
                        }
                    }
                }

                self.file_offset += 1;
                //println!("Offset: {}", self.file_offset);
                if self.file_offset as u64 == file_length {
                    end = true;
                    break;
                }

                buffer_offset += 1;
            }

            arr_copy(
                &file_buffer,
                file_buffer.len() - self.metafile.blocksize,
                &mut back_buffer,
                0,
                self.metafile.blocksize,
            );
            buffer_offset = 0;

            if end {
                break;
            }
        }
        self.match_control()
    }

    #[allow(clippy::collapsible_if)]
    pub fn match_control(&mut self) -> f64 {
        let mut missing = 0;

        for i in 0..self.file_map.len() {
            if self.metafile.seq_num == 2 {
                if i > 0 && i < self.file_map.len() - 1 {
                    if self.file_map[i - 1] == -1
                        && self.file_map[i] != -1
                        && self.file_map[i + 1] == -1
                    {
                        self.file_map[i] = -1;
                    }
                } else if i == 0 {
                    if self.file_map[i] != -1 && self.file_map[i + 1] == -1 {
                        self.file_map[i] = -1;
                    }
                } else if i == self.file_map.len() - 1 {
                    if self.file_map[i] != -1 && self.file_map[i - 1] == -1 {
                        self.file_map[i] = -1;
                    }
                }
            }
            if self.file_map[i] == -1 {
                missing += 1;
            }
        }

        if !self.file_map.is_empty() {
            (((self.file_map.len() - missing) as f64 / self.file_map.len() as f64) * 100.0) as f64
        } else {
            0.0
        }
    }

    pub fn update_weak_sum(&mut self, weak: i32) -> i32 {
        let rsum: Vec<u8>;

        match self.metafile.rsum_bytes {
            2 => rsum = vec![0, 0, (weak >> 24) as u8, ((weak << 8) >> 24) as u8],
            3 => {
                rsum = vec![
                    ((weak << 8) >> 24) as u8,
                    0,
                    ((weak << 24) >> 24) as u8,
                    (weak >> 24) as u8,
                ]
            }
            4 => {
                rsum = vec![
                    (weak >> 24) as u8,
                    ((weak << 8) >> 24) as u8,
                    ((weak << 16) >> 24) as u8,
                    ((weak << 24) >> 24) as u8,
                ]
            }
            _ => rsum = vec![0; 4],
        }

        let mut weak_sum: i32 = 0;
        weak_sum += (rsum[0] as i32 & 0x000000FF) << 24;
        weak_sum += (rsum[1] as i32 & 0x000000FF) << 16;
        weak_sum += (rsum[2] as i32 & 0x000000FF) << 8;
        weak_sum += rsum[3] as i32 & 0x000000FF;

        weak_sum
    }

    fn hash_look_up(&mut self, weak_sum: i32, strong_sum: Vec<u8>) -> bool {
        if strong_sum.is_empty() {
            let mut p = ChecksumPair::new();
            p.weak = weak_sum;
            p.offset = -1;

            let link = self.hashtable.find(&p);
            if link.is_some() {
                return true;
            }
        } else {
            let mut p = ChecksumPair::new();
            p.weak = weak_sum;
            p.strong = strong_sum.clone();
            p.offset = -1;

            let link = self.hashtable.find_match(&p);
            if let Some(link) = link {
                let seq = link.seq;
                self.file_map[seq as usize] = self.file_offset;

                let mut del_p = ChecksumPair::new();
                del_p.weak = weak_sum;
                del_p.strong = strong_sum;
                del_p.offset = (self.metafile.blocksize * seq as usize) as i64;
                del_p.length = self.metafile.blocksize as i32;
                del_p.seq = seq;

                self.hashtable.delete(&del_p);
                return true;
            }
        }

        false
    }
}
