//! `.XVG` sound-group descriptors (group loader `AUDIO_LoadGroup`, file name unknown, 0x002BD6C0).
//!
//! ```text
//! u32 'GSVS'  u32 version (3)
//! u32 categoryMask  f32 volume% (/100 at load)  u32 eventCount  u32 bankCount
//! bankCount  x { str name; str dir; str (unused) }          -> bank = <xvg dir>/<dir>/<name>.XVA
//! eventCount x { event(20); soundCount x { sound(64); sampleCount x sample(16); chainCount x chain(12) } }
//! names: per event { str event; per sound { str sound; per sample str; per chain str } }   (debug, unread)
//! ```
//! _AUDIO_ReadString.cpp 0x00313010 / 0x003130B0: strings are a 7-bit varint length then bytes.
//! records are read into the runtime structs at +4 (+0 is the runtime link); offsets below are runtime.
//! event hashes are `HANDY_CRC32(uppercase name, 0)`.

use visos::handy::handy_crc::crc32;

pub const XVG_MAGIC: u32 = 0x5356_5347; // 'GSVS'
pub const XVG_VERSION: u32 = 3;

#[derive(thiserror::Error, Debug)]
pub enum XvgError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad magic {0:#x}")]
    Magic(u32),
    #[error("bad version {0}")]
    Version(u32),
    #[error("truncated at {0:#x}")]
    Truncated(usize),
}

/// event name hash as the data stores it.
pub fn event_hash(name: &str) -> u32 {
    if name.is_empty() { 0 } else { crc32(name.to_ascii_uppercase().as_bytes(), 0) }
}

/// selection mode in a `flags & 0xF00` word (event sound pick, sound sample/chain pick, ranges).
pub mod select {
    pub const RANDOM: u32 = 0x000;
    pub const SEQUENTIAL: u32 = 0x100;
    pub const PARAMETER: u32 = 0x200;
    pub const MIN: u32 = 0x300;
    pub const MID: u32 = 0x400;
    pub const MAX: u32 = 0x500;
    /// pick by `+4` weights instead of uniformly
    pub const WEIGHTED: u32 = 0x1000;
    /// random pick never repeats the previous index
    pub const NO_REPEAT: u32 = 0x2000;
    /// parameter-driven value is re-evaluated every update
    pub const CONTINUOUS: u32 = 0x4000;
}

/// event flags (+12)
pub mod event_flags {
    /// the sample loops (unless the sound chains on end)
    pub const LOOP: u32 = 0x20000;
    /// sample voice adds the global pitch offset (`dword_7829F4`)
    pub const GLOBAL_PITCH: u32 = 0x400000;
}

/// sound chain flags (+56)
pub mod chain_flags {
    /// start a chained event after a delay
    pub const DELAYED: u32 = 0x1000000;
    /// start a chained event when the sample finishes
    pub const ON_END: u32 = 0x2000000;
}

/// `AUDIO_GroupEvent` (32 bytes).
#[derive(Clone, Debug, Default)]
pub struct XvgEvent {
    pub hash: u32,  // +4
    pub unk8: u32,  // +8
    pub flags: u32, // +12 sound selection | event flags
    pub unk16: u32, // +16
    pub sounds: Vec<XvgSound>,
    pub name: String,
}

/// `AUDIO_GroupSound` (84 bytes).
#[derive(Clone, Debug, Default)]
pub struct XvgSound {
    pub weight: u32,       // +4
    pub unk8: u32,         // +8
    pub unk12: u32,        // +12
    pub volume_flags: u32, // +16
    pub volume_min: f32,   // +20 (/100)
    pub volume_max: f32,   // +24 (/100)
    pub pitch_flags: u32,  // +28
    pub pitch_min: f32,    // +32 cents
    pub pitch_max: f32,    // +36
    pub delay_flags: u32,  // +40
    pub delay_min: f32,    // +44 seconds
    pub delay_max: f32,    // +48
    pub sample_flags: u32, // +52 sample selection
    pub chain_flags: u32,  // +56 chain selection | chain_flags
    pub samples: Vec<XvgSample>,
    pub chains: Vec<XvgChain>,
    pub name: String,
}

/// `AUDIO_GroupSample` (20 bytes).
#[derive(Clone, Debug, Default)]
pub struct XvgSample {
    pub weight: u32,  // +4
    pub volume: f32,  // +8 (/100)
    pub bank: u32,    // +12 bank index (runtime: bank pointer)
    pub sample: u32,  // +16 sample index in the bank
    pub name: String,
}

/// `AUDIO_GroupChain` (16 bytes).
#[derive(Clone, Debug, Default)]
pub struct XvgChain {
    pub weight: u32, // +4
    pub event: u32,  // +8 event index in this group (runtime: event pointer)
    pub unk12: u32,  // +12
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct XvgBankRef {
    pub name: String,
    pub dir: String,
    pub extra: String,
}

/// `AUDIO_Group` (40 bytes; +12 category mask, +16 volume, +20/+24 counts, +28 events, +32 banks, +36 name hash).
#[derive(Clone, Debug, Default)]
pub struct XvgGroup {
    pub category_mask: u32, // +12
    pub volume: f32,        // +16 (/100)
    pub banks: Vec<XvgBankRef>,
    pub events: Vec<XvgEvent>,
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn u32(&mut self) -> Result<u32, XvgError> {
        let s = self.b.get(self.pos..self.pos + 4).ok_or(XvgError::Truncated(self.pos))?;
        self.pos += 4;
        Ok(u32::from_le_bytes(s.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, XvgError> {
        Ok(f32::from_bits(self.u32()?))
    }
    /// _AUDIO_ReadString (0x00313010): up to 5 varint bytes of length, then the bytes.
    fn string(&mut self) -> Result<String, XvgError> {
        let mut len = 0usize;
        for i in 0..5 {
            let c = *self.b.get(self.pos).ok_or(XvgError::Truncated(self.pos))?;
            self.pos += 1;
            len += ((c & 0x7F) as usize) << (7 * i);
            if c & 0x80 == 0 {
                break;
            }
        }
        let s = self.b.get(self.pos..self.pos + len).ok_or(XvgError::Truncated(self.pos))?;
        self.pos += len;
        Ok(String::from_utf8_lossy(s).into_owned())
    }
}

impl XvgGroup {
    /// AUDIO_LoadGroup (0x002BD6C0) parse; returns the group and the byte position the names end at.
    pub fn from_bytes(b: &[u8]) -> Result<(Self, usize), XvgError> {
        let mut r = Reader { b, pos: 0 };
        let magic = r.u32()?;
        if magic != XVG_MAGIC {
            return Err(XvgError::Magic(magic));
        }
        let version = r.u32()?;
        if version != XVG_VERSION {
            return Err(XvgError::Version(version));
        }
        let mut g = XvgGroup { category_mask: r.u32()?, volume: r.f32()? / 100.0, ..Default::default() };
        let event_count = r.u32()?;
        let bank_count = r.u32()?;
        for _ in 0..bank_count {
            let name = r.string()?;
            let dir = r.string()?;
            let extra = r.string()?;
            g.banks.push(XvgBankRef { name, dir, extra });
        }
        for _ in 0..event_count {
            let mut e =
                XvgEvent { hash: r.u32()?, unk8: r.u32()?, flags: r.u32()?, unk16: r.u32()?, ..Default::default() };
            let sound_count = r.u32()?;
            for _ in 0..sound_count {
                let mut s = XvgSound {
                    weight: r.u32()?,
                    unk8: r.u32()?,
                    unk12: r.u32()?,
                    volume_flags: r.u32()?,
                    volume_min: r.f32()? / 100.0,
                    volume_max: r.f32()? / 100.0,
                    pitch_flags: r.u32()?,
                    pitch_min: r.f32()?,
                    pitch_max: r.f32()?,
                    delay_flags: r.u32()?,
                    delay_min: r.f32()?,
                    delay_max: r.f32()?,
                    sample_flags: r.u32()?,
                    chain_flags: r.u32()?,
                    ..Default::default()
                };
                let sample_count = r.u32()?;
                let chain_count = r.u32()?;
                for _ in 0..sample_count {
                    s.samples.push(XvgSample {
                        weight: r.u32()?,
                        volume: r.f32()? / 100.0,
                        bank: r.u32()?,
                        sample: r.u32()?,
                        name: String::new(),
                    });
                }
                for _ in 0..chain_count {
                    s.chains.push(XvgChain { weight: r.u32()?, event: r.u32()?, unk12: r.u32()?, name: String::new() });
                }
                e.sounds.push(s);
            }
            g.events.push(e);
        }
        // debug names: not read by the game, parsed here for tooling
        for e in &mut g.events {
            e.name = r.string()?;
            for s in &mut e.sounds {
                s.name = r.string()?;
                for smp in &mut s.samples {
                    smp.name = r.string()?;
                }
                for c in &mut s.chains {
                    c.name = r.string()?;
                }
            }
        }
        Ok((g, r.pos))
    }

    pub fn find_event(&self, hash: u32) -> Option<usize> {
        self.events.iter().position(|e| e.hash == hash)
    }
}
