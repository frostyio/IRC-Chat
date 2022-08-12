pub struct Reader {
    buffer: Vec<u8>,
    idx: usize,
}

impl Reader {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            buffer: bytes,
            idx: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        return self.idx + 1 < self.buffer.len() - 1;
    }

    pub fn byte(&mut self) -> u8 {
        self.idx += 1;
        self.buffer[self.idx - 1]
    }

    pub fn bytes(&mut self, n: usize) -> &[u8] {
        self.idx += n;
        &self.buffer[self.idx - n..self.idx]
    }

    pub fn short(&mut self) -> u16 {
        (self.byte() as u16) + ((self.byte() as u16) << 8)
    }

    pub fn u64(&mut self) -> u64 {
        u64::from_be_bytes(self.bytes(8).try_into().unwrap())
    }

    pub fn f32(&mut self) -> f32 {
        f32::from_be_bytes(self.bytes(4).try_into().unwrap())
    }

    pub fn i32(&mut self) -> i32 {
        i32::from_be_bytes(self.bytes(4).try_into().unwrap())
    }

    pub fn string(&mut self) -> String {
        let len = self.i32() as usize;
        String::from_utf8(self.bytes(len).to_vec()).unwrap()
    }
}
