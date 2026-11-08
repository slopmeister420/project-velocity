//! Text config reader used by the ai data files (`AI/Handling/*.txt`, `AI/ResCurve/*.txt`).
//!
//! Stand-in for the visos `Handy/VsStringAnsi.cpp` text-file reader until core publishes it; the
//! line/comment/number rules below are ported from the debug xbe so parsing matches the game.

use std::path::Path;

/// One loaded text file with a read cursor (`+0x04 data`, `+0x08 size`, `+0x0C cursor`).
#[derive(Debug, Clone, Default)]
pub struct VsTextFile {
    pub name: String,
    data: Vec<u8>,
    cursor: usize,
}

impl VsTextFile {
    pub fn from_bytes(name: &str, data: Vec<u8>) -> Self {
        Self { name: name.to_owned(), data, cursor: 0 }
    }

    /// VsStringAnsi.cpp 0x002B2E30
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Ok(Self::from_bytes(&name, data))
    }

    fn end(&self) -> usize {
        self.data.len()
    }

    fn at(&self, i: usize) -> u8 {
        self.data.get(i).copied().unwrap_or(0)
    }

    /// skips blank lines and lines starting with `#` or `//`.
    /// VsStringAnsi.cpp 0x002B3A90
    fn skip_comments(&mut self) {
        while self.cursor < self.end() {
            let mut i = self.cursor;
            while i < self.end() && self.at(i) != b'\r' && self.at(i) != b'\n' {
                i += 1;
            }
            while i < self.end() && (self.at(i) == b'\r' || self.at(i) == b'\n') {
                i += 1;
            }
            let mut j = self.cursor;
            while j < i && matches!(self.at(j), b' ' | b'\t' | b'\n' | b'\r') {
                j += 1;
            }
            let blank = j == i;
            let comment = self.at(j) == b'#' || (self.at(j) == b'/' && self.at(j + 1) == b'/');
            if !blank && !comment {
                break;
            }
            self.cursor = i;
        }
    }

    /// reads the next data line (at most `max - 1` chars); returns None at eof.
    /// VsStringAnsi.cpp 0x002B3950
    pub fn read_line(&mut self, max: usize) -> Option<String> {
        self.skip_comments();
        if self.cursor >= self.end() {
            return None;
        }
        let mut out = Vec::new();
        let mut n = 0usize;
        loop {
            let c = self.at(self.cursor);
            if c == b'\r' || c == b'\n' {
                break;
            }
            out.push(c);
            self.cursor += 1;
            if self.cursor >= self.end() {
                break;
            }
            n += 1;
            if n >= max - 1 {
                break;
            }
        }
        while self.cursor < self.end() && self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
            self.cursor += 1;
        }
        while self.cursor < self.end() && (self.at(self.cursor) == b'\r' || self.at(self.cursor) == b'\n') {
            self.cursor += 1;
        }
        // the original returns the char count, so an empty line reads as "nothing"
        if n == 0 {
            return None;
        }
        Some(String::from_utf8_lossy(&out).into_owned())
    }

    /// `( %f` after the first `(` on the next data line.
    /// VsStringAnsi.cpp 0x002B3190
    pub fn read_single_number(&mut self) -> Option<f32> {
        let line = self.read_line(256)?;
        let Some(p) = line.find('(') else {
            log::warn!("Error: (ReadSingleNumber - F32) malformed input file \"{}\" in line \"{line}\"", self.name);
            return Some(0.0);
        };
        let mut s = Scan::new(&line[p..]);
        if s.lit(b'(') && let Some(v) = s.float() {
            return Some(v);
        }
        log::warn!("Error: (ReadSingleNumber - F32) malformed input file \"{}\" in line \"{line}\"", self.name);
        Some(0.0)
    }

    /// `( %f, %f` after the first `(` on the next data line; missing values read as 0.
    /// VsStringAnsi.cpp 0x002B3320
    pub fn read_double_number(&mut self) -> Option<(f32, f32)> {
        let line = self.read_line(256)?;
        let Some(p) = line.find('(') else {
            log::warn!("Error: (ReadDoubleNumber) malformed input file \"{}\" in line \"{line}\"", self.name);
            return Some((0.0, 0.0));
        };
        let mut s = Scan::new(&line[p..]);
        let mut v = [0.0f32; 2];
        let mut got = 0;
        if s.lit(b'(') && let Some(a) = s.float() {
            v[0] = a;
            got = 1;
            if s.lit(b',') && let Some(b) = s.float() {
                v[1] = b;
                got = 2;
            }
        }
        if got < 2 {
            log::warn!("Error: (ReadDoubleNumber) malformed input file \"{}\" in line \"{line}\"", self.name);
            for x in v.iter_mut().skip(got) {
                *x = 0.0;
            }
        }
        Some((v[0], v[1]))
    }

    /// the text between the first pair of double quotes on the next data line.
    /// VsStringAnsi.cpp 0x002B3F60
    pub fn read_string(&mut self) -> Option<String> {
        self.skip_comments();
        if self.cursor >= self.end() {
            return None;
        }
        while self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
            if self.at(self.cursor) == b'"' {
                self.cursor += 1;
                let mut out = Vec::new();
                while self.cursor < self.end() && self.at(self.cursor) != b'"' {
                    out.push(self.at(self.cursor));
                    self.cursor += 1;
                }
                while self.cursor < self.end() && self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
                    self.cursor += 1;
                }
                while self.cursor < self.end() && (self.at(self.cursor) == b'\r' || self.at(self.cursor) == b'\n') {
                    self.cursor += 1;
                }
                return Some(String::from_utf8_lossy(&out).into_owned());
            }
            self.cursor += 1;
            if self.cursor >= self.end() {
                return Some(String::new());
            }
        }
        Some(String::new())
    }
}

/// the subset of `sscanf` the readers use: literal chars (with whitespace skipping) and `%f`.
struct Scan<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Scan<'a> {
    fn new(s: &'a str) -> Self {
        Self { s: s.as_bytes(), i: 0 }
    }
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn lit(&mut self, c: u8) -> bool {
        self.ws();
        if self.i < self.s.len() && self.s[self.i] == c {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn float(&mut self) -> Option<f32> {
        self.ws();
        let start = self.i;
        let s = self.s;
        let mut i = self.i;
        if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
            i += 1;
        }
        let digits = i;
        while i < s.len() && s[i].is_ascii_digit() {
            i += 1;
        }
        if i < s.len() && s[i] == b'.' {
            i += 1;
            while i < s.len() && s[i].is_ascii_digit() {
                i += 1;
            }
        }
        if i == digits || (i == digits + 1 && s[digits] == b'.') {
            return None;
        }
        if i < s.len() && (s[i] == b'e' || s[i] == b'E') {
            let mut j = i + 1;
            if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
                j += 1;
            }
            if j < s.len() && s[j].is_ascii_digit() {
                while j < s.len() && s[j].is_ascii_digit() {
                    j += 1;
                }
                i = j;
            }
        }
        let v = std::str::from_utf8(&s[start..i]).ok()?.parse::<f32>().ok()?;
        self.i = i;
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_strings() {
        let src = b"// c\r\n\r\n\"350Z\"\r\n# x\r\n(60.0)\r\n(0.125,\t0.000030517578125) \r\n(1.0,\t1.0 )";
        let mut f = VsTextFile::from_bytes("t", src.to_vec());
        assert_eq!(f.read_string().as_deref(), Some("350Z"));
        assert_eq!(f.read_single_number(), Some(60.0));
        assert_eq!(f.read_double_number(), Some((0.125, 0.000030517578125)));
        assert_eq!(f.read_double_number(), Some((1.0, 1.0)));
        assert_eq!(f.read_single_number(), None);
    }
}
