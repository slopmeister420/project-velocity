//! Bink 1 video: frame buffers (0x0053E8D0), the per-frame plane split (0x00547B60) and the plane decoder
//! (0x00545E20) with its bundle readers, DCT coefficient / residue readers and IDCTs.
//!
//! Planes are 8-bit, width / height rounded up to 8 (chroma = half, rounded up, then to 8), stride = width.
//! Out-of-plane motion reads return 0 (the original reads whatever memory is there); out-of-plane writes of a
//! scaled block on the last column wrap into the next row exactly as the original's linear addressing does.

use super::bink_file::BinkFile;
use super::bitstream::BitReader;
use super::tables::BinkTables;

/// natural 8x8 position -> Bink's internal coefficient / residue order (0x0054EF00 tail, 0x0054FD50)
const BINK_ORDER: [u8; 64] = [
    0, 1, 4, 5, 8, 9, 12, 13, //
    2, 3, 6, 7, 10, 11, 14, 15, //
    24, 25, 44, 45, 16, 17, 20, 21, //
    26, 27, 46, 47, 18, 19, 22, 23, //
    28, 29, 32, 33, 48, 49, 52, 53, //
    30, 31, 34, 35, 50, 51, 54, 55, //
    36, 37, 40, 41, 56, 57, 60, 61, //
    38, 39, 42, 43, 58, 59, 62, 63,
];

#[derive(Clone, Debug, Default)]
pub struct Plane {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Plane {
    fn new(width: usize, height: usize) -> Self {
        Self { data: vec![0; width * height], width, height }
    }
}

/// BINK32 0x00550FC0 per pixel (MMX words): `pmulhw` by `qword_7D2A30` (Y) / `7D2A38` (chroma), the per-row
/// biases `qword_7D29A0` R / `7D29B0` G / `7D29C0` B (0x00352260: equal on both rows, so no dither), `paddsw`,
/// `psubusw qword_7D2A40`, `psrlw 4`.
pub fn yuv_to_rgb_mmx(y: u8, u: u8, v: u8) -> [u8; 3] {
    const KY: i32 = 0x4A89;
    const KC: i32 = 0x6810;
    const SUB: i32 = 0x7000;
    let mulhi = |a: i32, k: i32| (a * k) >> 16;
    let y = mulhi((y as i32) << 6, KY);
    let u = mulhi((u as i32) << 4, KC);
    let v = mulhi((v as i32) << 5, KC);
    let px = |x: i32, bias: i32| {
        let s = (x as i16 as i32 + bias).clamp(-32768, 32767); // paddw wraps, paddsw saturates
        ((s - SUB).max(0) >> 4) as u8
    };
    let r = px(2 * v + y, 0x61D0);
    let g = px(y - (u + v), 0x78A7);
    let b = px(5 * u + y, 0x5E8D);
    [r, g, b]
}

#[derive(Clone, Debug, Default)]
pub struct FrameBuffer {
    pub y: Plane,
    pub u: Plane,
    pub v: Plane,
    pub a: Option<Plane>,
}

/// a huffman tree: one of the 16 lookup tables plus the symbol permutation (0x00544450)
#[derive(Clone, Copy, Debug, Default)]
struct Tree {
    table: usize,
    syms: [u8; 16],
}

impl Tree {
    fn get(&self, t: &BinkTables, br: &mut BitReader) -> u8 {
        let h = &t.huff[self.table];
        let e = h.table[br.peek(h.maxlen) as usize];
        br.skip((e >> 4) as u32);
        self.syms[(e & 15) as usize]
    }
}

/// 0x005443E0: merge two sorted runs of `n`, one bit per decision until a side runs out
fn merge(br: &mut BitReader, left: &[u8], right: &[u8], out: &mut [u8]) {
    let (mut l, mut r, mut o) = (0, 0, 0);
    loop {
        if br.bit() {
            out[o] = right[r];
            r += 1;
        } else {
            out[o] = left[l];
            l += 1;
        }
        o += 1;
        if l == left.len() {
            while r < right.len() {
                out[o] = right[r];
                r += 1;
                o += 1;
            }
            return;
        }
        if r == right.len() {
            while l < left.len() {
                out[o] = left[l];
                l += 1;
                o += 1;
            }
            return;
        }
    }
}

/// 0x00544450 read a tree
fn read_tree(br: &mut BitReader) -> Tree {
    let mut tree = Tree { table: br.read(4) as usize, syms: [0; 16] };
    if tree.table == 0 {
        for (i, s) in tree.syms.iter_mut().enumerate() {
            *s = i as u8;
        }
        return tree;
    }
    if !br.bit() {
        let depth = br.read(2);
        let mut cur = [0u8; 16];
        for i in (0..16).step_by(2) {
            if br.bit() {
                cur[i] = i as u8 + 1;
                cur[i + 1] = i as u8;
            } else {
                cur[i] = i as u8;
                cur[i + 1] = i as u8 + 1;
            }
        }
        let mut size = 2;
        for _ in 0..depth {
            let mut next = [0u8; 16];
            for g in (0..16).step_by(2 * size) {
                merge(br, &cur[g..g + size], &cur[g + size..g + 2 * size], &mut next[g..g + 2 * size]);
            }
            cur = next;
            size *= 2;
        }
        tree.syms = cur;
    } else {
        let n = br.read(3) as usize;
        let mut left: u32 = 0xFFFF;
        for k in 0..=n {
            let s = br.read(4) as u8;
            left &= !(1 << s);
            tree.syms[k] = s;
        }
        let mut at = n + 1;
        let mut s = 0u8;
        while left != 0 {
            if left & 1 != 0 && at < 16 {
                tree.syms[at] = s;
                at += 1;
            }
            left >>= 1;
            s += 1;
        }
    }
    tree
}

/// bundle count width (0x00545E20 setup: floor(log2(v)) + 1 for v >= 0x100, capped at 15)
fn count_bits(v: u32) -> u32 {
    if v < 0x800 {
        if v < 0x200 {
            9 - (v < 0x100) as u32
        } else {
            11 - (v < 0x400) as u32
        }
    } else if v < 0x2000 {
        13 - (v < 0x1000) as u32
    } else {
        15 - (v < 0x4000) as u32
    }
}

/// a byte bundle `{cur, end, .., syms, maxlen, table, count bits, buffer}`
struct Bundle {
    buf: Vec<u8>,
    cur: usize,
    end: usize,
    count_bits: u32,
    tree: Tree,
}

impl Bundle {
    fn new(size: usize, count_bits: u32) -> Self {
        Self { buf: vec![0; size], cur: 0, end: 0, count_bits, tree: Tree::default() }
    }

    fn next(&mut self) -> u8 {
        let v = self.buf.get(self.cur).copied().unwrap_or(0);
        self.cur += 1;
        v
    }

    /// common head of every reader: refill only when used up; a zero count ends the bundle (`cur = buf + 4`)
    fn start(&mut self, br: &mut BitReader) -> Option<usize> {
        if self.cur != self.end {
            return None;
        }
        let n = br.read(self.count_bits) as usize;
        if n == 0 {
            self.cur = 4;
            self.end = 0;
            return None;
        }
        self.cur = 0;
        self.end = n;
        Some(n)
    }

    fn put(&mut self, at: usize, v: u8) {
        if let Some(b) = self.buf.get_mut(at) {
            *b = v;
        }
    }

    /// 0x00544B80 block types: fill, or symbols with 12..15 = repeat the last one
    fn read_block_types(&mut self, t: &BinkTables, br: &mut BitReader) {
        let Some(n) = self.start(br) else { return };
        if br.bit() {
            let v = br.read(4) as u8;
            for i in 0..n {
                self.put(i, v);
            }
            return;
        }
        let (mut left, mut at, mut last) = (n as i64, 0usize, 0u8);
        while left > 0 {
            let s = self.tree.get(t, br);
            if s < 12 {
                self.put(at, s);
                at += 1;
                left -= 1;
                last = s;
            } else {
                let run = t.run_len[s as usize] as usize;
                left -= run as i64;
                for _ in 0..run {
                    self.put(at, last);
                    at += 1;
                }
            }
        }
    }

    /// 0x00545390 runs: fill, or plain symbols
    fn read_runs(&mut self, t: &BinkTables, br: &mut BitReader) {
        let Some(n) = self.start(br) else { return };
        if br.bit() {
            let v = br.read(4) as u8;
            for i in 0..n {
                self.put(i, v);
            }
            return;
        }
        for i in 0..n {
            let s = self.tree.get(t, br);
            self.put(i, s);
        }
    }

    /// 0x005455A0 patterns: two symbols per byte, low nibble first
    fn read_patterns(&mut self, t: &BinkTables, br: &mut BitReader) {
        let Some(n) = self.start(br) else { return };
        for i in 0..n {
            let lo = self.tree.get(t, br);
            let hi = self.tree.get(t, br);
            self.put(i, lo | hi << 4);
        }
    }

    /// 0x005457A0 motion offsets: signed 4-bit, fill or symbols
    fn read_motion(&mut self, t: &BinkTables, br: &mut BitReader) {
        let Some(n) = self.start(br) else { return };
        if br.bit() {
            let mut v = br.read(4) as i32;
            if v != 0 && br.bit() {
                v = -v;
            }
            for i in 0..n {
                self.put(i, v as u8);
            }
            return;
        }
        for i in 0..n {
            let mut v = self.tree.get(t, br) as i32;
            if v != 0 && br.bit() {
                v = -v;
            }
            self.put(i, v as u8);
        }
    }

    /// 0x005450D0 (and 0x00544E00 when flag 0x8000: signed colours) colours: high nibble through the
    /// context tree of the previous high nibble, low nibble through the bundle's own tree
    fn read_colors(&mut self, t: &BinkTables, br: &mut BitReader, ctx: &mut ColorCtx, signed: bool) {
        let Some(n) = self.start(br) else { return };
        let fill = br.bit();
        let count = if fill { 1 } else { n };
        for i in 0..count {
            let hi = ctx.trees[ctx.last as usize].get(t, br);
            let lo = self.tree.get(t, br);
            ctx.last = hi;
            let mut v = lo | hi << 4;
            if signed && v & 0x80 != 0 {
                v = (-((v & 0x7F) as i32)) as u8;
            }
            self.put(i, if signed { v.wrapping_add(0x80) } else { v });
        }
        if fill && n > 1 {
            let v = self.buf[0];
            for i in 0..n {
                self.put(i, v);
            }
        }
    }
}

/// the 16 high-nibble trees of the colour bundle and the last high nibble (0x00544B10 ctx, +384)
struct ColorCtx {
    trees: [Tree; 16],
    last: u8,
}

/// a DC bundle (`u16` elements, 11-bit start value, then 4-bit-width delta groups of 8)
struct DcBundle {
    buf: Vec<u16>,
    cur: usize,
    end: usize,
    count_bits: u32,
    signed: bool,
}

impl DcBundle {
    fn next(&mut self) -> u16 {
        let v = self.buf.get(self.cur).copied().unwrap_or(0);
        self.cur += 1;
        v
    }

    /// 0x00545A10
    fn read(&mut self, br: &mut BitReader) {
        if self.cur != self.end {
            return;
        }
        let n = br.read(self.count_bits) as usize;
        if n == 0 {
            self.cur = 4;
            self.end = 0;
            return;
        }
        let mut v: u16 = if self.signed {
            let mut v = br.read(10) as i32;
            if v != 0 && br.bit() {
                v = -v;
            }
            v as u16
        } else {
            br.read(11) as u16
        };
        let mut at = 0usize;
        let mut put = |b: &mut Vec<u16>, v: u16| {
            if let Some(o) = b.get_mut(at) {
                *o = v;
            }
            at += 1;
        };
        put(&mut self.buf, v);
        self.cur = 0;
        self.end = n;
        let mut left = n - 1;
        while left != 0 {
            let len = left.min(8);
            let bits = br.read(4);
            if bits != 0 {
                for _ in 0..len {
                    let mut d = br.read(bits) as i32;
                    if d != 0 && br.bit() {
                        d = -d;
                    }
                    v = v.wrapping_add(d as u16);
                    put(&mut self.buf, v);
                }
            } else {
                for _ in 0..len {
                    put(&mut self.buf, v);
                }
            }
            left -= len;
        }
    }
}

/// 0x0054EF00 DCT coefficients (`out[0]` = DC, set by the caller)
fn read_coefs(br: &mut BitReader, out: &mut [i16; 64]) {
    let mut c = [0i16; 64];
    let mut list = [0u8; 256];
    let (mut front, mut back) = (128usize, 134usize);
    list[128..134].copy_from_slice(&[16, 96, 0xB0, 7, 11, 15]);
    let nbits = br.read(4);
    if nbits > 1 {
        let mut bits = nbits;
        loop {
            let top = 1i32 << (bits - 1);
            let value = |br: &mut BitReader| -> i16 {
                let v = br.read(bits) as i32;
                let m = top | (v & (top - 1));
                (if v & top != 0 { -m } else { m }) as i16
            };
            let mut i = front;
            while i < back {
                let e = list[i];
                if e == 0 || !br.bit() {
                    i += 1;
                    continue;
                }
                let idx = (e >> 2) as usize;
                let mut four = |br: &mut BitReader, list: &mut [u8; 256], front: &mut usize| {
                    for ci in idx..idx + 4 {
                        if br.bit() {
                            *front -= 1;
                            list[*front] = (ci as u8) << 2 | 3;
                        } else if let Some(o) = c.get_mut(ci) {
                            *o = value(br);
                        } else {
                            value(br);
                        }
                    }
                };
                match e & 3 {
                    0 => {
                        list[i] = e.wrapping_add(17);
                        four(br, &mut list, &mut front);
                    }
                    1 => {
                        list[i] = e & !3 | 2;
                        for k in 1..4u8 {
                            list[back] = (e & !3).wrapping_add(16 * k) | 2;
                            back += 1;
                        }
                    }
                    2 => {
                        list[i] = 0;
                        i += 1;
                        four(br, &mut list, &mut front);
                    }
                    _ => {
                        let v = value(br);
                        if let Some(o) = c.get_mut(idx) {
                            *o = v;
                        }
                        list[i] = 0;
                        i += 1;
                    }
                }
            }
            bits -= 1;
            if bits == 1 {
                break;
            }
        }
    }
    if nbits != 0 {
        let sign = |br: &mut BitReader| -> i16 { if br.bit() { -1 } else { 1 } };
        let mut i = front;
        while i < back {
            let e = list[i];
            if e == 0 || !br.bit() {
                i += 1;
                continue;
            }
            let idx = (e >> 2) as usize;
            let mut four = |br: &mut BitReader, list: &mut [u8; 256], front: &mut usize| {
                for ci in idx..idx + 4 {
                    if br.bit() {
                        *front -= 1;
                        list[*front] = (ci as u8) << 2 | 3;
                    } else {
                        let v = sign(br);
                        if let Some(o) = c.get_mut(ci) {
                            *o = v;
                        }
                    }
                }
            };
            match e & 3 {
                0 => {
                    list[i] = e.wrapping_add(17);
                    four(br, &mut list, &mut front);
                }
                1 => {
                    list[i] = e & !3 | 2;
                    for k in 1..4u8 {
                        list[back] = (e & !3).wrapping_add(16 * k) | 2;
                        back += 1;
                    }
                }
                2 => {
                    list[i] = 0;
                    i += 1;
                    four(br, &mut list, &mut front);
                }
                _ => {
                    let v = sign(br);
                    if let Some(o) = c.get_mut(idx) {
                        *o = v;
                    }
                    list[i] = 0;
                    i += 1;
                }
            }
        }
    }
    for n in 1..64 {
        out[n] = c[BINK_ORDER[n] as usize];
    }
}

/// 0x0054F780 residue (internal order), at most `max + 1` mask steps
fn read_residue(br: &mut BitReader, res: &mut [i8; 64], max: u32) {
    let mut list = [0u8; 256];
    let (mut front, mut back) = (128usize, 132usize);
    list[128..132].copy_from_slice(&[16, 96, 0xB0, 2]);
    let mut nz: Vec<u8> = Vec::with_capacity(64);
    let mut count = 0u32;
    let levels = br.read(3) + 1;
    let mut mask = 1i32 << (levels - 1);
    // returns true when the mask budget is spent
    let step = |count: &mut u32| -> bool {
        let done = *count == max;
        *count += 1;
        done
    };
    for _ in 0..levels {
        for j in 0..nz.len() {
            if br.bit() {
                let at = nz[j] as usize;
                let add = if res[at] < 0 { -mask } else { mask };
                res[at] = (res[at] as i32 + add) as i8;
                if step(&mut count) {
                    return;
                }
            }
        }
        let mut i = front;
        while i < back {
            let e = list[i];
            if e == 0 || !br.bit() {
                i += 1;
                continue;
            }
            let idx = (e >> 2) as usize;
            match e & 3 {
                0 | 2 => {
                    if e & 3 == 0 {
                        list[i] = e.wrapping_add(17);
                    } else {
                        list[i] = 0;
                        i += 1;
                    }
                    for ci in idx..idx + 4 {
                        if br.bit() {
                            front -= 1;
                            list[front] = (ci as u8) << 2 | 3;
                        } else {
                            nz.push(ci as u8);
                            let v = if br.bit() { -mask } else { mask };
                            if let Some(o) = res.get_mut(ci) {
                                *o = v as i8;
                            }
                            if step(&mut count) {
                                return;
                            }
                        }
                    }
                }
                1 => {
                    list[i] = e & !3 | 2;
                    for k in 1..4u8 {
                        list[back] = (e & !3).wrapping_add(16 * k) | 2;
                        back += 1;
                    }
                }
                _ => {
                    nz.push(idx as u8);
                    let v = if br.bit() { -mask } else { mask };
                    if let Some(o) = res.get_mut(idx) {
                        *o = v as i8;
                    }
                    if step(&mut count) {
                        return;
                    }
                    list[i] = 0;
                    i += 1;
                }
            }
        }
        mask >>= 1;
    }
}

/// the IDCT shared by 0x0054E3C0 (put), 0x0054E6F0 (scaled put) and 0x0054EB20 (add): dequantise
/// (`coef * q >> 11`), columns, then rows to `(u16)(x + 127) >> 8`
fn idct(coef: &[i16; 64], q: &[i32; 64]) -> [u8; 64] {
    let m = |a: i32, b: i32| a.wrapping_mul(b) >> 11;
    let mut t = [0i32; 64];
    for c in 0..8 {
        let r = |k: usize| coef[k * 8 + c] as i32;
        let qq = |k: usize| q[k * 8 + c];
        if (1..8).all(|k| coef[k * 8 + c] == 0) {
            let v = m(qq(0), r(0));
            for k in 0..8 {
                t[k * 8 + c] = v;
            }
            continue;
        }
        let v9 = m(qq(0), r(0));
        let v10 = m(qq(6), r(6));
        let v11 = m(qq(2), r(2));
        let v12 = m(qq(4), r(4));
        let v13 = v12.wrapping_add(v9);
        let v14 = v9.wrapping_sub(v12);
        let v15 = v10.wrapping_add(v11);
        let v16 = m(2896, v11.wrapping_sub(v10)).wrapping_sub(v15);
        let v58 = v13.wrapping_sub(v15);
        let v65 = v16.wrapping_add(v14);
        let v60 = v14.wrapping_sub(v16);
        let v62 = v15.wrapping_add(v13);
        let v17 = m(qq(1), r(1));
        let v50 = m(qq(7), r(7));
        let v18 = m(qq(3), r(3));
        let v19 = m(qq(5), r(5));
        let v54 = v19.wrapping_add(v18);
        let v20 = v19.wrapping_sub(v18);
        let v21 = v17.wrapping_add(v50);
        let v22 = v17.wrapping_sub(v50);
        let v51 = v21.wrapping_add(v54);
        let v23 = m(3784, v22.wrapping_add(v20));
        let v24 = v23.wrapping_add(m(-5352, v20)).wrapping_sub(v51);
        let v25 = m(2896, v21.wrapping_sub(v54)).wrapping_sub(v24);
        let v64 = v25.wrapping_add(m(2217, v22)).wrapping_sub(v23);
        t[c] = v62.wrapping_add(v51);
        t[56 + c] = v62.wrapping_sub(v51);
        t[8 + c] = v24.wrapping_add(v65);
        t[48 + c] = v65.wrapping_sub(v24);
        t[40 + c] = v60.wrapping_sub(v25);
        t[16 + c] = v25.wrapping_add(v60);
        t[32 + c] = v64.wrapping_add(v58);
        t[24 + c] = v58.wrapping_sub(v64);
    }
    let mut out = [0u8; 64];
    let px = |x: i32| ((x.wrapping_add(127) as u16) >> 8) as u8;
    for r in 0..8 {
        let s = &t[r * 8..r * 8 + 8];
        let e04 = s[0].wrapping_add(s[4]);
        let d04 = s[0].wrapping_sub(s[4]);
        let e26 = s[2].wrapping_add(s[6]);
        let o26 = m(2896, s[2].wrapping_sub(s[6])).wrapping_sub(e26);
        let a0 = e26.wrapping_add(e04);
        let a3 = e04.wrapping_sub(e26);
        let a1 = o26.wrapping_add(d04);
        let a2 = d04.wrapping_sub(o26);
        let e35 = s[3].wrapping_add(s[5]);
        let d53 = s[5].wrapping_sub(s[3]);
        let e17 = s[1].wrapping_add(s[7]);
        let d17 = s[1].wrapping_sub(s[7]);
        let b0 = e17.wrapping_add(e35);
        let v46 = m(3784, d17.wrapping_add(d53));
        let b1 = v46.wrapping_add(m(-5352, d53)).wrapping_sub(b0);
        let b2 = m(2896, e17.wrapping_sub(e35)).wrapping_sub(b1);
        let b3 = b2.wrapping_add(m(2217, d17)).wrapping_sub(v46);
        let o = &mut out[r * 8..r * 8 + 8];
        o[0] = px(b0.wrapping_add(a0));
        o[1] = px(b1.wrapping_add(a1));
        o[2] = px(a2.wrapping_add(b2));
        o[3] = px(a3.wrapping_sub(b3));
        o[4] = px(b3.wrapping_add(a3));
        o[5] = px(a2.wrapping_sub(b2));
        o[6] = px(a1.wrapping_sub(b1));
        o[7] = px(a0.wrapping_sub(b0));
    }
    out
}

/// plane access with the original's linear addressing
struct PlaneIo<'a> {
    cur: &'a mut [u8],
    prev: Option<&'a [u8]>,
    stride: usize,
}

impl PlaneIo<'_> {
    fn get_prev(&self, off: isize) -> u8 {
        if off < 0 {
            return 0;
        }
        match self.prev {
            Some(p) => p.get(off as usize).copied().unwrap_or(0),
            None => self.cur.get(off as usize).copied().unwrap_or(0),
        }
    }

    fn set(&mut self, off: usize, v: u8) {
        if let Some(p) = self.cur.get_mut(off) {
            *p = v;
        }
    }

    fn block8(&mut self, off: usize, px: &[u8; 64]) {
        for r in 0..8 {
            for c in 0..8 {
                self.set(off + r * self.stride + c, px[r * 8 + c]);
            }
        }
    }

    /// 8x8 doubled to 16x16
    fn block16(&mut self, off: usize, px: &[u8; 64]) {
        for r in 0..16 {
            for c in 0..16 {
                self.set(off + r * self.stride + c, px[(r / 2) * 8 + c / 2]);
            }
        }
    }

    fn prev8(&self, off: isize) -> [u8; 64] {
        let mut b = [0u8; 64];
        for r in 0..8 {
            for c in 0..8 {
                b[r * 8 + c] = self.get_prev(off + (r * self.stride + c) as isize);
            }
        }
        b
    }
}

/// run fill of 0x00545E20 case 3 / scaled 3: `scan` order, bit 1 = one colour repeated, 0 = literal colours
fn read_run_block(scan: &[u8; 64], colors: &mut Bundle, runs: &mut Bundle, br: &mut BitReader) -> [u8; 64] {
    let mut b = [0u8; 64];
    let mut i = 0usize;
    while i < 63 {
        if br.bit() {
            let v = colors.next();
            let n = runs.next() as usize + 1;
            for _ in 0..n {
                if i < 64 {
                    b[scan[i] as usize] = v;
                }
                i += 1;
            }
        } else {
            let n = runs.next() as usize + 1;
            for _ in 0..n {
                let v = colors.next();
                if i < 64 {
                    b[scan[i] as usize] = v;
                }
                i += 1;
            }
        }
    }
    if i == 63 {
        b[scan[63] as usize] = colors.next();
    }
    b
}

pub struct BinkVideo {
    pub width: u32,
    pub height: u32,
    pub flags: u32,
    pub header_flags: u32,
    pub buffers: Vec<FrameBuffer>,
    /// `a1[5]`: the buffer holding the newest frame
    pub current: usize,
    bundle_sizes: [usize; 9],
    /// last decode: (luma end, luma declared end, chroma end, video bytes) — the reader test's consistency check
    pub last_check: (usize, usize, usize, usize),
}

impl BinkVideo {
    /// the video part of BinkOpen (0x0053E8D0 buffers, 0x00545D80 bundle sizes)
    pub fn new(file: &BinkFile) -> Self {
        let (w, h) = (file.width as usize, file.height as usize);
        let yw = (w + 7) & !7;
        let yh = (h + 7) & !7;
        let cw = (((w + 1) >> 1) + 7) & !7;
        let ch = (((h + 1) >> 1) + 7) & !7;
        let alpha = file.flags & 0x10_0000 != 0;
        let n = if file.all_key_frames() { 1 } else { 2 };
        let buffers = (0..n)
            .map(|_| FrameBuffer {
                y: Plane::new(yw, yh),
                u: Plane::new(cw, ch),
                v: Plane::new(cw, ch),
                a: alpha.then(|| Plane::new(yw, yh)),
            })
            .collect();
        // bundle sizes from the (scaled) width rounded to 16
        let sw = {
            let mut sw = w;
            let s = file.flags & 0x7000_0000;
            if s == 0x3000_0000 || s == 0x4000_0000 || s == 0x5000_0000 {
                sw *= 2;
            }
            (sw + 15) & !15
        };
        let b = sw >> 3;
        let a = |v: usize| (v + 515) & !3;
        let dc = ((2 * b) + 1027) & !3;
        let bundle_sizes = [a(b), a(sw >> 4), a(b << 6), a(8 * b), a(b), a(b), dc, dc, a(48 * b)];
        Self { width: file.width, height: file.height, flags: file.flags, header_flags: file.header_flags, buffers, current: 0, bundle_sizes, last_check: (0, 0, 0, 0) }
    }

    /// the decoded frame
    pub fn frame(&self) -> &FrameBuffer {
        &self.buffers[self.current]
    }

    /// BinkCopyToBuffer 0x00541300 -> 0x0053F970 -> 0x00548DB0 (rows) with the BINKSURFACE32 row blitter
    /// 0x00550FC0 (installed by selector 0x00352260 for format 3: shifts R 16 / G 8 / B 0, 0x005484B0), as RGBA8.
    /// Chroma: one sample per pixel pair, row y >> 1 (0x00548980 strides). The top byte is `x`: the BINK32 blitter
    /// (surfaces 3 / 5) unpacks with `qword_7D2A48` = 0, never written, so the caller passes 0 there (255 only for
    /// the full-screen YUY2 stand-in); the alpha plane (never opened by VsMovie) would replace it.
    pub fn to_rgba(&self, out: &mut Vec<u8>, x: u8) {
        let f = self.frame();
        let (w, h) = (self.width as usize, self.height as usize);
        out.resize(w * h * 4, 0);
        // the first decoded chroma plane is Cr unless flag 0x10000 (BIKf/g) swaps them (0x00548980 `a12 & 0x10000`)
        let (cb, cr) = if self.flags & 0x10000 == 0 { (&f.v, &f.u) } else { (&f.u, &f.v) };
        // gray (0x20000): both chroma pointers at a 0x80 row (0x00548980 memset)
        let gray = self.flags & 0x20000 != 0;
        for y in 0..h {
            for px in 0..w {
                let (cx, cy) = (px >> 1, y >> 1);
                let (u, v) = if gray { (0x80, 0x80) } else { (cb.data[cy * cb.width + cx], cr.data[cy * cr.width + cx]) };
                let [r, g, b] = yuv_to_rgb_mmx(f.y.data[y * f.y.width + px], u, v);
                let o = (y * w + px) * 4;
                out[o] = r;
                out[o + 1] = g;
                out[o + 2] = b;
                out[o + 3] = f.a.as_ref().map(|a| a.data[y * a.width + px]).unwrap_or(x);
            }
        }
    }

    /// 0x00547B60: swap buffers, then alpha (when present), luma, chroma
    pub fn decode(&mut self, t: &BinkTables, video: &[u8]) {
        let old = self.current;
        if self.buffers.len() > 1 {
            self.current = old ^ 1;
        }
        let new = self.current;
        let mut at = 0usize;
        let u32_at = |o: usize| video.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).unwrap_or(0) as usize;
        if self.header_flags & 0x10_0000 != 0 {
            if self.flags & 0x10_0000 != 0 {
                self.decode_plane(t, video, at + 4, new, old, 3);
            }
            at += u32_at(at);
        }
        if self.flags & 0x8000 == 0 {
            at += 4;
        }
        let end = self.decode_plane(t, video, at, new, old, 0);
        let next = if self.flags & 0x8000 == 0 { at + u32_at(at - 4) - 4 } else { end };
        let mut cend = next;
        if self.flags & 0x20000 == 0 {
            let n = self.decode_plane(t, video, next, new, old, 1);
            cend = self.decode_plane(t, video, n, new, old, 2);
        }
        self.last_check = (end, next, cend, video.len());
    }

    fn decode_plane(&mut self, t: &BinkTables, video: &[u8], at: usize, new: usize, old: usize, which: usize) -> usize {
        let sizes = self.bundle_sizes;
        let signed_colors = self.flags & 0x8000 != 0;
        let pick = |f: &mut FrameBuffer| -> *mut Plane {
            match which {
                0 => &mut f.y as *mut Plane,
                1 => &mut f.u as *mut Plane,
                2 => &mut f.v as *mut Plane,
                _ => f.a.as_mut().map(|p| p as *mut Plane).unwrap_or(std::ptr::null_mut()),
            }
        };
        let cur_p = pick(&mut self.buffers[new]);
        let prev_p = pick(&mut self.buffers[old]);
        if cur_p.is_null() {
            return at;
        }
        // SAFETY: `cur_p` / `prev_p` point into distinct buffers unless new == old, where `prev` stays None
        let (cur, prev) = unsafe {
            let cur = &mut *cur_p;
            let prev = if new != old { Some(&*(prev_p as *const Plane)) } else { None };
            (cur, prev)
        };
        let (w, h) = (cur.width, cur.height);
        let mut io = PlaneIo { cur: &mut cur.data, prev: prev.map(|p| p.data.as_slice()), stride: w };
        decode_plane(t, video, at, w, h, &mut io, &sizes, signed_colors)
    }
}

/// 0x00545E20 one plane; returns the byte offset after the last loaded word
fn decode_plane(
    t: &BinkTables,
    data: &[u8],
    at: usize,
    w: usize,
    h: usize,
    io: &mut PlaneIo,
    sizes: &[usize; 9],
    signed_colors: bool,
) -> usize {
    let mut br = BitReader::new(data, at);
    let b = (w >> 3) as u32;
    let v15 = count_bits(b + 511);
    let mut btype = Bundle::new(sizes[0], v15);
    let mut sub = Bundle::new(sizes[1], count_bits((w >> 4) as u32 + 511));
    let mut colors = Bundle::new(sizes[2], count_bits((b << 6) + 511));
    let mut pattern = Bundle::new(sizes[3], count_bits(8 * b + 511));
    let mut xoff = Bundle::new(sizes[4], v15);
    let mut yoff = Bundle::new(sizes[5], v15);
    let mut intra = DcBundle { buf: vec![0; sizes[6] / 2], cur: 0, end: 0, count_bits: v15, signed: false };
    let mut inter = DcBundle { buf: vec![0; sizes[7] / 2], cur: 0, end: 0, count_bits: v15, signed: true };
    let mut runs = Bundle::new(sizes[8], count_bits(48 * b + 511));
    btype.tree = read_tree(&mut br);
    sub.tree = read_tree(&mut br);
    let mut ctx = ColorCtx { trees: [Tree::default(); 16], last: 0 };
    for tr in ctx.trees.iter_mut() {
        *tr = read_tree(&mut br);
    }
    colors.tree = read_tree(&mut br);
    pattern.tree = read_tree(&mut br);
    xoff.tree = read_tree(&mut br);
    yoff.tree = read_tree(&mut br);
    runs.tree = read_tree(&mut br);
    let stride = io.stride;
    let mut coef = [0i16; 64];
    let mut y = 0usize;
    while y < h {
        btype.read_block_types(t, &mut br);
        sub.read_block_types(t, &mut br);
        colors.read_colors(t, &mut br, &mut ctx, signed_colors);
        pattern.read_patterns(t, &mut br);
        xoff.read_motion(t, &mut br);
        yoff.read_motion(t, &mut br);
        intra.read(&mut br);
        inter.read(&mut br);
        runs.read_runs(t, &mut br);
        let mut x = 0usize;
        let mut left = w as isize;
        while left > 0 {
            let off = y * stride + x;
            let mut adv = 8usize;
            let motion = |xo: &mut Bundle, yo: &mut Bundle| -> isize {
                let mx = xo.next() as i8 as isize;
                let my = yo.next() as i8 as isize;
                off as isize + my * stride as isize + mx
            };
            match btype.next() {
                0 => {
                    let b = io.prev8(off as isize);
                    io.block8(off, &b);
                }
                1 => {
                    adv = 16;
                    if y & 8 == 0 {
                        match sub.next() {
                            3 => {
                                let scan = &t.scan[br.read(4) as usize];
                                let b = read_run_block(scan, &mut colors, &mut runs, &mut br);
                                io.block16(off, &b);
                            }
                            5 => {
                                coef[0] = intra.next() as i16;
                                read_coefs(&mut br, &mut coef);
                                let q = br.read(4) as usize;
                                let b = idct(&coef, &t.intra_q[q]);
                                io.block16(off, &b);
                            }
                            6 => {
                                let b = [colors.next(); 64];
                                io.block16(off, &b);
                            }
                            8 => {
                                let c1 = u32::from_ne_bytes([colors.next(); 4]);
                                let c2 = u32::from_ne_bytes([colors.next(); 4]);
                                for r in 0..8 {
                                    let p = pattern.next() as usize;
                                    for k in 0..4 {
                                        let n = (p >> (2 * k)) & 3;
                                        let v = (c1 & t.pat16_a[n] | c2 & t.pat16_b[n]).to_le_bytes();
                                        for (i, px) in v.iter().enumerate() {
                                            io.set(off + 2 * r * stride + 4 * k + i, *px);
                                            io.set(off + (2 * r + 1) * stride + 4 * k + i, *px);
                                        }
                                    }
                                }
                            }
                            9 => {
                                let mut b = [0u8; 64];
                                for px in b.iter_mut() {
                                    *px = colors.next();
                                }
                                io.block16(off, &b);
                            }
                            _ => {}
                        }
                    }
                }
                2 => {
                    let src = motion(&mut xoff, &mut yoff);
                    let b = io.prev8(src);
                    io.block8(off, &b);
                }
                3 => {
                    let scan = &t.scan[br.read(4) as usize];
                    let b = read_run_block(scan, &mut colors, &mut runs, &mut br);
                    io.block8(off, &b);
                }
                4 => {
                    let src = motion(&mut xoff, &mut yoff);
                    let max = br.read(7);
                    let mut res = [0i8; 64];
                    read_residue(&mut br, &mut res, max);
                    let mut b = io.prev8(src);
                    for (n, px) in b.iter_mut().enumerate() {
                        *px = px.wrapping_add(res[BINK_ORDER[n] as usize] as u8);
                    }
                    io.block8(off, &b);
                }
                5 => {
                    coef[0] = intra.next() as i16;
                    read_coefs(&mut br, &mut coef);
                    let q = br.read(4) as usize;
                    let b = idct(&coef, &t.intra_q[q]);
                    io.block8(off, &b);
                }
                6 => {
                    let b = [colors.next(); 64];
                    io.block8(off, &b);
                }
                7 => {
                    let src = motion(&mut xoff, &mut yoff);
                    coef[0] = inter.next() as i16;
                    read_coefs(&mut br, &mut coef);
                    let q = br.read(4) as usize;
                    let d = idct(&coef, &t.inter_q[q]);
                    let mut b = io.prev8(src);
                    for (px, v) in b.iter_mut().zip(d) {
                        *px = px.wrapping_add(v);
                    }
                    io.block8(off, &b);
                }
                8 => {
                    let c1 = u32::from_ne_bytes([colors.next(); 4]);
                    let c2 = u32::from_ne_bytes([colors.next(); 4]);
                    let mut b = [0u8; 64];
                    for r in 0..8 {
                        let p = pattern.next() as usize;
                        let lo = (c1 & t.pat8_a[p & 15] | c2 & t.pat8_b[p & 15]).to_le_bytes();
                        let hi = (c1 & t.pat8_a[p >> 4] | c2 & t.pat8_b[p >> 4]).to_le_bytes();
                        b[r * 8..r * 8 + 4].copy_from_slice(&lo);
                        b[r * 8 + 4..r * 8 + 8].copy_from_slice(&hi);
                    }
                    io.block8(off, &b);
                }
                9 => {
                    let mut b = [0u8; 64];
                    for px in b.iter_mut() {
                        *px = colors.next();
                    }
                    io.block8(off, &b);
                }
                _ => {}
            }
            x += adv;
            left -= adv as isize;
        }
        y += 8;
    }
    br.word_end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_is_a_permutation() {
        let mut seen = [false; 64];
        for &i in BINK_ORDER.iter() {
            assert!(!seen[i as usize]);
            seen[i as usize] = true;
        }
    }

    #[test]
    fn idct_dc_only() {
        let mut c = [0i16; 64];
        c[0] = 100;
        let q = [1 << 11; 64];
        let b = idct(&c, &q);
        assert!(b.iter().all(|&v| v == b[0]));
    }

    #[test]
    fn mmx_yuv_blitter() {
        // hand-run through 0x00550FC0 with the BINKDATA constants
        assert_eq!(yuv_to_rgb_mmx(16, 128, 128), [0, 1, 0]);
        assert_eq!(yuv_to_rgb_mmx(235, 128, 128), [254, 255, 254]);
        assert_eq!(yuv_to_rgb_mmx(255, 255, 255), [255, 124, 255]);
        assert_eq!(yuv_to_rgb_mmx(0, 0, 0), [0, 138, 0]);
        // every input stays in range (paddsw never goes negative with these biases)
        for y in (0..=255).step_by(5) {
            for u in (0..=255).step_by(5) {
                for v in (0..=255).step_by(5) {
                    let _ = yuv_to_rgb_mmx(y, u, v);
                }
            }
        }
    }
}
