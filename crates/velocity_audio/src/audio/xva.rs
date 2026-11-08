//! `.XVA` sample banks (bank loader `_AUDIO_LoadBank`, file name unknown, code at 0x00316450 / 0x003422F0).
//!
//! ```text
//! +0  u32 'ASVX'   +4 u32 version (3)   +8 u32 sampleCount   +12 u32 bank flags
//! +16 sampleCount x { u32 byteSize; u32 format; u32 fileOffset }
//! ```
//! format word: bits 0-3 codec (0 pcm, 1 xbox adpcm), 4-6 channels, 7-11 bits per sample,
//! 12-15 rate index (`_AUDIO_RateFromFormat`). bank flags 0x70000 is the residency class
//! (0x20000 = streamed from disc, data at 2 KB aligned offsets); the loader ors it into every
//! sample's format word. resident banks are read in one go from the first sample's offset,
//! each sample padded to 4 bytes.

use std::path::Path;

use super::adpcm;

pub const XVA_MAGIC: u32 = 0x5856_5341; // 'ASVX'
pub const XVA_VERSION: u32 = 3;

/// bank flag: sample data streams from disc instead of being loaded (`(flags & 0x70000) == 0x20000`).
pub const BANK_STREAMED: u32 = 0x20000;
pub const BANK_RESIDENCY_MASK: u32 = 0x70000;

#[derive(thiserror::Error, Debug)]
pub enum XvaError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad magic {0:#x}")]
    Magic(u32),
    #[error("bad version {0}")]
    Version(u32),
    #[error("truncated")]
    Truncated,
}

/// _AUDIO_FormatTag.cpp 0x00341600: wave format tag from the codec nibble.
pub fn format_tag(format: u32) -> u16 {
    match format & 0xF {
        0 => 1,
        1 => 105,
        _ => 0,
    }
}

/// _AUDIO_RateFromFormat.cpp 0x00341540 unknown indices fall back to 30000 like the original.
pub fn rate_from_format(format: u32) -> u32 {
    match (format & 0xF000) >> 12 {
        1 => 8000,
        2 => 11025,
        3 => 12000,
        4 => 16000,
        5 => 22050,
        6 => 24000,
        7 => 32000,
        8 => 44100,
        9 => 48000,
        0xA => 64000,
        0xB => 28000,
        0xC => 30000,
        _ => 30000,
    }
}

/// _AUDIO_RateIndex.cpp 0x00341430: inverse of `rate_from_format`.
pub fn rate_index(rate: u32) -> u32 {
    match rate {
        8000 => 1,
        11025 => 2,
        12000 => 3,
        16000 => 4,
        22050 => 5,
        24000 => 6,
        28000 => 11,
        30000 => 12,
        32000 => 7,
        44100 => 8,
        48000 => 9,
        64000 => 10,
        _ => 0,
    }
}

pub fn channels_from_format(format: u32) -> u32 {
    (format & 0x70) >> 4
}

pub fn bits_from_format(format: u32) -> u32 {
    (format & 0xF80) >> 7
}

/// one 12-byte sample header (`AUDIO_SampleHeader`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XvaSampleHeader {
    pub size: u32,   // +0 bytes of data
    pub format: u32, // +4 format word | bank residency flags
    pub offset: u32, // +8 file offset (runtime: pointer into the bank's data block)
}

impl XvaSampleHeader {
    pub fn channels(&self) -> u32 {
        channels_from_format(self.format)
    }
    pub fn rate(&self) -> u32 {
        rate_from_format(self.format)
    }
    pub fn is_adpcm(&self) -> bool {
        format_tag(self.format) == 105
    }
    /// sample frames in the data, as the voice sees them.
    pub fn frames(&self) -> usize {
        let ch = self.channels().max(1) as usize;
        if self.is_adpcm() {
            adpcm::frames_in(self.size as usize, ch)
        } else {
            let bits = bits_from_format(self.format).max(8) as usize;
            self.size as usize / (ch * bits / 8)
        }
    }
    /// playback length in seconds at the native rate.
    pub fn duration(&self) -> f64 {
        self.frames() as f64 / self.rate() as f64
    }
}

/// a loaded bank (`AUDIO_Bank`, 40 bytes in the original; +12 count, +16 headers, +24 data, +28 name hash).
#[derive(Clone, Debug, Default)]
pub struct XvaBank {
    pub flags: u32,
    pub samples: Vec<XvaSampleHeader>,
    /// resident data: every sample's bytes, each padded to 4 (empty for streamed banks).
    pub data: Vec<u8>,
    /// offset of each sample inside `data` (resident banks).
    pub data_offsets: Vec<usize>,
}

impl XvaBank {
    pub fn is_streamed(&self) -> bool {
        self.flags & BANK_RESIDENCY_MASK == BANK_STREAMED
    }

    /// parse the header and sample table only.
    pub fn parse_header(b: &[u8]) -> Result<(u32, Vec<XvaSampleHeader>), XvaError> {
        let rd = |o: usize| -> Result<u32, XvaError> {
            b.get(o..o + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap())).ok_or(XvaError::Truncated)
        };
        let magic = rd(0)?;
        if magic != XVA_MAGIC {
            return Err(XvaError::Magic(magic));
        }
        let version = rd(4)?;
        if version != XVA_VERSION {
            return Err(XvaError::Version(version));
        }
        let count = rd(8)? as usize;
        let flags = rd(12)?;
        let mut samples = Vec::with_capacity(count);
        for i in 0..count {
            let o = 16 + i * 12;
            samples.push(XvaSampleHeader { size: rd(o)?, format: rd(o + 4)?, offset: rd(o + 8)? });
        }
        Ok((flags, samples))
    }

    /// load a bank from bytes the way `_AUDIO_LoadBank` + `_AUDIO_LoadBankDataM` do (resident banks get their
    /// data block; streamed banks keep only headers). `load_flags` is the caller's flag word (`a2`).
    pub fn from_bytes(b: &[u8], load_flags: u32) -> Result<Self, XvaError> {
        let (file_flags, mut samples) = Self::parse_header(b)?;
        let flags = if load_flags & 1 != 0 { file_flags } else { load_flags | (file_flags & 0xFFF8_FF00) };
        let mut bank = XvaBank { flags, ..Default::default() };
        if !bank.is_streamed() && !samples.is_empty() {
            // _AUDIO_LoadBankDataM: one read of sum(align4(size)) bytes from the first sample's offset
            let total: usize = samples.iter().map(|s| (s.size as usize + 3) & !3).sum();
            let start = samples[0].offset as usize;
            let src = b.get(start..start + total).ok_or(XvaError::Truncated)?;
            bank.data = src.to_vec();
            let mut at = 0usize;
            for s in &samples {
                bank.data_offsets.push(at);
                at += (s.size as usize + 3) & !3;
            }
        }
        for s in &mut samples {
            s.format |= flags & BANK_RESIDENCY_MASK;
        }
        bank.samples = samples;
        Ok(bank)
    }

    pub fn load(path: &Path, load_flags: u32) -> Result<Self, XvaError> {
        let b = std::fs::read(path)?;
        Self::from_bytes(&b, load_flags)
    }

    /// raw bytes of sample `i`: resident from the data block, streamed from the file image `file`.
    pub fn sample_bytes<'a>(&'a self, i: usize, file: Option<&'a [u8]>) -> Option<&'a [u8]> {
        let s = self.samples.get(i)?;
        if self.is_streamed() {
            let f = file?;
            f.get(s.offset as usize..s.offset as usize + s.size as usize)
        } else {
            let o = *self.data_offsets.get(i)?;
            self.data.get(o..o + s.size as usize)
        }
    }
}

/// decode raw sample bytes to interleaved s16 frames.
pub fn decode_sample(h: &XvaSampleHeader, bytes: &[u8]) -> Vec<i16> {
    let ch = h.channels().max(1) as usize;
    if h.is_adpcm() {
        adpcm::decode(bytes, ch)
    } else if bits_from_format(h.format) == 8 {
        // _AUDIO_Resample reads 8-bit data as signed char
        bytes.iter().map(|&b| ((b as i8) as i16) << 8).collect()
    } else {
        bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
    }
}
