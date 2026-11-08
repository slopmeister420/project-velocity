//! platform layer: replaces the NV2A's DXT samplers. Xbox DXT surfaces are linear (not swizzled) BC1/BC2/BC3 blocks;
//! a level smaller than 4x4 still occupies one block.

fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 31) as u32;
    let g = ((c >> 5) & 63) as u32;
    let b = (c & 31) as u32;
    [((r << 3) | (r >> 2)) as u8, ((g << 2) | (g >> 4)) as u8, ((b << 3) | (b >> 2)) as u8]
}

/// colour block (8 bytes) into 16 rgba texels. `dxt1` enables the 3-colour + transparent mode.
fn colour_block(b: &[u8], dxt1: bool, out: &mut [[u8; 4]; 16]) {
    let c0 = u16::from_le_bytes([b[0], b[1]]);
    let c1 = u16::from_le_bytes([b[2], b[3]]);
    let (p0, p1) = (rgb565(c0), rgb565(c1));
    let mut pal = [[0u8; 4]; 4];
    pal[0] = [p0[0], p0[1], p0[2], 255];
    pal[1] = [p1[0], p1[1], p1[2], 255];
    if c0 > c1 || !dxt1 {
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
    let bits = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
    for (i, t) in out.iter_mut().enumerate() {
        *t = pal[((bits >> (2 * i)) & 3) as usize];
    }
}

/// decode one level of format `d3dfmt` (0x0C DXT1, 0x0E DXT2/3, 0x0F DXT4/5).
/// returns rgba8 pixels and the number of source bytes consumed.
pub fn decode(d3dfmt: u32, w: u32, h: u32, src: &[u8]) -> Option<(Vec<u8>, usize)> {
    let bsize = match d3dfmt {
        0x0C => 8,
        0x0E | 0x0F => 16,
        _ => return None,
    };
    let (bw, bh) = (w.div_ceil(4).max(1), h.div_ceil(4).max(1));
    let need = (bw * bh) as usize * bsize;
    let src = src.get(..need)?;
    let mut out = vec![0u8; (w * h * 4) as usize];
    let mut texels = [[0u8; 4]; 16];
    for by in 0..bh {
        for bx in 0..bw {
            let b = &src[((by * bw + bx) as usize) * bsize..][..bsize];
            match d3dfmt {
                0x0C => colour_block(b, true, &mut texels),
                0x0E => {
                    colour_block(&b[8..], false, &mut texels);
                    for (i, t) in texels.iter_mut().enumerate() {
                        let a = (b[i / 2] >> ((i & 1) * 4)) & 15;
                        t[3] = a * 17;
                    }
                }
                _ => {
                    colour_block(&b[8..], false, &mut texels);
                    let (a0, a1) = (b[0] as u32, b[1] as u32);
                    let mut ap = [0u32; 8];
                    ap[0] = a0;
                    ap[1] = a1;
                    if a0 > a1 {
                        for k in 1..7 {
                            ap[k + 1] = ((7 - k as u32) * a0 + k as u32 * a1) / 7;
                        }
                    } else {
                        for k in 1..5 {
                            ap[k + 1] = ((5 - k as u32) * a0 + k as u32 * a1) / 5;
                        }
                        ap[6] = 0;
                        ap[7] = 255;
                    }
                    let mut bits = 0u64;
                    for k in 0..6 {
                        bits |= (b[2 + k] as u64) << (8 * k);
                    }
                    for (i, t) in texels.iter_mut().enumerate() {
                        t[3] = ap[((bits >> (3 * i)) & 7) as usize] as u8;
                    }
                }
            }
            for ty in 0..4 {
                for tx in 0..4 {
                    let (x, y) = (bx * 4 + tx, by * 4 + ty);
                    if x < w && y < h {
                        let o = ((y * w + x) * 4) as usize;
                        out[o..o + 4].copy_from_slice(&texels[(ty * 4 + tx) as usize]);
                    }
                }
            }
        }
    }
    Some((out, need))
}
