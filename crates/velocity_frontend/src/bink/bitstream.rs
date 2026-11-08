//! The Bink bit reader: little-endian 32-bit words consumed from bit 0 up (`{cache, word ptr, bits left}`, the
//! three-dword state every reader in the BINK section passes around). Peeks read the next word without
//! advancing; reads past the data return zeros.

#[derive(Clone, Debug)]
pub struct BitReader<'a> {
    data: &'a [u8],
    /// byte offset of the next word to load
    pub pos: usize,
    cache: u64,
    bits: u32,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos, cache: 0, bits: 0 }
    }

    fn word(&self, at: usize) -> u32 {
        let mut b = [0u8; 4];
        for (i, o) in b.iter_mut().enumerate() {
            *o = self.data.get(at + i).copied().unwrap_or(0);
        }
        u32::from_le_bytes(b)
    }

    fn fill(&mut self, n: u32) {
        while self.bits < n {
            self.cache |= (self.word(self.pos) as u64) << self.bits;
            self.pos += 4;
            self.bits += 32;
        }
    }

    /// `n` (1..=32) bits
    pub fn read(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.fill(n);
        let v = (self.cache & ((1u64 << n) - 1)) as u32;
        self.cache >>= n;
        self.bits -= n;
        v
    }

    pub fn bit(&mut self) -> bool {
        self.read(1) != 0
    }

    /// the next `n` bits without consuming them (does not move `pos`, like the original's `*ptr << bits` peek)
    pub fn peek(&self, n: u32) -> u32 {
        let mut c = self.cache;
        if self.bits < n {
            c |= (self.word(self.pos) as u64) << self.bits;
        }
        (c & ((1u64 << n) - 1)) as u32
    }

    pub fn skip(&mut self, n: u32) {
        self.read(n);
    }

    /// byte offset after the last loaded word (the pointer a plane decode returns, 0x00545E20's result)
    pub fn word_end(&self) -> usize {
        self.pos
    }
}
