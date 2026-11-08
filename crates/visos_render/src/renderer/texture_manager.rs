//! Renderer\source\Xbox\TextureManager.cpp — `CTextureManager`: `.vtf` (VTP_) texture files to device textures.
//!
//! The xbox build hands DXT data straight to `XGSetTextureHeader` and converts palettised / 32-bit data with
//! `D3DXLoadSurfaceFromMemory`. Here the D3D layer is replaced by decoding to RGBA8 (`DecodedTexture`); everything
//! above it (header parsing, format switch, palette fix-up, row flip, level counts) follows the original.

use crate::renderer::xbox_dxt;

/// `'VTP_'`
pub const VTF_MAGIC: u32 = 0x5F50_5456;
/// trailer written after the header by the tool (`0xD0FEC0FF`).
pub const VTF_HEADER_END: u32 = 0xD0FE_C0FF;

/// `VT_*` texture formats (`VtfHeader::m_format`, +0x14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VtfFormat {
    /// no pixel data uploaded
    None = 0,
    P4 = 1,
    P8 = 2,
    Argb8888 = 7,
    Dxt1 = 9,
    Dxt2 = 10,
    Dxt3 = 11,
    Dxt4 = 12,
    Dxt5 = 13,
}

impl VtfFormat {
    pub fn from_u32(v: u32) -> Option<Self> {
        Some(match v {
            0 => Self::None,
            1 => Self::P4,
            2 => Self::P8,
            7 => Self::Argb8888,
            9 => Self::Dxt1,
            10 => Self::Dxt2,
            11 => Self::Dxt3,
            12 => Self::Dxt4,
            13 => Self::Dxt5,
            _ => return None,
        })
    }
}

/// on-disk header of a version 1 `.vtf` (48 bytes, then the zeroed D3D texture header at `m_pD3DHeader`).
#[derive(Clone, Debug)]
pub struct VtfHeader {
    pub magic: u32,          // +0x00
    pub version: u16,        // +0x04
    pub mip_levels: u8,      // +0x06
    pub clut_entries: u8,    // +0x07 255 means 256
    pub width: u16,          // +0x08
    pub height: u16,         // +0x0A
    pub width2: u16,         // +0x0C
    pub height2: u16,        // +0x0E
    pub format_src: u32,     // +0x10
    pub format: u32,         // +0x14
    pub clut_format: u32,    // +0x18 0 none, 1 rgba8888
    pub data_offset: u32,    // +0x1C m_pData (relative until fixed up)
    pub clut_offset: u32,    // +0x20 m_pClut
    pub d3d_header: u32,     // +0x24 m_pD3DHeader
}

/// pre-release (version 0) header found in a handful of files: the shipping loader cannot read these
/// (its `m_format` read lands on the data offset and hits the "Invalid texture format" assert).
#[derive(Clone, Debug)]
pub struct VtfHeaderV0 {
    pub width: u16,       // +0x06
    pub height: u16,      // +0x08
    pub mip_levels: u8,   // +0x0A
    pub clut_entries: u8, // +0x0B
    pub format: u32,      // +0x0C
    pub clut_format: u32, // +0x10
    pub data_offset: u32, // +0x14
    pub clut_offset: u32, // +0x18
}

/// a decoded texture: rgba8 levels, largest first.
#[derive(Clone, Debug)]
pub struct DecodedTexture {
    pub width: u32,
    pub height: u32,
    /// rgba8, one entry per mip level
    pub levels: Vec<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum VtfError {
    #[error("not a vtf (magic {0:08x})")]
    BadMagic(u32),
    #[error("truncated vtf")]
    Truncated,
    #[error("unsupported vtf format {0}")]
    Unsupported(u32),
    #[error("unsupported vtf version {0}")]
    Version(u16),
}

fn rd16(b: &[u8], o: usize) -> Result<u16, VtfError> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or(VtfError::Truncated)
}
fn rd32(b: &[u8], o: usize) -> Result<u32, VtfError> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(VtfError::Truncated)
}

impl VtfHeader {
    pub fn parse(b: &[u8]) -> Result<Self, VtfError> {
        let magic = rd32(b, 0)?;
        if magic != VTF_MAGIC {
            return Err(VtfError::BadMagic(magic));
        }
        Ok(Self {
            magic,
            version: rd16(b, 4)?,
            mip_levels: *b.get(6).ok_or(VtfError::Truncated)?,
            clut_entries: *b.get(7).ok_or(VtfError::Truncated)?,
            width: rd16(b, 8)?,
            height: rd16(b, 10)?,
            width2: rd16(b, 12)?,
            height2: rd16(b, 14)?,
            format_src: rd32(b, 16)?,
            format: rd32(b, 20)?,
            clut_format: rd32(b, 24)?,
            data_offset: rd32(b, 28)?,
            clut_offset: rd32(b, 32)?,
            d3d_header: rd32(b, 36)?,
        })
    }
}

impl VtfHeaderV0 {
    pub fn parse(b: &[u8]) -> Result<Self, VtfError> {
        Ok(Self {
            width: rd16(b, 6)?,
            height: rd16(b, 8)?,
            mip_levels: *b.get(10).ok_or(VtfError::Truncated)?,
            clut_entries: *b.get(11).ok_or(VtfError::Truncated)?,
            format: rd32(b, 12)?,
            clut_format: rd32(b, 16)?,
            data_offset: rd32(b, 20)?,
            clut_offset: rd32(b, 24)?,
        })
    }
}

/// D3D format chosen for the compressed formats (`D3DFMT_DXT1` 0x0C, `DXT2/3` 0x0E, `DXT4/5` 0x0F).
fn d3d_format(format: u32) -> u32 {
    match format {
        9 => 12,
        10 | 11 => 14,
        12 | 13 => 15,
        _ => u32::MAX,
    }
}

/// TextureManager.cpp 0x0029E3F0 — build a texture from an in-memory `.vtf`.
pub fn create_from_vtf(file: &[u8]) -> Result<DecodedTexture, VtfError> {
    let h = VtfHeader::parse(file)?;
    if h.version != 1 {
        return Err(VtfError::Version(h.version));
    }
    let data = h.data_offset as usize;
    match h.format {
        9..=13 => {
            if h.d3d_header == 0 {
                log::error!("pVtfHeader->m_pD3DHeader != 0");
            }
            // XGSetTextureHeader(width, height, levels, 0, fmt, 0, header, 0, 0) then Register(data)
            let fmt = d3d_format(h.format);
            let (w, hgt) = (h.width as u32, h.height as u32);
            let mut levels = Vec::new();
            let mut src = file.get(data..).ok_or(VtfError::Truncated)?;
            for l in 0..h.mip_levels.max(1) as u32 {
                let (lw, lh) = ((w >> l).max(1), (hgt >> l).max(1));
                let (rgba, used) = xbox_dxt::decode(fmt, lw, lh, src).ok_or(VtfError::Truncated)?;
                levels.push(rgba);
                src = &src[used..];
            }
            Ok(DecodedTexture { width: w, height: hgt, levels })
        }
        _ => {
            let clut_entries: usize = if h.clut_entries == 255 { 256 } else { h.clut_entries as usize };
            let _clut_bytes = match h.clut_format {
                0 => 0,
                1 => clut_entries * 4,
                _ => {
                    log::error!("!\"VT_CLUT_ARGB8888 not supported by the PS2\"");
                    0
                }
            };
            let (w, hgt) = (h.width as usize, h.height as usize);
            let size = match h.format {
                0 => 0,
                1 => {
                    log::error!("!\"VT_P4 Texture format not currently supported\"");
                    return Err(VtfError::Unsupported(1));
                }
                2 => hgt * w,
                7 => 4 * hgt * w,
                f => {
                    log::error!("!\"Invalid texture format\"");
                    return Err(VtfError::Unsupported(f));
                }
            };
            // CreateTexture(w, h, 1, 0, D3DFMT_A8R8G8B8, 0): one level only, the file's extra mips are ignored
            let mut out = vec![0u8; w * hgt * 4];
            if h.format == 2 {
                let clut = file.get(h.clut_offset as usize..).ok_or(VtfError::Truncated)?;
                let pal = convert_clut(clut, clut_entries)?;
                let src = file.get(data..data + size).ok_or(VtfError::Truncated)?;
                // rows are stored bottom-up: copy from the last row backwards
                for (y, row) in src.chunks_exact(w).rev().enumerate() {
                    for (x, &i) in row.iter().enumerate() {
                        let o = (y * w + x) * 4;
                        out[o..o + 4].copy_from_slice(&pal[i as usize]);
                    }
                }
            } else if h.format == 7 {
                let src = file.get(data..data + size).ok_or(VtfError::Truncated)?;
                // source handed to D3DX as D3DFMT_A8R8G8B8 (b,g,r,a in memory)
                for (y, row) in src.chunks_exact(4 * w).rev().enumerate() {
                    for (x, p) in row.chunks_exact(4).enumerate() {
                        let o = (y * w + x) * 4;
                        out[o..o + 4].copy_from_slice(&[p[2], p[1], p[0], p[3]]);
                    }
                }
            }
            Ok(DecodedTexture { width: w as u32, height: hgt as u32, levels: vec![out] })
        }
    }
}

/// TextureManager.cpp 0x0029E3F0 (palette loop) — ps2 style clut: rgb doubled (wrapping, as the u8 store does),
/// alpha copied as is. `PALETTEENTRY` order is r, g, b, flags(alpha).
fn convert_clut(clut: &[u8], entries: usize) -> Result<[[u8; 4]; 256], VtfError> {
    let mut pal = [[0u8; 4]; 256];
    for (i, p) in pal.iter_mut().enumerate().take(entries) {
        let s = clut.get(i * 4..i * 4 + 4).ok_or(VtfError::Truncated)?;
        *p = [s[0].wrapping_mul(2), s[1].wrapping_mul(2), s[2].wrapping_mul(2), s[3]];
    }
    Ok(pal)
}

/// decode the pre-release (version 0) layout. not something the game can load; kept so every file decodes.
pub fn decode_vtf_v0(file: &[u8]) -> Result<DecodedTexture, VtfError> {
    let h = VtfHeaderV0::parse(file)?;
    let (w, hgt) = (h.width as usize, h.height as usize);
    if h.format != 2 {
        return Err(VtfError::Unsupported(h.format));
    }
    let entries = if h.clut_entries == 255 { 256 } else { h.clut_entries as usize };
    let pal = convert_clut(file.get(h.clut_offset as usize..).ok_or(VtfError::Truncated)?, entries)?;
    let src = file.get(h.data_offset as usize..h.data_offset as usize + w * hgt).ok_or(VtfError::Truncated)?;
    let mut out = vec![0u8; w * hgt * 4];
    for (y, row) in src.chunks_exact(w).rev().enumerate() {
        for (x, &i) in row.iter().enumerate() {
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&pal[i as usize]);
        }
    }
    Ok(DecodedTexture { width: w as u32, height: hgt as u32, levels: vec![out] })
}

/// any `.vtf` the data set contains.
pub fn decode_any_vtf(file: &[u8]) -> Result<DecodedTexture, VtfError> {
    match rd16(file, 4)? {
        0 => decode_vtf_v0(file),
        _ => create_from_vtf(file),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use visos::file::Vcd;

    fn all_vcds() -> Vec<std::path::PathBuf> {
        fn walk(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            if let Ok(rd) = std::fs::read_dir(p) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(&p, out);
                    } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("vcd")) {
                        out.push(p);
                    }
                }
            }
        }
        let mut v = Vec::new();
        walk(visos::file::game_dir(), &mut v);
        v
    }

    /// every .vtf in every VCD of the install decodes, and DXT sizes match the file exactly.
    #[test]
    fn decode_every_vtf() {
        let (mut ok, mut v0) = (0, 0);
        let mut failures = Vec::new();
        for p in all_vcds() {
            let Ok(vcd) = Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            for e in vcd.entries() {
                if !e.name.to_ascii_lowercase().ends_with(".vtf") {
                    continue;
                }
                let d = vcd.data(&e).unwrap();
                match decode_any_vtf(d) {
                    Ok(t) => {
                        assert_eq!(t.levels[0].len(), (t.width * t.height * 4) as usize);
                        if rd16(d, 4).unwrap() == 0 {
                            v0 += 1;
                        }
                        ok += 1;
                    }
                    Err(err) => failures.push(format!("{}:{} {err}", p.display(), e.name)),
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
        assert!(ok > 3000, "only {ok} textures");
        eprintln!("decoded {ok} vtf ({v0} version 0)");
    }
}
