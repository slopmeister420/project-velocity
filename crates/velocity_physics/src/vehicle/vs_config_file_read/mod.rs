//! src/Physics/Vehicle/VsConfigFileRead.cpp — the FastCar config text reader: a line based
//! preprocessor (`define`, macro substitution, `//` and `#` comments), constant-expression
//! evaluation of every whitespace separated token, and typed readers that expect the next
//! significant line to start with a given parameter name.

use crate::fastcar::expression::{self, Image};

/// Define table: names kept sorted with the original binary search (`this+8` names, `this+12` values).
#[derive(Clone, Debug, Default)]
pub struct Defines {
    names: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
}

fn strcmp(a: &[u8], b: &[u8]) -> i32 {
    match a.cmp(b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl Defines {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// the search loop shared by 1B56A0 and 1B5B20: returns (index, last compare)
    fn search(&self, key: &[u8]) -> (usize, i32) {
        let n = self.names.len() as i32;
        let mut lo = 0i32;
        let mut hi = n - 1;
        let mut mid = hi / 2;
        while lo < hi {
            if strcmp(key, &self.names[mid as usize]) <= 0 {
                if hi == mid {
                    hi -= 1;
                } else {
                    hi = mid;
                }
            } else if lo == mid {
                lo += 1;
            } else {
                lo = mid;
            }
            mid = (hi + lo) / 2;
        }
        (mid as usize, strcmp(key, &self.names[mid as usize]))
    }

    /// VsConfigFileRead.cpp 0x001B56A0
    pub fn define(&mut self, name: &str, value: &str) {
        self.define_bytes(name.as_bytes(), value.as_bytes());
    }

    pub fn define_bytes(&mut self, name: &[u8], value: &[u8]) {
        if self.names.is_empty() {
            self.names.push(name.to_vec());
            self.values.push(value.to_vec());
            return;
        }
        let (mut idx, c) = self.search(name);
        if c != 0 {
            if c > 0 {
                idx += 1;
            }
            self.names.insert(idx, name.to_vec());
            self.values.insert(idx, value.to_vec());
        } else {
            self.values[idx] = value.to_vec();
        }
    }

    /// lookup used by the preprocessor (`None` when the search ends on a different name)
    fn find(&self, key: &[u8]) -> Option<usize> {
        if self.names.is_empty() {
            return None;
        }
        let (idx, c) = self.search(key);
        (c == 0).then_some(idx)
    }

    /// VsConfigFileRead.cpp 0x001B59C0
    pub fn value(&self, name: &str) -> Option<&str> {
        self.names
            .iter()
            .position(|n| n.as_slice() == name.as_bytes())
            .and_then(|i| std::str::from_utf8(&self.values[i]).ok())
    }
}

/// `VsStringAnsi` text file as used by the config reader (`VsStringAnsi__2B2E30` load,
/// `VsStringAnsi__2B3950` read line).
#[derive(Clone, Debug)]
pub struct TextFile {
    data: Vec<u8>,
    pos: usize,
}

impl TextFile {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    /// Loads a game-relative path (`Vehicles\cfgs\Main.cfg`).
    pub fn open(path: &str) -> Option<Self> {
        visos::file::platform::read(path).ok().map(Self::new)
    }

    /// VsStringAnsi::ReadLine: copies up to `max-1` bytes, stops at CR/LF, then skips the rest of
    /// the line and every following CR/LF. Returns the count the original returns (0 = end).
    fn read_line(&mut self, max: usize) -> (usize, Vec<u8>) {
        let end = self.data.len();
        if self.pos >= end {
            return (0, vec![]);
        }
        let mut out = vec![];
        let mut n = 0usize;
        loop {
            let c = self.data[self.pos];
            if c == 13 || c == 10 {
                break;
            }
            out.push(c);
            self.pos += 1;
            if self.pos >= end {
                break;
            }
            n += 1;
            if n >= max - 1 {
                break;
            }
        }
        while self.pos < end && self.data[self.pos] != 13 && self.data[self.pos] != 10 {
            self.pos += 1;
        }
        while self.pos < end && (self.data[self.pos] == 13 || self.data[self.pos] == 10) {
            self.pos += 1;
        }
        (n, out)
    }
}

fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn strstr(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// `%.7g` as the XDK CRT prints a float promoted to double (round half up on the exact digits).
pub fn fmt_g7(v: f32) -> String {
    fmt_g(v as f64, 7)
}

/// `%g` with the given precision (exponent printed with three digits, as MSVC does).
pub fn fmt_g(v: f64, prec: usize) -> String {
    if v == 0.0 {
        return if v.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    if !v.is_finite() {
        return if v.is_nan() { "-1.#IND".into() } else if v > 0.0 { "1.#INF".into() } else { "-1.#INF".into() };
    }
    let neg = v < 0.0;
    // exact decimal expansion of the double
    let s = format!("{:.1100e}", v.abs());
    let (mant, exp) = s.split_once('e').unwrap();
    let mut exp: i32 = exp.parse().unwrap();
    let digits: Vec<u8> = mant.bytes().filter(|c| c.is_ascii_digit()).collect();
    let mut d: Vec<u8> = digits[..prec].to_vec();
    if digits[prec] >= b'5' {
        let mut i = prec;
        loop {
            if i == 0 {
                d.insert(0, b'1');
                d.pop();
                exp += 1;
                break;
            }
            i -= 1;
            if d[i] == b'9' {
                d[i] = b'0';
            } else {
                d[i] += 1;
                break;
            }
        }
    }
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    if exp < -4 || exp >= prec as i32 {
        out.push(d[0] as char);
        let mut frac: Vec<u8> = d[1..].to_vec();
        while frac.last() == Some(&b'0') {
            frac.pop();
        }
        if !frac.is_empty() {
            out.push('.');
            out.extend(frac.iter().map(|&c| c as char));
        }
        out.push_str(&format!("e{}{:03}", if exp < 0 { '-' } else { '+' }, exp.abs()));
    } else if exp >= 0 {
        let ip = (exp + 1) as usize;
        out.extend(d[..ip].iter().map(|&c| c as char));
        let mut frac: Vec<u8> = d[ip..].to_vec();
        while frac.last() == Some(&b'0') {
            frac.pop();
        }
        if !frac.is_empty() {
            out.push('.');
            out.extend(frac.iter().map(|&c| c as char));
        }
    } else {
        out.push_str("0.");
        for _ in 0..(-exp - 1) {
            out.push('0');
        }
        let mut frac = d.clone();
        while frac.last() == Some(&b'0') {
            frac.pop();
        }
        out.extend(frac.iter().map(|&c| c as char));
    }
    out
}

/// VsConfigFileRead.cpp 0x001B5B20 — macro substitution and `define` handling.
/// Returns false when the line was consumed (a define) or is a comment.
pub fn preprocess(defs: &mut Defines, line: &mut Vec<u8>) -> bool {
    if !defs.is_empty() {
        loop {
            let mut substituted = false;
            let mut p = 0usize;
            let at = |l: &Vec<u8>, i: usize| -> u8 { l.get(i).copied().unwrap_or(0) };
            while at(line, p) != 0 && !(at(line, p) == b'/' && at(line, p + 1) == b'/') {
                while at(line, p) != 0 && !at(line, p).is_ascii_alphanumeric() && at(line, p) != b'_' {
                    p += 1;
                }
                if at(line, p) == 0 || (at(line, p) == b'/' && at(line, p + 1) == b'/') {
                    break;
                }
                let mut q = p;
                while at(line, q).is_ascii_digit() {
                    q += 1;
                }
                if at(line, q).is_ascii_alphabetic() || at(line, q) == b'_' {
                    while is_ident(at(line, q)) {
                        q += 1;
                    }
                    let key = line[p..q].to_vec();
                    match defs.find(&key) {
                        None => p = q,
                        Some(idx) => {
                            // do not expand the name being (re)defined
                            if let Some(d) = strstr(line, b"define") {
                                let mut r = d + 7;
                                while at(line, r) == b' ' || at(line, r) == b'\t' {
                                    r += 1;
                                }
                                let name = &defs.names[idx];
                                if r <= line.len() && strstr(&line[r..], name) == Some(0) {
                                    let after = r + name.len();
                                    if !is_ident(at(line, after)) {
                                        break;
                                    }
                                }
                            }
                            let value = defs.values[idx].clone();
                            let mut nl = line[..p].to_vec();
                            nl.extend_from_slice(&value);
                            nl.extend_from_slice(&line[q..]);
                            *line = nl;
                            substituted = true;
                            if value.as_slice() == b"//" {
                                break;
                            }
                            p += value.len();
                        }
                    }
                } else {
                    p = q;
                }
            }
            if !substituted {
                break;
            }
        }
    }
    let mut s = 0;
    while s < line.len() && (line[s] == b' ' || line[s] == b'\t') {
        s += 1;
    }
    let rest = &line[s..];
    if rest.starts_with(b"define") && rest.len() > 6 && (rest[6] == b' ' || rest[6] == b'\t') {
        // sscanf(line+6, " %s", name)
        let mut a = 6;
        while a < rest.len() && is_space(rest[a]) {
            a += 1;
        }
        let mut b = a;
        while b < rest.len() && !is_space(rest[b]) {
            b += 1;
        }
        let name = rest[a..b].to_vec();
        let mut v = strstr(line, &name).unwrap_or(0) + name.len();
        while v < line.len() && (line[v] == b' ' || line[v] == b'\t') {
            v += 1;
        }
        let mut value = line[v.min(line.len())..].to_vec();
        if let Some(i) = value.iter().position(|&c| c == b'\n') {
            value.truncate(i);
        }
        if let Some(i) = value.iter().position(|&c| c == b'\r') {
            value.truncate(i);
        }
        while matches!(value.last(), Some(b' ') | Some(b'\t')) {
            value.pop();
        }
        defs.define_bytes(&name, &value);
        return false;
    }
    let c0 = rest.first().copied().unwrap_or(0);
    let c1 = rest.get(1).copied().unwrap_or(0);
    c0 != b'#' && c0 != b'\n' && !(c0 == b'/' && c1 == b'/')
}

/// VsConfigFileRead.cpp 0x001B6140 — replace every token that is a constant expression by its
/// `%.7g` value.
pub fn evaluate_tokens(line: &mut Vec<u8>) {
    let mut p = 0usize;
    loop {
        while p < line.len() && matches!(line[p], b' ' | b'\t' | b'\n') {
            p += 1;
        }
        if p >= line.len() || line[p] == 0 {
            break;
        }
        let start = p;
        while p < line.len() && !matches!(line[p], 0 | b' ' | b'\t' | b'\n') {
            p += 1;
        }
        let tok = String::from_utf8_lossy(&line[start..p]).into_owned();
        if let Some(v) = expression::eval_const(&tok) {
            let num = fmt_g7(v);
            let mut nl = line[..start].to_vec();
            nl.extend_from_slice(num.as_bytes());
            nl.extend_from_slice(&line[p..]);
            *line = nl;
            p = start + num.len();
        }
    }
}

/// Reader state for one file (`VsStringAnsi` + the 1024 byte line buffer).
pub struct ConfigReader<'a> {
    pub file: TextFile,
    pub defines: &'a mut Defines,
    /// last line returned (the "possible culprit line" of the error messages)
    pub line: Vec<u8>,
}

/// read failure, carrying the parameter and the culprit line
#[derive(Clone, Debug, thiserror::Error)]
#[error("cannot read \"{name}\" (line \"{line}\")")]
pub struct ReadError {
    pub name: String,
    pub line: String,
}

/// one conversion of a C scanf: skips whitespace, returns (value text consumed?) via callbacks
fn skip_ws(s: &[u8], mut i: usize) -> usize {
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    i
}

/// MSVC scanf `%f`: sign, digits, '.', digits, optional e/E exponent
fn scan_f(s: &[u8], i: usize) -> Option<(f32, usize)> {
    let mut j = i;
    let mut t = String::new();
    if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
        t.push(s[j] as char);
        j += 1;
    }
    let mut dig = 0;
    while j < s.len() && s[j].is_ascii_digit() {
        t.push(s[j] as char);
        j += 1;
        dig += 1;
    }
    if j < s.len() && s[j] == b'.' {
        t.push('.');
        j += 1;
        while j < s.len() && s[j].is_ascii_digit() {
            t.push(s[j] as char);
            j += 1;
            dig += 1;
        }
    }
    if dig == 0 {
        return None;
    }
    if j < s.len() && (s[j] == b'e' || s[j] == b'E') {
        let mut k = j + 1;
        let mut e = String::from("e");
        if k < s.len() && (s[k] == b'+' || s[k] == b'-') {
            e.push(s[k] as char);
            k += 1;
        }
        let d0 = k;
        while k < s.len() && s[k].is_ascii_digit() {
            e.push(s[k] as char);
            k += 1;
        }
        if k > d0 {
            t.push_str(&e);
            j = k;
        }
    }
    // the CRT converts to double, then the assignment rounds to float
    Some((t.parse::<f64>().ok()? as f32, j))
}

/// MSVC scanf `%d`
fn scan_d(s: &[u8], i: usize) -> Option<(i32, usize)> {
    let mut j = i;
    let neg = j < s.len() && s[j] == b'-';
    if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
        j += 1;
    }
    let d0 = j;
    let mut v: i64 = 0;
    while j < s.len() && s[j].is_ascii_digit() {
        v = v.wrapping_mul(10).wrapping_add((s[j] - b'0') as i64);
        j += 1;
    }
    if j == d0 {
        return None;
    }
    Some(((if neg { -v } else { v }) as i32, j))
}

/// `sscanf(line, " name")`: returns the index after the literal, or None
fn scan_name(line: &[u8], name: &str) -> Option<usize> {
    let i = skip_ws(line, 0);
    let n = name.as_bytes();
    if line.len() >= i + n.len() && &line[i..i + n.len()] == n {
        Some(i + n.len())
    } else {
        None
    }
}

impl<'a> ConfigReader<'a> {
    pub fn new(file: TextFile, defines: &'a mut Defines) -> Self {
        Self { file, defines, line: vec![] }
    }

    /// VsConfigFileRead.cpp 0x001B6400 — next significant, preprocessed and evaluated line
    pub fn next_line(&mut self) -> bool {
        let mut n;
        loop {
            let (cnt, mut l) = self.file.read_line(1024);
            n = cnt;
            let mut skip = false;
            if n != 0 {
                let mut i = 0;
                while i < l.len() && (l[i] == b' ' || l[i] == b'\t') {
                    i += 1;
                }
                let c0 = l.get(i).copied().unwrap_or(0);
                let c1 = l.get(i + 1).copied().unwrap_or(0);
                if c0 == b'#' || c0 == b'\n' || (c0 == b'/' && c1 == b'/') {
                    skip = true;
                }
                if !skip {
                    while i < l.len() && l[i] != 0 && l[i] != b'#' && l[i] != b'\n' && !(l[i] == b'/' && l.get(i + 1) == Some(&b'/')) {
                        i += 1;
                    }
                    l.truncate(i);
                    // the trailing-whitespace trim that follows starts below the string: a no-op
                    if !preprocess(self.defines, &mut l) {
                        skip = true;
                    }
                }
            }
            self.line = l;
            if !skip {
                break;
            }
        }
        if n != 0 {
            evaluate_tokens(&mut self.line);
        }
        n != 0
    }

    fn fail(&self, what: &str, name: &str) -> ReadError {
        log::warn!(" Error: cannot read {} \"{}\"", what, name);
        if std::env::var_os("VELOCITY_CFG_TRACE").is_some() { eprintln!("cannot read {what} {name}: {:?}", String::from_utf8_lossy(&self.line)); }
        log::warn!("  possible culprit line: \"{}\"", String::from_utf8_lossy(&self.line));
        ReadError { name: name.into(), line: String::from_utf8_lossy(&self.line).into() }
    }

    /// VsConfigFileRead.cpp 0x001B6570
    pub fn read_ints(&mut self, dst: &mut [i32], name: &str) -> Result<(), ReadError> {
        if self.next_line() {
            if let Some(mut i) = scan_name(&self.line, name) {
                let mut got = vec![];
                for _ in 0..dst.len() {
                    i = skip_ws(&self.line, i);
                    match scan_d(&self.line, i) {
                        Some((v, j)) => {
                            got.push(v);
                            i = j;
                        }
                        None => break,
                    }
                }
                if got.len() == dst.len() {
                    dst.copy_from_slice(&got);
                    return Ok(());
                }
            }
        }
        Err(self.fail(&format!("array of {} integers", dst.len()), name))
    }

    /// VsConfigFileRead.cpp 0x001B6710
    pub fn read_reals(&mut self, dst: &mut [f32], name: &str) -> Result<(), ReadError> {
        if self.next_line() {
            if let Some(mut i) = scan_name(&self.line, name) {
                let mut got = vec![];
                for _ in 0..dst.len() {
                    i = skip_ws(&self.line, i);
                    match scan_f(&self.line, i) {
                        Some((v, j)) => {
                            got.push(v);
                            i = j;
                        }
                        None => break,
                    }
                }
                if got.len() == dst.len() {
                    dst.copy_from_slice(&got);
                    return Ok(());
                }
            }
        }
        Err(self.fail(&format!("array of {} reals", dst.len()), name))
    }

    pub fn read_int(&mut self, dst: &mut i32, name: &str) -> Result<(), ReadError> {
        self.read_ints(std::slice::from_mut(dst), name)
    }

    pub fn read_real(&mut self, dst: &mut f32, name: &str) -> Result<(), ReadError> {
        self.read_reals(std::slice::from_mut(dst), name)
    }

    /// VsConfigFileRead.cpp 0x001B68B0 — `" name %s"`, then the text between the first two quotes
    pub fn read_string(&mut self, name: &str) -> Result<String, ReadError> {
        if self.next_line() {
            if let Some(i) = scan_name(&self.line, name) {
                let a = skip_ws(&self.line, i);
                let mut b = a;
                while b < self.line.len() && !is_space(self.line[b]) {
                    b += 1;
                }
                if b > a {
                    let tok = &self.line[a..b];
                    // VsConfigFileRead.cpp 0x001B6390
                    let s = match tok.iter().position(|&c| c == b'"') {
                        None => Some(tok.to_vec()),
                        Some(q) => tok[q + 1..].iter().position(|&c| c == b'"').map(|e| tok[q + 1..q + 1 + e].to_vec()),
                    };
                    if let Some(s) = s {
                        return Ok(String::from_utf8_lossy(&s).into_owned());
                    }
                }
            }
        }
        Err(self.fail("string", name))
    }

    /// VsConfigFileRead.cpp 0x001B6A20 — string compiled into an optimised expression image
    pub fn read_expression(&mut self, name: &str, vars: &str) -> Result<Expression, ReadError> {
        let text = self.read_string(name)?;
        match expression::compile(&text, vars) {
            Ok(mut image) => {
                image.optimize();
                Ok(Expression { text, image })
            }
            Err(e) => {
                log::warn!(" Error: Could not parse the expression \"{}\".{}\n  Expression parser reporded error: \"{}\"", name, text, e);
                Err(ReadError { name: name.into(), line: text })
            }
        }
    }
}

/// A config expression: source text and its compiled image.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expression {
    pub text: String,
    pub image: Image,
}

impl Expression {
    /// `AUDIO_Resample__3AAFF0(&out, 0, image, vars)`; None when the evaluator reports an error
    pub fn eval(&self, vars: &[f32]) -> Option<f32> {
        self.image.evaluate(vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g7_formatting() {
        assert_eq!(fmt_g7(0.03), "0.03");
        assert_eq!(fmt_g7(1.0 / 3.6), "0.2777778");
        assert_eq!(fmt_g7(1234567.5), "1234568");
        assert_eq!(fmt_g7(1.0e-5), "1e-005");
        assert_eq!(fmt_g7(745.712), "745.712");
        assert_eq!(fmt_g7(-0.5), "-0.5");
        assert_eq!(fmt_g7(1.0e7), "1e+007");
    }

    #[test]
    fn define_and_substitute() {
        let mut d = Defines::new();
        d.define("LC", "");
        d.define("GC", "//");
        d.define("240SX", "");
        let mut l = b"240SX   define MASS 1721".to_vec();
        assert!(!preprocess(&mut d, &mut l));
        assert_eq!(d.value("MASS"), Some("1721"));
        let mut l = b"LC mass MASS".to_vec();
        assert!(preprocess(&mut d, &mut l));
        evaluate_tokens(&mut l);
        assert_eq!(l, b" mass 1721");
        let mut l = b"GC gravity 0 -9.8 0".to_vec();
        assert!(!preprocess(&mut d, &mut l));
    }
}
#[cfg(feature = "tr")]
pub mod tr;
