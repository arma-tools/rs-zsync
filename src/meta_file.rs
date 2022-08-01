use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::{fs::File, path::Path};

use chrono::{DateTime, FixedOffset, NaiveDateTime};

use crate::util::{chaininghash::ChainingHash, checksumpair::ChecksumPair};

#[derive(Debug)]
pub struct MetaFile {
    pub zsync: String,
    pub filename: String,
    pub m_time: DateTime<FixedOffset>,
    pub blocksize: usize,
    pub length: usize,
    pub seq_num: u32,
    pub rsum_bytes: u32,
    pub checksum_bytes: u32,
    pub block_num: u32,
    pub url: String,
    pub sha1: String,

    pub hashtable: ChainingHash,
}

impl MetaFile {
    pub fn new() -> MetaFile {
        MetaFile {
            zsync: String::new(),
            filename: String::new(),
            m_time: DateTime::<FixedOffset>::from_utc(
                NaiveDateTime::from_timestamp(0, 0),
                FixedOffset::east(0),
            ),
            blocksize: 0,
            length: 0,
            url: String::new(),
            sha1: String::new(),
            seq_num: 0,
            rsum_bytes: 0,
            checksum_bytes: 0,
            block_num: 0,
            hashtable: ChainingHash::new(0),
        }
    }

    pub fn parse_zsync(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut br = BufReader::new(file);

        //let mut kvps = HashMap::new();

        let mut run: bool = true;

        while run {
            let mut line = String::new();
            match br.read_line(&mut line) {
                Ok(_) => {
                    if !line.is_empty() {
                        let splitted: Vec<String> =
                            line.splitn(2, ':').map(|s| s.trim().to_string()).collect();

                        if splitted.len() == 2 {
                            let value = splitted[1].clone();
                            dbg!(value.clone());
                            match splitted[0].trim().to_lowercase().as_ref() {
                                "zsync" => self.zsync = value,
                                "filename" => self.filename = value,
                                "mtime" => {
                                    self.m_time =
                                        DateTime::parse_from_rfc2822(&value).expect("no parse")
                                }
                                "blocksize" => self.blocksize = value.parse().unwrap_or_default(),
                                "length" => self.length = value.parse().unwrap_or_default(),
                                "hash-lengths" => {
                                    let hash_lenghts: Vec<u32> = value
                                        .split(',')
                                        .map(|s| s.parse().unwrap_or_default())
                                        .collect();
                                    self.seq_num = *hash_lenghts.get(0).unwrap_or(&0);
                                    self.rsum_bytes = *hash_lenghts.get(1).unwrap_or(&0);
                                    self.checksum_bytes = *hash_lenghts.get(2).unwrap_or(&0);
                                }
                                "url" => self.url = value,
                                "sha-1" => self.sha1 = value,
                                e => println!("Unknown: {}", e),
                            }
                        } else if line.len() == 1 && line.bytes().next() == Some(0x0A) {
                            run = false;
                        }
                    } else {
                        run = false;
                    }
                }
                Err(_) => run = false,
            }
        }

        self.block_num = (self.length as f64 / self.blocksize as f64).ceil() as u32;

        //br.seek(SeekFrom::Current(-4));
        let mut buf = Vec::new();
        //br.seek(SeekFrom::Start(0))?;
        br.read_to_end(&mut buf)?;

        self.fill_hash_table(buf);

        Ok(())
    }

    fn fill_hash_table(&mut self, checksums: Vec<u8>) {
        let mut i: u32 = 16;

        while (2 << (i - 1)) > self.block_num && i > 4 {
            i -= 1;
        }

        self.hashtable = ChainingHash::new((2 << (i - 1)) as i32);

        let mut offset: i64 = 0;
        let mut weak_sum: i32 = 0;
        let mut seq: i32 = 0;
        let mut off = 0;

        let mut weak = vec![0u8; 4];
        let mut strong_sum = vec![0u8; self.checksum_bytes as usize];

        while seq < self.block_num as i32 {
            for w in 0..self.rsum_bytes {
                weak[w as usize] = checksums[off];
                off += 1;
            }

            for s in 0..strong_sum.len() {
                strong_sum[s] = checksums[off];
                off += 1;
            }
            weak_sum = 0;
            weak_sum += (weak[2] as i32 & 0x000000FF) << 24;
            weak_sum += (weak[3] as i32 & 0x000000FF) << 16;
            weak_sum += (weak[0] as i32 & 0x000000FF) << 8;
            weak_sum += weak[1] as i32 & 0x000000FF;

            let pair = ChecksumPair {
                weak: weak_sum,
                strong: strong_sum.clone(),
                offset,
                length: self.blocksize as i32,
                seq,
            };

            offset += self.blocksize as i64;
            seq += 1;
            self.hashtable.insert(&pair);
        }
    }
}

impl Default for MetaFile {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MetaFile {
    fn clone(&self) -> Self {
        Self {
            zsync: self.zsync.clone(),
            filename: self.filename.clone(),
            m_time: self.m_time,
            blocksize: self.blocksize,
            length: self.length,
            seq_num: self.seq_num,
            rsum_bytes: self.rsum_bytes,
            checksum_bytes: self.checksum_bytes,
            block_num: self.block_num,
            url: self.url.clone(),
            sha1: self.sha1.clone(),
            hashtable: self.hashtable.clone(),
        }
    }
}
