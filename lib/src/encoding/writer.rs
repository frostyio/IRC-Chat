pub struct Writer {
    buffer: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { buffer: vec![] }
    }

    pub fn dump(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    pub fn dump_with_size_hint(&self) -> Vec<u8> {
        let mut size_hint = ((self.buffer.len()) as u64).to_be_bytes().to_vec();
        let mut data = self.buffer.clone();
        size_hint.append(&mut data);
        size_hint
    }

    pub fn byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    pub fn bytes(&mut self, bytes: &[u8]) {
        bytes.iter().for_each(|b| self.byte(*b))
    }

    pub fn short(&mut self, num: u16) {
        self.byte(num as u8);
        self.byte((num >> 8) as u8);
    }

    pub fn u64(&mut self, num: u64) {
        self.bytes(&num.to_be_bytes());
    }

    pub fn f32(&mut self, num: f32) {
        self.bytes(&num.to_be_bytes());
    }

    pub fn i32(&mut self, num: i32) {
        self.bytes(&num.to_be_bytes());
    }

    pub fn string(&mut self, str: &str) {
        self.i32(str.len() as i32);
        self.bytes(str.as_bytes())
    }
}
