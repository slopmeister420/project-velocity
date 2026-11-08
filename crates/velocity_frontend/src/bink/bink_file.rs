//! Bink container: the header / track / frame index part of BinkOpen (0x0053E9A0) and the packet split of
//! BinkDoFrame (0x00541340).
//!
//! File: 44-byte header `{sig, size-8, frames, largest frame, index count, width, height, fps num, fps den,
//! flags, audio tracks}`, then per track `max decoded size`, `rate | flags << 16` (0x10000000 DCT, 0x20000000
//! stereo), `track id`, then `index count + 1` frame offsets (bit 0 = key frame). A frame packet is, per audio
//! track, `u32 size` + `size` bytes (`u32 decoded bytes` + blocks when non-zero), then the video data.

#[derive(Debug, thiserror::Error)]
pub enum BinkError {
    #[error("not a Bink file")]
    NotBink,
    #[error("the file doesn't contain any frames")]
    NoFrames,
    #[error("truncated Bink file")]
    Truncated,
}

#[derive(Clone, Copy, Debug)]
pub struct BinkTrack {
    pub max_size: u32,
    pub rate: u32,
    /// high half of the rate dword (0x10000000 DCT, 0x20000000 stereo)
    pub flags: u32,
    pub id: u32,
}

impl BinkTrack {
    pub fn channels(&self) -> u32 {
        ((self.flags & 0x2000_0000) != 0) as u32 + 1
    }

    /// BinkOpen's `v59 >= 0 && (v59 & 0x10000000)`: DCT (else RDFT) audio
    pub fn dct(&self) -> bool {
        (self.flags as i32) >= 0 && (self.flags & 0x1000_0000) != 0
    }
}

pub struct BinkFile {
    pub data: Vec<u8>,
    pub signature: [u8; 4],
    pub frames: u32,
    pub largest_frame: u32,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    /// header flags (`bink+36`)
    pub header_flags: u32,
    /// decode flags (`bink+32`): open flags merged with the header's (BIKf/g |= 0x18000, BIKh |= 0x8000)
    pub flags: u32,
    pub tracks: Vec<BinkTrack>,
    pub index: Vec<u32>,
}

/// one frame's audio blocks (per track) and video bitstream
pub struct BinkPacket<'a> {
    pub audio: Vec<&'a [u8]>,
    pub video: &'a [u8],
    pub key: bool,
}

impl BinkFile {
    /// BinkOpen 0x0053E9A0 (header part; `open_flags` = the caller's flags, VsMovie passes 0 / 0x84000)
    pub fn parse(data: Vec<u8>, open_flags: u32) -> Result<Self, BinkError> {
        let u = |o: usize| data.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
        let h: Vec<u32> = (0..11).map(|i| u(4 * i)).collect::<Option<_>>().ok_or(BinkError::Truncated)?;
        let sig = h[0].to_le_bytes();
        if &sig[0..3] != b"BIK" || !(b'f'..=b'i').contains(&sig[3]) {
            return Err(BinkError::NotBink);
        }
        if h[2] == 0 {
            return Err(BinkError::NoFrames);
        }
        let mut flags = open_flags & 0x8FFF_FFFF | h[9] & 0x20000;
        if h[9] & 0x10_0000 == 0 {
            flags = open_flags & 0x8FEF_FFFF | h[9] & 0x20000;
        }
        let scale = open_flags & 0x7000_0000;
        if scale != 0x7000_0000 {
            flags |= if scale != 0 { scale } else { h[9] & 0x7000_0000 };
        }
        match sig[3] {
            b'f' | b'g' => flags |= 0x18000,
            b'h' => flags |= 0x8000,
            _ => {}
        }
        let n = h[10] as usize;
        let mut tracks = Vec::with_capacity(n);
        for t in 0..n {
            let max_size = u(44 + 4 * t).ok_or(BinkError::Truncated)?;
            let rf = u(44 + 4 * (n + t)).ok_or(BinkError::Truncated)?;
            let id = u(44 + 4 * (2 * n + t)).ok_or(BinkError::Truncated)?;
            tracks.push(BinkTrack { max_size, rate: rf & 0xFFFF, flags: rf & 0xFFFF_0000, id });
        }
        let at = 44 + 12 * n;
        let index = (0..=h[4] as usize).map(|i| u(at + 4 * i)).collect::<Option<Vec<_>>>().ok_or(BinkError::Truncated)?;
        Ok(Self {
            signature: sig,
            frames: h[2],
            largest_frame: h[3],
            width: h[5],
            height: h[6],
            fps_num: h[7],
            fps_den: h[8],
            header_flags: h[9],
            flags,
            tracks,
            index,
            data,
        })
    }

    /// frame `i` (0-based) split into its audio and video parts (BinkDoFrame 0x00541340)
    pub fn packet(&self, i: usize) -> Result<BinkPacket<'_>, BinkError> {
        let start = (*self.index.get(i).ok_or(BinkError::Truncated)? & !1) as usize;
        let end = (*self.index.get(i + 1).ok_or(BinkError::Truncated)? & !1) as usize;
        let frame = self.data.get(start..end).ok_or(BinkError::Truncated)?;
        let mut at = 0usize;
        let mut audio = Vec::with_capacity(self.tracks.len());
        for _ in &self.tracks {
            let size = frame.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or(BinkError::Truncated)?
                as usize;
            audio.push(frame.get(at + 4..at + 4 + size).ok_or(BinkError::Truncated)?);
            at += 4 + size;
        }
        Ok(BinkPacket { audio, video: frame.get(at..).ok_or(BinkError::Truncated)?, key: self.index[i] & 1 != 0 })
    }

    /// 0x0053E6E0 `a5`: every index entry is a key frame -> one frame buffer (0x0053E8D0)
    pub fn all_key_frames(&self) -> bool {
        self.index[..self.frames as usize].iter().all(|&o| o & 1 != 0)
    }
}
