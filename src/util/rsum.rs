pub struct Rsum {
    a: i16,
    b: i16,
    old_byte: i32,
    block_length: i32,
    buffer: Vec<u8>,
}

impl Rsum {
    pub fn new() -> Self {
        Rsum {
            a: 0,
            b: 0,
            old_byte: 0,
            block_length: 0,
            buffer: Vec::new(),
        }
    }

    pub fn get_value(&self) -> i32 {
        (self.a as i32 & 0xffff) | ((self.b as i32) << 16)
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.old_byte = 0;
    }

    pub fn roll(&mut self, new_byte: u8) {
        let old_unsigned_b = self.buffer[self.old_byte as usize] as i16;
        self.a = self.a.overflowing_sub(old_unsigned_b).0;
        self.b = self
            .b
            .overflowing_sub((self.block_length * old_unsigned_b as i32) as i16)
            .0;
        self.a = self.a.overflowing_add(new_byte as i16).0;
        self.b = self.b.overflowing_add(self.a).0;
        self.buffer[self.old_byte as usize] = new_byte;
        self.old_byte += 1;
        if self.old_byte == self.block_length {
            self.old_byte = 0;
        }
    }

    pub fn check(&mut self, buf: &mut [u8], offset: i32, length: i32) {
        self.reset();
        let mut index = offset;
        let mut unsigned_b: i16;
        for i in (1..(length + 1)).rev() {
            unsigned_b = buf[index as usize] as i16;
            self.a = self.a.overflowing_add(unsigned_b).0;
            self.b = self.b.overflowing_add((i * unsigned_b as i32) as i16).0;
            index += 1;
        }
    }

    pub fn first(&mut self, buf: &mut [u8], offset: i32, length: i32) {
        self.reset();
        let mut index = offset;
        let mut unsigned_b: i16;
        for i in (1..(length + 1)).rev() {
            unsigned_b = buf[index as usize] as i16;
            self.a = self.a.overflowing_add(unsigned_b).0;
            self.b = self.b.overflowing_add((i * unsigned_b as i32) as i16).0;
            index += 1;
        }

        self.block_length = length;
        //self.buffer = vec![0; self.block_length];
        self.buffer = buf.to_vec();
        self.buffer.resize(self.block_length as usize, 0);
    }

    // pub fn unsigned_byte(b: u8) -> i16 {
    //     if b < 0 {
    //         return b.overflowing_add(256).0 as i16;
    //     }
    //     b as i16
    // }
}
