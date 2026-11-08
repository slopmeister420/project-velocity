//! platform layer: replaces `D3DXCreateTextureFromFileInMemoryEx` for the `.tga` files the game loads
//! (colour-mapped and true-colour, raw or RLE, either origin).

use crate::renderer::texture_manager::DecodedTexture;

pub fn decode_tga(d: &[u8]) -> Option<DecodedTexture> {
    let id_len = *d.first()? as usize;
    let cmap_type = *d.get(1)?;
    let img_type = *d.get(2)?;
    let rd16 = |o: usize| d.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let cmap_first = rd16(3)? as usize;
    let cmap_len = rd16(5)? as usize;
    let cmap_bits = *d.get(7)? as usize;
    let w = rd16(12)? as usize;
    let h = rd16(14)? as usize;
    let bpp = *d.get(16)? as usize;
    let desc = *d.get(17)?;
    let mut p = 18 + id_len;
    let mut pal = Vec::new();
    if cmap_type == 1 {
        let eb = cmap_bits.div_ceil(8);
        for i in 0..cmap_len {
            pal.push(texel(d.get(p + i * eb..p + i * eb + eb)?, cmap_bits)?);
        }
        p += cmap_len * eb;
    }
    let pb = bpp.div_ceil(8);
    let n = w * h;
    let mut raw: Vec<[u8; 4]> = Vec::with_capacity(n);
    let get = |s: &[u8]| -> Option<[u8; 4]> {
        if img_type & 7 == 1 {
            let i = if pb == 1 { s[0] as usize } else { u16::from_le_bytes([s[0], s[1]]) as usize };
            pal.get(i.checked_sub(cmap_first)?).copied()
        } else if img_type & 7 == 3 {
            Some([s[0], s[0], s[0], 255])
        } else {
            texel(s, bpp)
        }
    };
    if img_type & 8 != 0 {
        while raw.len() < n {
            let c = *d.get(p)?;
            p += 1;
            let cnt = (c & 0x7F) as usize + 1;
            if c & 0x80 != 0 {
                let t = get(d.get(p..p + pb)?)?;
                p += pb;
                raw.extend(std::iter::repeat_n(t, cnt));
            } else {
                for _ in 0..cnt {
                    raw.push(get(d.get(p..p + pb)?)?);
                    p += pb;
                }
            }
        }
        raw.truncate(n);
    } else {
        for i in 0..n {
            raw.push(get(d.get(p + i * pb..p + i * pb + pb)?)?);
        }
    }
    let top = desc & 0x20 != 0;
    let right = desc & 0x10 != 0;
    let mut out = vec![0u8; n * 4];
    for y in 0..h {
        let sy = if top { y } else { h - 1 - y };
        for x in 0..w {
            let sx = if right { w - 1 - x } else { x };
            out[(y * w + x) * 4..][..4].copy_from_slice(&raw[sy * w + sx]);
        }
    }
    Some(DecodedTexture { width: w as u32, height: h as u32, levels: vec![out] })
}

fn texel(s: &[u8], bits: usize) -> Option<[u8; 4]> {
    Some(match bits {
        32 => [s[2], s[1], s[0], s[3]],
        24 => [s[2], s[1], s[0], 255],
        15 | 16 => {
            let v = u16::from_le_bytes([*s.first()?, *s.get(1)?]);
            let c = |x: u16| ((x & 31) as u8) << 3 | ((x & 31) as u8) >> 2;
            [c(v >> 10), c(v >> 5), c(v), if bits == 16 && v & 0x8000 == 0 { 0 } else { 255 }]
        }
        8 => [s[0], s[0], s[0], 255],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every .tga, loose or inside a VCD, decodes.
    #[test]
    fn decode_every_tga() {
        fn walk(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(p).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out)
                } else {
                    out.push(p)
                }
            }
        }
        let mut files = Vec::new();
        walk(visos::file::game_dir(), &mut files);
        let (mut n, mut bad) = (0, Vec::new());
        for p in files {
            let ext = p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if ext == "tga" {
                n += 1;
                if decode_tga(&std::fs::read(&p).unwrap()).is_none() {
                    bad.push(p.display().to_string());
                }
            } else if ext == "vcd" {
                let Ok(v) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
                for e in v.entries() {
                    if e.name.to_ascii_lowercase().ends_with(".tga") {
                        n += 1;
                        if decode_tga(v.data(&e).unwrap()).is_none() {
                            bad.push(format!("{}:{}", p.display(), e.name));
                        }
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
        assert!(n > 150);
        eprintln!("decoded {n} tga");
    }
}
