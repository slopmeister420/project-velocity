//! line-based text config reader used by every `.txt` / `.ini` / `.cfg` loader (CConfig, EventDatabase track
//! loading, FE3D*, camera manager, audio track list ...). Lives in the Handy\VsStringAnsi.cpp object range
//! (0x2B2D80..0x2B4690); the class name is not in the binary.
//!
//! blank lines and lines starting (after whitespace) with `#` or `//` are skipped. numbers are read from the first
//! `(` on a line: `( 1.5, 2, 3 )`; `key "value"` lines give a key up to the first space and a quoted value.

use crate::file::platform;
use crate::file::vs_file_directory::VsFileDirectoryDetails;

/// fixed line buffer of the readers (`char[256]`)
pub const LINE_MAX: usize = 256;

#[derive(Debug, Default)]
pub struct VsTextFile {
    pub name: String,  // +0x10
    pub buffer: Vec<u8>, // +0x04 / +0x08
    pub cursor: usize, // +0x0C
}

impl VsTextFile {
    pub fn from_bytes(name: &str, buffer: Vec<u8>) -> Self {
        VsTextFile { name: name.to_string(), buffer, cursor: 0 }
    }

    /// VsStringAnsi.cpp 0x002B2E30 — load from a file directory (pack or folder) or a plain game path.
    pub fn load(&mut self, dir: Option<&VsFileDirectoryDetails>, name: &str) -> bool {
        if name.eq_ignore_ascii_case(&self.name) && !self.buffer.is_empty() {
            self.cursor = 0;
            return true;
        }
        let r = match dir {
            Some(d) => d.load_file(name).ok(),
            None => platform::read(name).ok(),
        };
        self.name = name.to_string();
        match r {
            Some(b) => {
                self.buffer = b;
                self.cursor = 0;
                true
            }
            None => {
                log::warn!("Error loading text file {}", name);
                false
            }
        }
    }

    fn end(&self) -> usize {
        self.buffer.len()
    }

    fn at(&self, i: usize) -> u8 {
        self.buffer.get(i).copied().unwrap_or(0)
    }

    /// VsStringAnsi.cpp 0x002B3A90 — skip blank and comment lines.
    pub fn skip_blank_and_comments(&mut self) {
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

    /// VsStringAnsi.cpp 0x002B3950 — next non-comment line (at most `max - 1` chars). the returned count misses
    /// the last character when the line runs to end of file (original off-by-one).
    pub fn read_line_raw(&mut self, max: usize) -> (String, usize) {
        self.skip_blank_and_comments();
        if self.cursor >= self.end() {
            return (String::new(), 0);
        }
        let mut out = Vec::new();
        let mut n = 0;
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
        (out.iter().map(|&c| c as char).collect(), n)
    }

    /// next line, `None` where the original reports 0 characters.
    pub fn read_line(&mut self) -> Option<String> {
        let (s, n) = self.read_line_raw(LINE_MAX);
        (n != 0).then_some(s)
    }

    fn read_paren(&mut self, n: usize) -> Option<Result<Vec<f32>, Vec<f32>>> {
        let line = self.read_line()?;
        let Some(p) = line.find('(') else {
            log::warn!("malformed input file \"{}\" in line \"{}\"", self.name, line);
            return Some(Err(vec![0.0; n]));
        };
        let vals = scan_paren(&line[p..], n);
        if vals.len() >= n {
            Some(Ok(vals))
        } else {
            log::warn!("malformed input file \"{}\" in line \"{}\"", self.name, line);
            let mut v = vals;
            v.resize(n, 0.0);
            Some(Err(v))
        }
    }

    /// VsStringAnsi.cpp 0x002B3000 — `( %d`; `None` at end of file, `Some((ok, value))` otherwise.
    pub fn read_s32(&mut self) -> Option<(bool, i32)> {
        let line = self.read_line()?;
        let v = line.find('(').and_then(|p| {
            let s = line[p + 1..].trim_start();
            scan_int(s)
        });
        if v.is_none() {
            log::warn!("Error: (ReadSingleNumber - S32) malformed input file \"{}\" in line \"{}\"", self.name, line);
        }
        Some((v.is_some(), v.unwrap_or(0)))
    }

    /// VsStringAnsi.cpp 0x002B3190 — `( %f`.
    pub fn read_f32(&mut self) -> Option<(bool, f32)> {
        self.read_paren(1).map(|r| match r {
            Ok(v) => (true, v[0]),
            Err(v) => (false, v[0]),
        })
    }

    /// VsStringAnsi.cpp 0x002B3320 / 0x002B3520 / 0x002B3730 — `( %f, %f[, %f[, %f]]`.
    pub fn read_f32n<const N: usize>(&mut self) -> Option<(bool, [f32; N])> {
        self.read_paren(N).map(|r| {
            let (ok, v) = match r {
                Ok(v) => (true, v),
                Err(v) => (false, v),
            };
            let mut a = [0.0; N];
            a.copy_from_slice(&v[..N]);
            (ok, a)
        })
    }

    /// VsStringAnsi.cpp 0x002B3CE0 — value after the first space/tab on the line (`key 12`).
    pub fn read_line_value_s32(&mut self) -> Option<(bool, i32)> {
        let line = self.read_line()?;
        let rest = line.find([' ', '\t']).map(|p| &line[p..]).unwrap_or("");
        let v = scan_int(rest.trim_start());
        Some((v.is_some(), v.unwrap_or(0)))
    }

    /// VsStringAnsi.cpp 0x002B3E20 — `key 1.5`.
    pub fn read_line_value_f32(&mut self) -> Option<(bool, f32)> {
        let line = self.read_line()?;
        let rest = line.find([' ', '\t']).map(|p| &line[p..]).unwrap_or("");
        let v = scan_float(rest.trim_start()).map(|(f, _)| f);
        Some((v.is_some(), v.unwrap_or(0.0)))
    }

    /// VsStringAnsi.cpp 0x002B3F60 — first quoted string on the next line (empty if the line has none).
    pub fn read_string(&mut self) -> Option<String> {
        self.skip_blank_and_comments();
        if self.cursor >= self.end() {
            return None;
        }
        let mut out = String::new();
        while self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
            if self.at(self.cursor) == b'"' {
                self.cursor += 1;
                while self.cursor < self.end() && self.at(self.cursor) != b'"' {
                    out.push(self.at(self.cursor) as char);
                    self.cursor += 1;
                }
                self.skip_rest_of_line();
                return Some(out);
            }
            self.cursor += 1;
            if self.cursor >= self.end() {
                return Some(out);
            }
        }
        Some(out)
    }

    fn skip_rest_of_line(&mut self) {
        while self.cursor < self.end() && self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
            self.cursor += 1;
        }
        while self.cursor < self.end() && (self.at(self.cursor) == b'\r' || self.at(self.cursor) == b'\n') {
            self.cursor += 1;
        }
    }

    /// VsStringAnsi.cpp 0x002B40E0 / 0x002B42C0 — `KEY "value"`: key up to the first space, quoted value.
    pub fn read_key_value(&mut self) -> Option<(String, String)> {
        self.skip_blank_and_comments();
        if self.cursor >= self.end() {
            return None;
        }
        let mut key = String::new();
        while self.cursor < self.end() && self.at(self.cursor) != b' ' {
            key.push(self.at(self.cursor) as char);
            self.cursor += 1;
        }
        let mut val = String::new();
        while self.cursor < self.end() && self.at(self.cursor) != b'\r' && self.at(self.cursor) != b'\n' {
            if self.at(self.cursor) == b'"' {
                self.cursor += 1;
                while self.cursor < self.end() && self.at(self.cursor) != b'"' {
                    val.push(self.at(self.cursor) as char);
                    self.cursor += 1;
                }
                self.skip_rest_of_line();
                return Some((key, val));
            }
            self.cursor += 1;
        }
        Some((key, val))
    }

    pub fn at_end(&mut self) -> bool {
        self.skip_blank_and_comments();
        self.cursor >= self.end()
    }
}

/// `sscanf(s, "( %f, %f, ...")` starting at the `(`.
fn scan_paren(s: &str, n: usize) -> Vec<f32> {
    let mut out = Vec::new();
    let mut rest = &s[1..];
    for i in 0..n {
        if i > 0 {
            rest = rest.trim_start_matches([' ', '\t']);
            match rest.strip_prefix(',') {
                Some(r) => rest = r,
                None => break,
            }
        }
        match scan_float(rest.trim_start()) {
            Some((f, used)) => {
                out.push(f);
                rest = &rest.trim_start()[used..];
            }
            None => break,
        }
    }
    out
}

/// crt `%d`.
pub fn scan_int(s: &str) -> Option<i32> {
    let b = s.as_bytes();
    let mut i = 0;
    let neg = match b.first() {
        Some(b'-') => {
            i += 1;
            true
        }
        Some(b'+') => {
            i += 1;
            false
        }
        _ => false,
    };
    let start = i;
    let mut v: i64 = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        v = v * 10 + (b[i] - b'0') as i64;
        i += 1;
    }
    (i > start).then(|| (if neg { -v } else { v }) as i32)
}

/// crt `%f`: returns the value and the bytes consumed.
pub fn scan_float(s: &str) -> Option<(f32, usize)> {
    let b = s.as_bytes();
    let mut i = 0;
    if matches!(b.first(), Some(b'-' | b'+')) {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - ds;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - fs;
    }
    if digits == 0 {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if matches!(b.get(j), Some(b'-' | b'+')) {
            j += 1;
        }
        let es = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > es {
            i = j;
        }
    }
    s[..i].parse::<f64>().ok().map(|f| (f as f32, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads() {
        let t = b"# comment\r\n\r\n  // c2\r\nSpeed ( 1.5 )\r\nPos ( 1, -2.5, 3e1 )\r\nCount ( 7 )\r\nNAME \"Hello World\" x\r\nLAST 42\r\n";
        let mut f = VsTextFile::from_bytes("t", t.to_vec());
        assert_eq!(f.read_f32(), Some((true, 1.5)));
        assert_eq!(f.read_f32n::<3>(), Some((true, [1.0, -2.5, 30.0])));
        assert_eq!(f.read_s32(), Some((true, 7)));
        assert_eq!(f.read_key_value(), Some(("NAME".into(), "Hello World".into())));
        assert_eq!(f.read_line_value_s32(), Some((true, 42)));
        assert!(f.at_end());
    }
}
