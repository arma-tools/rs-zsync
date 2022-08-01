#[derive(Debug)]
pub struct ChecksumPair {
    pub weak: i32,
    pub strong: Vec<u8>,
    pub offset: i64,
    pub length: i32,
    pub seq: i32,
}

impl Clone for ChecksumPair {
    fn clone(&self) -> ChecksumPair {
        ChecksumPair {
            weak: self.weak,
            strong: self.strong.clone(),
            offset: self.offset,
            length: self.length,
            seq: self.seq,
        }
    }
}

impl ChecksumPair {
    pub fn new() -> Self {
        ChecksumPair {
            weak: 0,
            strong: Vec::new(),
            offset: 0,
            length: 0,
            seq: 0,
        }
    }

    pub fn hash_code(&self) -> i32 {
        let weak_byte: Vec<u8> = vec![
            (self.weak >> 24) as u8,
            ((self.weak << 8) >> 24) as u8,
            ((self.weak << 16) >> 24) as u8,
            ((self.weak << 24) >> 24) as u8,
        ];

        let weak_add: Vec<u8> = vec![
            weak_byte[0].overflowing_add(weak_byte[1]).0,
            weak_byte[2].overflowing_add(weak_byte[3]).0,
        ];
        let mut hash_code: i32 = 0;
        for i in 0..2 {
            let shift: i32 = (1 - i) * 8;
            hash_code += ((weak_add[i as usize] as i32) & 0x00FF) << shift;
        }
        hash_code
    }
}

impl Default for ChecksumPair {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ChecksumPair {
    fn eq(&self, other: &Self) -> bool {
        self.weak == other.weak && self.strong == other.strong
    }
}
