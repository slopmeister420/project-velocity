//! Fonts\FontManager.cpp — `CBaseFont` / `CFontManager`: bitmap fonts from `\Fonts\fonts.vcd`.
//!
//! each font is three files in the pack: `<name>.fml` (FontCreator xml source, unused at runtime),
//! `<name>.fob` (metrics) and `<name>.vtf` (glyph page).
//!
//! `.fob` (version 0x198):
//! ```text
//! +0x00 'FNTF' | +0x08 u16 version | +0x0A u16 char count | +0x0C u32 flags | +0x14 u32 char table offset
//! +0x18 u32 kerning pair count | +0x24 u16 line height | +0x68 u32 range count | +0x6C u32 range table offset
//! +0x70 u16 ascent
//! char (12 bytes): u16 code | u8 w | u8 h | u16 u | u16 v | s8 advance | s8 x offset | s8 y offset | s8 v advance
//! kerning (6 bytes, after the chars when FOB_FLAG_KERN6): u16 first | u16 second | s8 amount | pad
//! range (8 bytes): u16 first code | u16 first index | u16 count | pad
//! ```

use crate::render2d::{Colour, DrawList, Quad, TextureId, TextureStore};

pub const FOB_VERSION: u16 = 0x198;
pub const FOB_FLAG_CHARSTRUCT16: u32 = 0x4_0000;
pub const FOB_FLAG_KERN6: u32 = 0x8_0000;
pub const FOB_FLAG_RANGES: u32 = 0x10_0000;
pub const FOB_FLAG_CHAR12: u32 = 0x20_0000;

/// `MAX_FONTS`
pub const MAX_FONTS: usize = 36;

/// text flags (`FONT_*`).
pub const FONT_CENTRE_X: u32 = 0x1;
pub const FONT_RIGHT: u32 = 0x2;
pub const FONT_JUSTIFY_X: u32 = 0x4;
pub const FONT_CENTRE_Y: u32 = 0x8;
pub const FONT_BOTTOM: u32 = 0x10;
pub const FONT_JUSTIFY_Y: u32 = 0x20;
pub const FONT_VERTICAL: u32 = 0x40;
pub const FONT_REVERSE: u32 = 0x80;
pub const FONT_RTL: u32 = 0x100;
pub const FONT_WORDWRAP: u32 = 0x200;
pub const FONT_CHARWRAP: u32 = 0x400;
pub const FONT_NOCLIP: u32 = 0x800;

pub const FONT_CHAR_NULL: u16 = 0;
pub const FONT_CHAR_NEWLINE: u16 = 10;
pub const FONT_CHAR_RETURN: u16 = 13;
pub const FONT_CHAR_SPACE: u16 = 32;
pub const FONT_CHAR_NBSP: u16 = 160;
pub const FONT_CHAR_ZWSP: u16 = 0x200B;
/// inline texture: `E000, u32 texture id`
pub const FONT_CHAR_TEXTURE: u16 = 0xE000;
/// inline texture with size/offset: `E001, u32 texture id, u16 w, u16 h, s16 x, s16 y`
pub const FONT_CHAR_TEXTURE_EX: u16 = 0xE001;
/// colour change: `E010, u32 argb`
pub const FONT_CHAR_COLOUR: u16 = 0xE010;
/// colour reset
pub const FONT_CHAR_COLOUR_RESET: u16 = 0xE011;

/// `0x006C18DC`: the space character takes the metrics of this glyph ...
pub const SPACE_SUBSTITUTE: u16 = b's' as u16;
/// `0x006C18E0`: ... unless this tweak is set above -1000.
pub const SPACE_WIDTH_TWEAK: f32 = -1001.0;

/// `gRenderer+0x14` read through `Renderer::GetXScale` (0x002AFF00); 1.0 from the renderer ctor (0x002AB160).
pub const RENDER_X_SCALE: f32 = 1.0;

/// one 12-byte glyph record.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FobChar {
    pub code: u16,
    pub w: u8,
    pub h: u8,
    pub u: u16,
    pub v: u16,
    pub advance: i8,
    pub x_off: i8,
    pub y_off: i8,
    pub v_advance: i8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FobKern {
    pub first: u16,
    pub second: u16,
    pub amount: i8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FobRange {
    pub first: u16,
    pub index: u16,
    pub count: u16,
}

/// parsed `.fob`.
#[derive(Debug, Clone)]
pub struct FobData {
    pub version: u16,
    pub flags: u32,
    pub line_height: u16,
    pub ascent: u16,
    pub chars: Vec<FobChar>,
    pub kerns: Vec<FobKern>,
    pub ranges: Vec<FobRange>,
}

impl FobData {
    pub fn parse(b: &[u8]) -> Option<Self> {
        if b.get(0..4)? != b"FNTF" {
            return None;
        }
        let r16 = |o: usize| -> Option<u16> { Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?)) };
        let r32 = |o: usize| -> Option<u32> { Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?)) };
        let version = r16(8)?;
        let count = r16(10)? as usize;
        let flags = r32(12)?;
        let char_off = r32(0x14)? as usize;
        let kern_count = r32(0x18)? as usize;
        let line_height = r16(0x24)?;
        let mut chars = Vec::with_capacity(count);
        for i in 0..count {
            let o = char_off + 12 * i;
            let c = b.get(o..o + 12)?;
            chars.push(FobChar {
                code: u16::from_le_bytes([c[0], c[1]]),
                w: c[2],
                h: c[3],
                u: u16::from_le_bytes([c[4], c[5]]),
                v: u16::from_le_bytes([c[6], c[7]]),
                advance: c[8] as i8,
                x_off: c[9] as i8,
                y_off: c[10] as i8,
                v_advance: c[11] as i8,
            });
        }
        let mut kerns = Vec::new();
        if flags & FOB_FLAG_KERN6 != 0 {
            let ko = char_off + 12 * count;
            for i in 0..kern_count {
                let k = b.get(ko + 6 * i..ko + 6 * i + 6)?;
                kerns.push(FobKern {
                    first: u16::from_le_bytes([k[0], k[1]]),
                    second: u16::from_le_bytes([k[2], k[3]]),
                    amount: k[4] as i8,
                });
            }
        }
        let mut ranges = Vec::new();
        if flags & FOB_FLAG_RANGES != 0 {
            let n = r32(0x68)? as usize;
            let ro = r32(0x6C)? as usize;
            for i in 0..n {
                ranges.push(FobRange { first: r16(ro + 8 * i)?, index: r16(ro + 8 * i + 2)?, count: r16(ro + 8 * i + 4)? });
            }
        }
        let ascent = if flags & FOB_FLAG_CHAR12 != 0 { r16(0x70)? } else { line_height };
        Some(Self { version, flags, line_height, ascent, chars, kerns, ranges })
    }
}

/// result of `GetLineInfo` for one line.
#[derive(Debug, Clone, Copy, Default)]
pub struct LineInfo {
    pub start: usize,        // +0
    pub end: usize,          // +4 index of the last character examined
    pub width: f32,          // +8
    pub height: f32,         // +12
    pub char_count: u32,     // +16
    pub visible_count: u32,  // +20
}

/// `SFontDrawParams` (FontManager.cpp, the struct built in 0x002ADA30).
pub struct DrawParams<'a> {
    pub flags: u32,                     // +0
    pub colour: Colour,                 // +4
    pub current_colour: Colour,         // +8
    pub clip: Option<[f32; 4]>,         // +12
    pub out: &'a mut DrawList,
}

/// `CBaseFont`
#[derive(Debug, Clone)]
pub struct CBaseFont {
    pub name_crc: u32,      // +0x08
    pub inv_w: f32,         // +0x0C
    pub inv_h: f32,         // +0x10
    pub texture: TextureId, // +0x14
    pub fob: FobData,       // +0x1C
    pub line_height: u16,   // +0x3E
    pub ascent: u16,        // +0x3C
    pub scale: f32,         // +0x40
    pub spacing: i32,       // +0x44
}

fn next_char(s: &[u16], i: usize) -> Option<usize> {
    // FontManager.cpp 0x002AFF20 — GetNext
    let c = *s.get(i).unwrap_or(&0);
    match c {
        FONT_CHAR_COLOUR => Some(i + 3),
        FONT_CHAR_COLOUR_RESET => Some(i + 1),
        FONT_CHAR_TEXTURE_EX => Some(i + 7),
        0 => None,
        FONT_CHAR_TEXTURE => Some(i + 3),
        _ => Some(i + 1),
    }
}

fn at(s: &[u16], i: Option<usize>) -> u16 {
    i.and_then(|i| s.get(i).copied()).unwrap_or(0)
}

impl CBaseFont {
    /// FontManager.cpp 0x002AC780
    pub fn new(name_crc: u32, fob: FobData, texture: TextureId, tex_size: (u32, u32)) -> Self {
        assert!(fob.version >= FOB_VERSION, "m_pFontData->wVersion >= FOB_VERSION");
        assert!(fob.flags & FOB_FLAG_CHARSTRUCT16 == 0, "! ( m_pFontData->dFlags & FOB_FLAG_CHARSTRUCT16 )");
        let line_height = fob.line_height;
        let ascent = if fob.flags & FOB_FLAG_CHAR12 != 0 { fob.ascent } else { line_height };
        Self {
            name_crc,
            inv_w: 1.0 / tex_size.0 as f32,
            inv_h: 1.0 / tex_size.1 as f32,
            texture,
            fob,
            line_height,
            ascent,
            scale: 1.0,
            spacing: 0,
        }
    }

    /// FontManager.cpp 0x002B0260 — `FindCharacter`
    pub fn find_character(&self, c: u16) -> Option<FobChar> {
        let found = if !self.fob.ranges.is_empty() {
            let mut lo: i32 = -1;
            let mut hi: u32 = self.fob.ranges.len() as u32;
            while hi.wrapping_sub(lo as u32) > 1 {
                let mid = ((lo as u32).wrapping_add(hi) >> 1) as usize;
                if self.fob.ranges[mid].first as u32 <= c as u32 {
                    lo = mid as i32;
                } else {
                    hi = mid as u32;
                }
            }
            if lo > -1 {
                let r = self.fob.ranges[lo as usize];
                if r.first <= c && (r.count as u32 + r.first as u32) > c as u32 {
                    let ch = self.fob.chars[(r.index.wrapping_add(c - r.first)) as usize];
                    assert_eq!(ch.code, c, "pCharacter->code == c");
                    Some(ch)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            self.fob.chars.iter().find(|ch| ch.code == c).copied()
        };
        match found {
            Some(mut ch) if self.fob.flags & FOB_FLAG_CHAR12 == 0 => {
                // old format: the last byte is replaced by the line height
                ch.v_advance = self.line_height as i8;
                Some(ch)
            }
            f => f,
        }
    }

    /// FontManager.cpp 0x002AFFA0 — `GetCharacter`: glyph for the character at `s[i]` (synthesised for inline textures).
    pub fn get_character(&self, s: &[u16], i: usize, tex: &TextureStore) -> Option<FobChar> {
        let c = at(s, Some(i));
        if c > FONT_CHAR_ZWSP {
            match c {
                FONT_CHAR_TEXTURE => {
                    let (w, h) = tex.size(read_u32(s, i + 1));
                    Some(FobChar { code: 0, w: w as u8, h: h as u8, u: 0, v: 0, advance: w as i8, x_off: 0, y_off: 0, v_advance: h as i8 })
                }
                FONT_CHAR_TEXTURE_EX => Some(FobChar {
                    code: 0,
                    w: (s[i + 3]) as u8,
                    h: (s[i + 4]) as u8,
                    u: 0,
                    v: 0,
                    advance: s[i + 3].wrapping_add(s[i + 5]) as i8,
                    x_off: s[i + 5] as i8,
                    y_off: s[i + 6] as i8,
                    v_advance: s[i + 4].wrapping_add(s[i + 6]) as i8,
                }),
                FONT_CHAR_COLOUR | FONT_CHAR_COLOUR_RESET => None,
                _ => self.find_character(c),
            }
        } else if c == FONT_CHAR_ZWSP {
            None
        } else {
            match c {
                FONT_CHAR_NEWLINE | FONT_CHAR_RETURN => None,
                FONT_CHAR_SPACE | FONT_CHAR_NBSP => {
                    if SPACE_WIDTH_TWEAK < -1000.0 {
                        self.find_character(SPACE_SUBSTITUTE)
                    } else {
                        let w = SPACE_WIDTH_TWEAK as i32 as u8;
                        Some(FobChar { code: c, w, h: 0, u: 0, v: 0, advance: w as i8, x_off: 0, y_off: 0, v_advance: self.line_height as i8 })
                    }
                }
                _ => self.find_character(c),
            }
        }
    }

    /// FontManager.cpp 0x002B04E0 — `FindKerning`
    fn find_kerning(&self, a: u16, b: u16) -> Option<FobKern> {
        let k = &self.fob.kerns;
        if k.is_empty() {
            return None;
        }
        let key = ((b as u32) << 16) + a as u32;
        let mut lo: i32 = -1;
        let mut hi = k.len() as u32;
        while hi.wrapping_sub(lo as u32) > 1 {
            let mid = ((lo as u32).wrapping_add(hi) >> 1) as usize;
            if ((k[mid].second as u32) << 16) + k[mid].first as u32 <= key {
                lo = mid as i32;
            } else {
                hi = mid as u32;
            }
        }
        if lo > -1 && k[lo as usize].first == a && k[lo as usize].second == b {
            Some(k[lo as usize])
        } else {
            None
        }
    }

    /// FontManager.cpp 0x002B0480 — `GetKerning`: zero pair (0x007814C0) when either is a space or none is found.
    fn get_kerning(&self, a: u16, b: u16) -> i8 {
        if a == FONT_CHAR_SPACE || b == FONT_CHAR_SPACE {
            return 0;
        }
        self.find_kerning(a, b).map(|k| k.amount).unwrap_or(0)
    }

    /// FontManager.cpp 0x002B0420 — `IsWhiteSpace`
    fn is_white_space(c: u16, ch: Option<FobChar>) -> bool {
        if ch.is_none() {
            return false;
        }
        if c > 0xA0 { (FONT_CHAR_TEXTURE..=FONT_CHAR_TEXTURE_EX).contains(&c) } else { c == FONT_CHAR_NBSP || c == FONT_CHAR_SPACE }
    }

    /// FontManager.cpp 0x002B0640 — overhang right of the advance.
    fn overhang_x(ch: Option<FobChar>) -> u32 {
        match ch {
            Some(c) => {
                let a = (c.w as i32 + c.x_off as i32) as u32;
                let b = c.advance as i32 as u32;
                if a > b { a - b } else { 0 }
            }
            None => 0,
        }
    }

    /// FontManager.cpp 0x002B0690 — overhang below the vertical advance.
    fn overhang_y(ch: Option<FobChar>) -> u32 {
        match ch {
            Some(c) => {
                let a = (c.h as i32 + c.y_off as i32) as u32;
                let b = c.v_advance as i32 as u32;
                if a > b { a - b } else { 0 }
            }
            None => 0,
        }
    }

    fn x_scale(&self) -> f32 {
        RENDER_X_SCALE * self.scale
    }

    /// FontManager.cpp 0x002ACC40 — `GetLineInfo`: measures the line starting at `start`, returns the next line start.
    pub fn get_line_info(&self, s: &[u16], start: usize, out: &mut LineInfo, w: f32, h: f32, flags: u32, tex: &TextureStore) -> Option<usize> {
        let scale = self.x_scale();
        let inv = 1.0 / scale;
        let mut char_count: u32 = 0;
        let mut visible: u32 = 0;
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let mut ascent = 0.0f32;
        let mut vlen = 0.0f32;
        let (max_w, max_h) = if flags & FONT_VERTICAL != 0 { (f32::MAX, h) } else { (w * inv, f32::MAX) };
        let mut line_end = start;
        let mut last: Option<FobChar> = None;
        let mut cur = start;
        let mut last_ch_any: Option<FobChar>;
        loop {
            let mut wc: u32 = 0;
            let mut wv: u32 = 0;
            let mut ww = 0.0f32;
            let mut wh = 0.0f32;
            let mut wa = 0.0f32;
            let mut wvl = 0.0f32;
            let mut wlast: Option<FobChar> = None;
            // original bug: `== 32 && == 8203 && == 13` can never hold, so leading spaces are never skipped here
            while flags & FONT_WORDWRAP != 0 && at(s, Some(cur)) == 32 && at(s, Some(cur)) == 8203 && at(s, Some(cur)) == 13 {
                if at(s, Some(cur)) == 32 {
                    wc += 1;
                }
                cur = next_char(s, cur).unwrap_or(s.len());
            }
            if wc != 0 {
                let sp = self.get_character(&[32, 0], 0, tex).expect("pCharacter");
                ww = (wc as i32 * (self.spacing + sp.advance as i32)) as u32 as f32;
                wh = self.line_height as f32;
                wa = self.ascent as f32;
                wvl = (wc as i32 * (self.spacing + sp.v_advance as i32)) as u32 as f32;
            }
            let mut word_end;
            loop {
                let nxt = next_char(s, cur);
                let ch = self.get_character(s, cur, tex);
                if let Some(c) = ch {
                    let k = if flags & FONT_RTL != 0 {
                        self.get_kerning(at(s, nxt), at(s, Some(cur)))
                    } else {
                        self.get_kerning(at(s, Some(cur)), at(s, nxt))
                    };
                    wlast = Some(c);
                    wc += 1;
                    if !Self::is_white_space(at(s, Some(cur)), ch) {
                        wv += 1;
                    }
                    ww += (self.spacing + k as i32 + c.advance as i32) as f32;
                    wh = fmax(fmax(wh, (c.h as i32 + c.y_off as i32) as f32), c.v_advance as f32);
                    wa = fmax(fmax(wa, (c.w as i32 + c.x_off as i32) as f32), c.advance as f32);
                    wvl += (self.spacing + c.v_advance as i32) as f32;
                }
                word_end = cur;
                cur = nxt.unwrap_or(s.len());
                if !(flags & FONT_WORDWRAP != 0 && at(s, Some(word_end)) != 0 && at(s, Some(word_end)) != 10 && at(s, Some(cur)) != 32) {
                    break;
                }
            }
            last_ch_any = wlast;
            if flags & (FONT_WORDWRAP | FONT_CHARWRAP) != 0 && line_end != start {
                if (Self::overhang_x(last_ch_any) as f32) + (width + ww) >= max_w {
                    break;
                }
                if (Self::overhang_y(last_ch_any) as f32) + (vlen + wvl) >= max_h {
                    break;
                }
            }
            if wlast.is_some() {
                last = wlast;
            }
            line_end = word_end;
            char_count += wc;
            visible += wv;
            width += ww;
            height = fmax(height, wh);
            ascent = fmax(ascent, wa);
            vlen += wvl;
            if at(s, Some(line_end)) == 0 || at(s, Some(line_end)) == 10 {
                break;
            }
        }
        assert!(last.is_some() || char_count == 0, "( pLastCharacter != NULL ) || ( dCharCount == 0 )");
        if last.is_some() {
            width -= self.spacing as f32;
            width += Self::overhang_x(last) as f32;
            vlen += Self::overhang_y(last) as f32;
        }
        let mut next = next_char(s, line_end);
        if let Some(mut n) = next {
            if flags & FONT_WORDWRAP != 0 {
                while matches!(at(s, Some(n)), 32 | 8203 | 13) {
                    match next_char(s, n) {
                        Some(m) => n = m,
                        None => break,
                    }
                }
            }
            next = if at(s, Some(n)) == 0 { None } else { Some(n) };
        }
        out.start = start;
        out.end = line_end;
        if flags & FONT_VERTICAL != 0 {
            out.width = ascent * scale;
            out.height = vlen;
        } else {
            out.width = width * scale;
            out.height = height;
        }
        out.char_count = char_count;
        out.visible_count = visible;
        next
    }

    /// FontManager.cpp 0x002ACAC0 — `GetStringExtent`: (width, height, lines, chars, visible).
    pub fn get_string_extent(&self, s: &[u16], w: f32, h: f32, flags: u32, tex: &TextureStore) -> (f32, f32, u32, u32, u32) {
        let (mut ew, mut eh, mut lines, mut chars, mut vis) = (0.0f32, 0.0f32, 0u32, 0u32, 0u32);
        let mut cur = Some(0usize);
        let mut info = LineInfo::default();
        while let Some(c) = cur {
            cur = self.get_line_info(s, c, &mut info, w, h, flags, tex);
            if flags & FONT_VERTICAL != 0 {
                ew += info.width;
                eh = fmax(eh, info.height);
                if flags & FONT_JUSTIFY_Y != 0 && at(s, Some(info.end)) != 10 && at(s, Some(info.end)) != 0 {
                    eh = h;
                }
            } else {
                ew = fmax(ew, info.width);
                eh += info.height;
                if flags & FONT_JUSTIFY_X != 0 && at(s, Some(info.end)) != 10 && at(s, Some(info.end)) != 0 {
                    ew = w;
                }
            }
            chars += info.char_count;
            vis += info.visible_count;
            lines += 1;
        }
        if lines > 1 && flags & FONT_VERTICAL != 0 {
            if flags & FONT_JUSTIFY_X != 0 {
                ew = w;
            }
        } else if flags & FONT_JUSTIFY_Y != 0 {
            eh = h;
        }
        (ew, eh, lines, chars, vis)
    }

    /// FontManager.cpp 0x002AD550 — `GetStringWidth` (truncated to an int as the original returns it).
    pub fn get_string_width(&self, s: &[u16], flags: u32, tex: &TextureStore) -> i32 {
        let mut info = LineInfo::default();
        self.get_line_info(s, 0, &mut info, f32::MAX, f32::MAX, flags, tex);
        if flags & FONT_VERTICAL != 0 { info.height as i32 } else { info.width as i32 }
    }

    /// FontManager.cpp 0x002ADA30 — `DrawString` into a rectangle with alignment / wrapping.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_string_rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, s: &[u16], colour: Colour, flags: u32, offset: Option<[f32; 2]>, tex: &TextureStore, out: &mut DrawList) {
        let (mut rx0, mut ry0, mut rx1, mut ry1) = (x0, y0, x1, y1);
        if let Some(o) = offset {
            rx0 += o[0];
            ry0 += o[1];
            rx1 += o[0];
            ry1 += o[1];
        }
        let clip = if flags & FONT_NOCLIP != 0 { None } else { Some([rx0, ry0, rx1, ry1]) };
        let f = flags & 0xFFFF_FFDB;
        let (ew, eh, lines, chars, _) = self.get_string_extent(s, rx1 - rx0, ry1 - ry0, f, tex);
        if chars == 0 {
            return;
        }
        let (mut px, mut py);
        let (mut dir_x, mut dir_y, mut step_x, mut step_y) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let reverse = flags & FONT_REVERSE != 0;
        if flags & FONT_VERTICAL != 0 {
            py = ry0;
            if reverse {
                dir_x = -1.0;
                if flags & FONT_JUSTIFY_X != 0 {
                    px = rx1;
                    step_x = -((rx1 - rx0 - ew) / (lines - 1) as f32);
                } else if flags & FONT_CENTRE_X != 0 {
                    px = (rx0 + rx1 + ew) * 0.5;
                } else if flags & FONT_RIGHT != 0 {
                    px = rx1;
                } else {
                    px = rx0 + ew;
                }
            } else {
                dir_x = 1.0;
                if flags & FONT_JUSTIFY_X != 0 {
                    px = rx0;
                    step_x = (rx1 - rx0 - ew) / (lines - 1) as f32;
                } else if flags & FONT_CENTRE_X != 0 {
                    px = (rx0 + rx1 - ew) * 0.5;
                } else if flags & FONT_RIGHT != 0 {
                    px = rx1 - ew;
                } else {
                    px = rx0;
                }
            }
        } else {
            px = rx0;
            if reverse {
                dir_y = -1.0;
                if flags & FONT_JUSTIFY_Y != 0 {
                    py = ry1;
                    step_y = -((ry1 - ry0 - eh) / (lines - 1) as f32);
                } else if flags & FONT_CENTRE_Y != 0 {
                    py = (ry0 + ry1 + eh) * 0.5;
                } else if flags & FONT_BOTTOM != 0 {
                    py = ry1;
                } else {
                    py = ry0 + eh;
                }
            } else {
                dir_y = 1.0;
                if flags & FONT_JUSTIFY_Y != 0 {
                    py = ry0;
                    step_y = (ry1 - ry0 - eh) / (lines - 1) as f32;
                } else if flags & FONT_CENTRE_Y != 0 {
                    py = (ry0 + ry1 - eh) * 0.5;
                } else if flags & FONT_BOTTOM != 0 {
                    py = ry1 - eh;
                } else {
                    py = ry0;
                }
            }
        }
        let mut params = DrawParams { flags, colour, current_colour: colour, clip, out };
        let mut cur = Some(0usize);
        let mut info = LineInfo::default();
        while let Some(c) = cur {
            cur = self.get_line_info(s, c, &mut info, rx1 - rx0, ry1 - ry0, flags, tex);
            if reverse {
                px += dir_x * info.width;
                py += dir_y * info.height;
            }
            self.draw_line(&mut params, px, py, rx1, ry1, &info, s, tex);
            if !reverse {
                px += dir_x * info.width;
                py += dir_y * info.height;
            }
            px += step_x;
            py += step_y;
        }
    }

    /// FontManager.cpp 0x002AD850 — `DrawString` at a point (alignment relative to the point).
    pub fn draw_string_at(&self, x: f32, y: f32, s: &[u16], colour: Colour, flags: u32, tex: &TextureStore, out: &mut DrawList) {
        let (mut x, mut y) = (x, y);
        if flags & FONT_VERTICAL != 0 {
            if flags & FONT_CENTRE_X != 0 {
                x -= self.line_height as f32 * 0.5;
            } else if flags & FONT_RIGHT != 0 {
                x -= self.line_height as f32;
            }
        } else if flags & FONT_CENTRE_Y != 0 {
            y -= self.ascent as f32 * 0.5;
        } else if flags & FONT_BOTTOM != 0 {
            y -= self.ascent as f32;
        }
        let mut info = LineInfo::default();
        self.get_line_info(s, 0, &mut info, f32::MAX, f32::MAX, flags, tex);
        if info.char_count != 0 {
            let mut params = DrawParams { flags, colour, current_colour: colour, clip: None, out };
            self.draw_line(&mut params, x, y, x, y, &info, s, tex);
        }
    }

    /// FontManager.cpp 0x002ADFA0 — `DrawLine`: in-line alignment then the direction specific walker.
    #[allow(clippy::too_many_arguments)]
    fn draw_line(&self, p: &mut DrawParams, x: f32, y: f32, x1: f32, y1: f32, info: &LineInfo, s: &[u16], tex: &TextureStore) {
        let mut f = p.flags;
        let e = at(s, Some(info.end));
        if e == 0 || e == 10 {
            f &= 0xFFDB;
        }
        let start = info.start;
        let end = next_char(s, info.end).unwrap_or(s.len());
        let (mut lx, mut ly) = (x, y);
        let mut extra = 0.0f32;
        if f & FONT_VERTICAL != 0 {
            if f & FONT_JUSTIFY_Y != 0 {
                extra = (y1 - y - info.height) / (info.char_count - 1) as f32;
            } else if f & FONT_CENTRE_Y != 0 {
                ly = (y + y1 - info.height) * 0.5;
            } else if f & FONT_BOTTOM != 0 {
                ly = y1 - info.height;
            }
            if f & FONT_RTL != 0 {
                let base = if f & FONT_JUSTIFY_Y != 0 { y1 } else { ly + info.height };
                self.draw_vertical_rtl(p, x, base, start, end, extra, s, tex);
            } else {
                self.draw_vertical(p, x, ly, start, end, extra, s, tex);
            }
        } else {
            if f & FONT_JUSTIFY_X != 0 {
                extra = (x1 - x - info.width) / (info.char_count - 1) as f32;
            } else if f & FONT_CENTRE_X != 0 {
                lx = (x + x1 - info.width) * 0.5;
            } else if f & FONT_RIGHT != 0 {
                lx = x1 - info.width;
            }
            if f & FONT_RTL != 0 {
                let base = if f & FONT_JUSTIFY_X != 0 { x1 } else { lx + info.width };
                self.draw_rtl(p, base, y, start, end, extra, s, tex);
            } else {
                self.draw_ltr(p, lx, y, start, end, extra, s, tex);
            }
        }
    }

    /// FontManager.cpp 0x002AE1C0
    #[allow(clippy::too_many_arguments)]
    fn draw_ltr(&self, p: &mut DrawParams, mut x: f32, y: f32, start: usize, end: usize, extra: f32, s: &[u16], tex: &TextureStore) {
        let scale = self.x_scale();
        let mut i = start;
        while i != end && i < s.len() {
            let n = next_char(s, i).unwrap_or(s.len());
            let ch = self.get_character(s, i, tex);
            self.draw_char(p, x, y, ch, s, i, tex);
            if let Some(c) = ch {
                let k = self.get_kerning(at(s, Some(i)), at(s, Some(n)));
                let a = c.advance as f32 * scale + x;
                let b = k as f32 * scale + a;
                x = self.spacing as f32 * scale + b + extra;
            }
            i = n;
        }
    }

    /// FontManager.cpp 0x002AE330
    #[allow(clippy::too_many_arguments)]
    fn draw_rtl(&self, p: &mut DrawParams, x: f32, y: f32, start: usize, end: usize, extra: f32, s: &[u16], tex: &TextureStore) {
        let scale = self.x_scale();
        let first = self.get_character(s, start, tex);
        let mut x = x - Self::overhang_x(first) as f32 * scale;
        let mut i = start;
        while i != end && i < s.len() {
            let n = next_char(s, i).unwrap_or(s.len());
            let ch = self.get_character(s, i, tex);
            if let Some(c) = ch {
                x -= c.advance as f32 * scale;
            }
            self.draw_char(p, x, y, ch, s, i, tex);
            if ch.is_some() {
                let k = self.get_kerning(at(s, Some(n)), at(s, Some(i)));
                x = x - k as f32 * scale - self.spacing as f32 * scale - extra;
            }
            i = n;
        }
    }

    /// FontManager.cpp 0x002AE4E0
    #[allow(clippy::too_many_arguments)]
    fn draw_vertical(&self, p: &mut DrawParams, x: f32, mut y: f32, start: usize, end: usize, extra: f32, s: &[u16], tex: &TextureStore) {
        let mut i = start;
        while i != end && i < s.len() {
            let n = next_char(s, i).unwrap_or(s.len());
            let ch = self.get_character(s, i, tex);
            self.draw_char(p, x, y, ch, s, i, tex);
            if let Some(c) = ch {
                y = self.spacing as f32 + (c.v_advance as f32 + y) + extra;
            }
            i = n;
        }
    }

    /// FontManager.cpp 0x002AE610
    #[allow(clippy::too_many_arguments)]
    fn draw_vertical_rtl(&self, p: &mut DrawParams, x: f32, y: f32, start: usize, end: usize, extra: f32, s: &[u16], tex: &TextureStore) {
        let first = self.get_character(s, start, tex);
        let mut y = y - Self::overhang_y(first) as f32;
        let mut i = start;
        while i != end && i < s.len() {
            let n = next_char(s, i).unwrap_or(s.len());
            let ch = self.get_character(s, i, tex);
            if let Some(c) = ch {
                y -= c.v_advance as f32;
            }
            self.draw_char(p, x, y, ch, s, i, tex);
            if ch.is_some() {
                y = y - self.spacing as f32 - extra;
            }
            i = n;
        }
    }

    /// FontManager.cpp 0x002AE770 — `DrawCharacter`
    #[allow(clippy::too_many_arguments)]
    fn draw_char(&self, p: &mut DrawParams, x: f32, y: f32, ch: Option<FobChar>, s: &[u16], i: usize, tex: &TextureStore) {
        let scale = self.x_scale();
        match at(s, Some(i)) {
            FONT_CHAR_COLOUR => {
                assert!(ch.is_none(), "! pCharacter");
                let a = s[i + 1] >> 8;
                let r = s[i + 1] & 0xFF;
                let g = s[i + 2] >> 8;
                let b = s[i + 2] & 0xFF;
                p.current_colour = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
            FONT_CHAR_COLOUR_RESET => {
                assert!(ch.is_none(), "! pCharacter");
                p.current_colour = p.colour;
            }
            FONT_CHAR_SPACE | FONT_CHAR_NBSP => {
                assert!(ch.is_some(), "pCharacter");
            }
            FONT_CHAR_TEXTURE => {
                let id = read_u32(s, i + 1);
                let (tw, th) = tex.size(id);
                self.push_quad(p, id, x, y, tw as f32 * scale + x, th as f32 + y, 0.0, 0.0, 1.0, 1.0);
            }
            FONT_CHAR_TEXTURE_EX => {
                let id = read_u32(s, i + 1);
                let x0 = s[i + 5] as i16 as f32 * scale + x;
                let y0 = s[i + 6] as i16 as f32 + y;
                let x1 = (s[i + 3] as i32 + s[i + 5] as i16 as i32) as f32 * scale + x;
                let y1 = s[i + 6] as i16 as f32 + y + s[i + 4] as f32;
                self.push_quad(p, id, x0, y0, x1, y1, 0.0, 0.0, 1.0, 1.0);
            }
            _ => {
                if let Some(c) = ch {
                    if c.w == 0 || c.h == 0 {
                        return;
                    }
                    let v1 = (c.v as f32 + c.h as f32) * self.inv_h;
                    let u1 = (c.u as f32 + c.w as f32) * self.inv_w;
                    let v0 = c.v as f32 * self.inv_h;
                    let u0 = c.u as f32 * self.inv_w;
                    let y1 = c.y_off as f32 + y + c.h as f32;
                    let x1 = (c.w as i32 + c.x_off as i32) as f32 * scale + x;
                    let y0 = c.y_off as f32 + y;
                    let x0 = c.x_off as f32 * scale + x;
                    self.push_quad(p, self.texture, x0, y0, x1, y1, u0, v0, u1, v1);
                }
            }
        }
    }

    /// FontManager.cpp 0x002AEC70 — clip to the params rect, quantise uvs to 1/4096 and emit.
    #[allow(clippy::too_many_arguments)]
    fn push_quad(&self, p: &mut DrawParams, texture: TextureId, mut x1: f32, mut y1: f32, mut x2: f32, mut y2: f32, mut u1: f32, mut v1: f32, mut u2: f32, mut v2: f32) {
        let du = (u2 - u1) / (x2 - x1);
        let dv = (v2 - v1) / (y2 - y1);
        if let Some(c) = p.clip {
            if x1 >= c[2] || x2 <= c[0] || y1 >= c[3] || y2 <= c[1] {
                return;
            }
            if x1 < c[0] {
                let d = c[0] - x1;
                x1 += d;
                u1 += d * du;
            }
            if x2 > c[2] {
                let d = x2 - c[2];
                x2 -= d;
                u2 -= d * du;
            }
            if y1 < c[1] {
                let d = c[1] - y1;
                y1 += d;
                v1 += d * dv;
            }
            if y2 > c[3] {
                let d = y2 - c[3];
                y2 -= d;
                v2 -= d * dv;
            }
        }
        let q = |v: f32| ((v * 4096.0) as i32 as u16) as f32 / 4096.0;
        p.out.push(Quad {
            texture,
            pos: [[x1, y1], [x2, y1], [x1, y2], [x2, y2]],
            uv: [[q(u1), q(v1)], [q(u2), q(v1)], [q(u1), q(v2)], [q(u2), q(v2)]],
            colour: [p.current_colour; 4],
        });
    }
}

fn read_u32(s: &[u16], i: usize) -> u32 {
    (*s.get(i).unwrap_or(&0) as u32) | ((*s.get(i + 1).unwrap_or(&0) as u32) << 16)
}

/// `VsMax` (0x00057F90): returns the first unless the second is greater.
fn fmax(a: f32, b: f32) -> f32 {
    if b <= a { a } else { b }
}

/// `CFontManager`
#[derive(Default)]
pub struct CFontManager {
    pub fonts: Vec<CBaseFont>, // +0x04 m_pFonts[MAX_FONTS], +0x94 m_nFonts
}

impl CFontManager {
    /// FontManager.cpp 0x002AF750 — `LoadFont`: returns the font index (existing one if the crc is loaded).
    pub fn load_font(&mut self, vcd: &visos::file::Vcd, file: &str, name_crc: u32, tex: &mut TextureStore) -> Option<usize> {
        assert!(self.fonts.len() < MAX_FONTS, "m_nFonts < MAX_FONTS");
        if name_crc != 0 {
            if let Some(i) = self.fonts.iter().position(|f| f.name_crc == name_crc) {
                return Some(i);
            }
        }
        let fob = match vcd.get(&format!("{}.fob", file)).and_then(FobData::parse) {
            Some(f) => f,
            None => {
                log::error!("<!> FontManager: Failed to load font info file {}.fob", file);
                return None;
            }
        };
        let img = vcd.get(&format!("{}.vtf", file)).and_then(crate::vtf::decode)?;
        let size = (img.width, img.height);
        let id = tex.add(visos::handy::handy_crc::name_hash(file), img);
        self.fonts.push(CBaseFont::new(name_crc, fob, id, size));
        Some(self.fonts.len() - 1)
    }

    /// FontManager.cpp 0x002AFC80 — `FindFont` by name crc.
    pub fn find_font(&self, name_crc: u32) -> Option<usize> {
        self.fonts.iter().position(|f| f.name_crc == name_crc)
    }

    pub fn get(&self, i: usize) -> Option<&CBaseFont> {
        self.fonts.get(i)
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut CBaseFont> {
        self.fonts.get_mut(i)
    }
}

/// utf-16 helper with the terminating zero the walkers expect.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_every_fob() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let v = visos::file::Vcd::open("\\Fonts\\FONTS.VCD").unwrap();
        let mut n = 0;
        for e in v.entries() {
            if e.name.to_ascii_lowercase().ends_with(".fob") {
                let f = FobData::parse(v.data(&e).unwrap()).unwrap_or_else(|| panic!("{}", e.name));
                assert!(f.version >= FOB_VERSION);
                assert!(!f.chars.is_empty());
                // every range resolves to its own code
                let font = CBaseFont::new(0, f.clone(), 1, (256, 256));
                for c in &f.chars {
                    assert_eq!(font.find_character(c.code).map(|x| x.code), Some(c.code), "{} {:x}", e.name, c.code);
                }
                n += 1;
            }
        }
        assert_eq!(n, 29);
    }
}
