//! minimal `.vtf` (`VTP_`) texture reader for the frontend until visos_render publishes its loader.
//!
//! ```text
//! +0x00 'VTP_' | +0x04 u16 version (1) | +0x06 u16 mip count | +0x08 u16 width | +0x0A u16 height
//! +0x0C u16 width | +0x0E u16 height | +0x10 u32 format | +0x14 u32 format | +0x18 u32 | +0x1C u32 data offset
//! ```
//! format 13 = DXT5 (all FE/OSD/font pages), format 2 = 8-bit swizzled (one OSD page).

#[derive(Debug, Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    /// straight (non-premultiplied) rgba8, row major.
    pub data: Vec<u8>,
}

fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 31) as u32;
    let g = ((c >> 5) & 63) as u32;
    let b = (c & 31) as u32;
    [((r * 527 + 23) >> 6) as u8, ((g * 259 + 33) >> 6) as u8, ((b * 527 + 23) >> 6) as u8]
}

fn colour_block(b: &[u8], out: &mut [[u8; 4]; 16], dxt1: bool) {
    let c0 = u16::from_le_bytes([b[0], b[1]]);
    let c1 = u16::from_le_bytes([b[2], b[3]]);
    let p0 = rgb565(c0);
    let p1 = rgb565(c1);
    let mut pal = [[p0[0], p0[1], p0[2], 255], [p1[0], p1[1], p1[2], 255], [0; 4], [0; 4]];
    if !dxt1 || c0 > c1 {
        for k in 0..3 {
            pal[2][k] = ((2 * p0[k] as u32 + p1[k] as u32) / 3) as u8;
            pal[3][k] = ((p0[k] as u32 + 2 * p1[k] as u32) / 3) as u8;
        }
        pal[2][3] = 255;
        pal[3][3] = 255;
    } else {
        for k in 0..3 {
            pal[2][k] = ((p0[k] as u32 + p1[k] as u32) / 2) as u8;
        }
        pal[2][3] = 255;
        pal[3] = [0, 0, 0, 0];
    }
    let idx = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
    for (i, px) in out.iter_mut().enumerate() {
        let a = px[3];
        *px = pal[((idx >> (2 * i)) & 3) as usize];
        if !dxt1 {
            px[3] = a;
        }
    }
}

fn dxt5_alpha(b: &[u8], out: &mut [[u8; 4]; 16]) {
    let a0 = b[0] as u32;
    let a1 = b[1] as u32;
    let mut pal = [0u32; 8];
    pal[0] = a0;
    pal[1] = a1;
    if a0 > a1 {
        for i in 1..7 {
            pal[i + 1] = ((7 - i as u32) * a0 + i as u32 * a1) / 7;
        }
    } else {
        for i in 1..5 {
            pal[i + 1] = ((5 - i as u32) * a0 + i as u32 * a1) / 5;
        }
        pal[6] = 0;
        pal[7] = 255;
    }
    let mut bits = 0u64;
    for i in 0..6 {
        bits |= (b[2 + i] as u64) << (8 * i);
    }
    for (i, px) in out.iter_mut().enumerate() {
        px[3] = pal[((bits >> (3 * i)) & 7) as usize] as u8;
    }
}

/// xbox swizzle (morton order) offset of texel (x, y) in a w*h power-of-two image.
fn swizzle(mut x: u32, mut y: u32, w: u32, h: u32) -> usize {
    let mut off = 0usize;
    let mut bit = 0;
    let (mut mw, mut mh) = (w, h);
    while mw > 1 || mh > 1 {
        if mw > 1 {
            off |= ((x & 1) as usize) << bit;
            bit += 1;
            x >>= 1;
            mw >>= 1;
        }
        if mh > 1 {
            off |= ((y & 1) as usize) << bit;
            bit += 1;
            y >>= 1;
            mh >>= 1;
        }
    }
    off
}

/// decode the top mip of a `.vtf`.
pub fn decode(b: &[u8]) -> Option<Rgba> {
    if b.get(0..4)? != b"VTP_" {
        return None;
    }
    let r16 = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as u32;
    let r32 = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    let w = r16(8);
    let h = r16(10);
    let fmt = r32(0x10);
    let off = r32(0x1C) as usize;
    let mut data = vec![0u8; (w * h * 4) as usize];
    match fmt {
        13 => {
            let bw = w.div_ceil(4);
            let bh = h.div_ceil(4);
            for by in 0..bh {
                for bx in 0..bw {
                    let o = off + 16 * (by * bw + bx) as usize;
                    let blk = b.get(o..o + 16)?;
                    let mut px = [[0u8; 4]; 16];
                    dxt5_alpha(&blk[0..8], &mut px);
                    colour_block(&blk[8..16], &mut px, false);
                    for (i, p) in px.iter().enumerate() {
                        let x = bx * 4 + (i as u32 & 3);
                        let y = by * 4 + (i as u32 >> 2);
                        if x < w && y < h {
                            let d = ((y * w + x) * 4) as usize;
                            data[d..d + 4].copy_from_slice(p);
                        }
                    }
                }
            }
        }
        2 => {
            for y in 0..h {
                for x in 0..w {
                    let v = *b.get(off + swizzle(x, y, w, h))?;
                    let d = ((y * w + x) * 4) as usize;
                    data[d..d + 4].copy_from_slice(&[255, 255, 255, v]);
                }
            }
        }
        _ => return None,
    }
    Some(Rgba { width: w, height: h, data })
}

/// write rgba8 as png (tests / offscreen captures).
pub fn write_png(path: &std::path::Path, img: &Rgba) -> std::io::Result<()> {
    let f = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), img.width, img.height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(std::io::Error::other)?;
    w.write_image_data(&img.data).map_err(std::io::Error::other)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_every_vtf() {
        let dir = visos::file::game_dir();
        if !dir.exists() {
            return;
        }
        let mut n = 0;
        for p in ["\\FE\\FE.VCD", "\\Fonts\\FONTS.VCD", "\\OSD\\OSDT.VCD", "\\FE\\LOADSCR.VCD", "\\FE\\FETRACKS.VCD", "\\FE\\FECustom.VCD", "\\FE\\WORLDMAP.VCD", "\\FE\\feflags.vcd"] {
            let v = visos::file::Vcd::open(p).unwrap();
            for e in v.entries() {
                if e.name.to_ascii_lowercase().ends_with(".vtf") {
                    let img = decode(v.data(&e).unwrap()).unwrap_or_else(|| panic!("{} {}", p, e.name));
                    assert!(img.width > 0 && img.height > 0);
                    n += 1;
                }
            }
        }
        assert!(n > 300);
    }
}
