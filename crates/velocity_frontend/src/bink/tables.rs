//! Bink static tables (BINKDATA 0x007CEC20.., BINKBSS 0x007D2A80.., both initialised in the image).

use velocity_physics::material_properties::xbe_read;

/// one of the 16 huffman lookup tables `off_7D2920[i]`: `table[peek(maxlen)]` = code length << 4 | symbol slot
#[derive(Clone, Debug)]
pub struct HuffTable {
    pub maxlen: u32,
    pub table: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BinkTables {
    /// `off_7D2920` / `byte_7D2EF8`
    pub huff: Vec<HuffTable>,
    /// `byte_7D2EFC`, indexed by symbol (12..15 = run lengths 4/8/12/32)
    pub run_len: [u8; 16],
    /// `dword_7D2F50` (first colour) / `dword_7D2F10` (second colour) byte masks, one nibble -> 4 pixels
    pub pat8_a: [u32; 16],
    pub pat8_b: [u32; 16],
    /// `dword_7D2FA0` / `dword_7D2F90`: two bits -> 2 doubled pixels (scaled pattern)
    pub pat16_a: [u32; 4],
    pub pat16_b: [u32; 4],
    /// `unk_7D5188`: 16 run scan orders (pixel indices of an 8x8 block)
    pub scan: [[u8; 64]; 16],
    /// `dword_7D3100`: intra dequant, 16 x 64 (natural order)
    pub intra_q: [[i32; 64]; 16],
    /// `unk_7D4120`: inter dequant
    pub inter_q: [[i32; 64]; 16],
    /// `dword_7D2700`: audio band quantisers (96, f32 bits)
    pub audio_quant: [f32; 96],
    /// `flt_7D2880`: audio float exponent scale (32)
    pub audio_exp: [f32; 32],
    /// `dword_7D2698`: audio critical frequencies (25)
    pub audio_bands: [u32; 25],
    /// `byte_7D2688`: audio zero-run lengths (x 8 coefficients)
    pub audio_rle: [u8; 16],
}

fn u32s<const N: usize>(xbe: &[u8], va: u32) -> Option<[u32; N]> {
    let b = xbe_read(xbe, va, 4 * N)?;
    let mut out = [0u32; N];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().ok()?);
    }
    Some(out)
}

impl BinkTables {
    /// the tables at their debug-XBE addresses
    pub fn from_xbe(xbe: &[u8]) -> Option<Self> {
        let ptrs = u32s::<16>(xbe, 0x007D_2920)?;
        let lens = xbe_read(xbe, 0x007D_2EF8, 16)?;
        let mut huff = Vec::with_capacity(16);
        for i in 0..16 {
            let maxlen = lens[i] as u32;
            huff.push(HuffTable { maxlen, table: xbe_read(xbe, ptrs[i], 1 << maxlen)?.to_vec() });
        }
        let mut run_len = [0u8; 16];
        run_len.copy_from_slice(xbe_read(xbe, 0x007D_2EFC, 16)?);
        let mut scan = [[0u8; 64]; 16];
        let s = xbe_read(xbe, 0x007D_5188, 1024)?;
        for (i, row) in scan.iter_mut().enumerate() {
            row.copy_from_slice(&s[64 * i..64 * i + 64]);
        }
        let q = |va: u32| -> Option<[[i32; 64]; 16]> {
            let mut t = [[0i32; 64]; 16];
            for (i, row) in t.iter_mut().enumerate() {
                let r = u32s::<64>(xbe, va + 256 * i as u32)?;
                for (o, v) in row.iter_mut().zip(r) {
                    *o = v as i32;
                }
            }
            Some(t)
        };
        Some(Self {
            huff,
            run_len,
            pat8_a: u32s(xbe, 0x007D_2F50)?,
            pat8_b: u32s(xbe, 0x007D_2F10)?,
            pat16_a: u32s(xbe, 0x007D_2FA0)?,
            pat16_b: u32s(xbe, 0x007D_2F90)?,
            scan,
            intra_q: q(0x007D_3100)?,
            inter_q: q(0x007D_4120)?,
            audio_quant: u32s::<96>(xbe, 0x007D_2700)?.map(f32::from_bits),
            audio_exp: u32s::<32>(xbe, 0x007D_2880)?.map(f32::from_bits),
            audio_bands: u32s(xbe, 0x007D_2698)?,
            audio_rle: xbe_read(xbe, 0x007D_2688, 16)?.try_into().ok()?,
        })
    }

    /// from the install's `default.xbe`
    pub fn load() -> Option<Self> {
        let xbe = visos::file::platform::read(r"\default.xbe").ok()?;
        Self::from_xbe(&xbe)
    }
}
