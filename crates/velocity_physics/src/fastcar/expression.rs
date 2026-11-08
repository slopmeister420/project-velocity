//! FastCar expression parser (unattributed library object at 0x3A9000-0x3ABF10, linked into the
//! `_AUDIO_Resample` region): compiles an infix expression into a postfix "image" of f32 constants,
//! variable slots and operator calls, optimises it, and evaluates it on a float stack.
//!
//! The original parser scans right to left for the rightmost lowest-precedence operator; for every
//! valid expression that yields the standard left-associative postfix order, which is what
//! [`compile`] produces. Unary minus binds to the primary (so `-2^2` is `(-2)^2`), a signed number
//! is a single constant, all arithmetic is single precision (the original uses SSE scalar ops),
//! and division by `|b| <= 0.001` yields 0.

/// One image entry (`{u32 type, u32 value}` in the original: type 0 = call, 1 = const, 2 = variable).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    Call(Func),
    Const(f32),
    Var(usize),
}

/// Operators (`AUDIO_Resample__3A9940` codes 1..7) and table functions (`off_6CDA30`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Neg,
    Abs,
    Acos,
    Asin,
    Atan,
    Atan2,
    Cos,
    Exp,
    Max,
    Min,
    If,
    Not,
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
    Tan,
    Sqrt,
    Sin,
    Step,
    Deg,
    Rad,
    Log10,
}

/// Function table `off_6CDA30` {name, func, number of commas}; order matters for prefix matching.
const FUNCTIONS: [(&str, Func, usize); 24] = [
    ("abs", Func::Abs, 0),
    ("fabs", Func::Abs, 0),
    ("acos", Func::Acos, 0),
    ("asin", Func::Asin, 0),
    ("atan", Func::Atan, 0),
    ("atan2", Func::Atan2, 1),
    ("cos", Func::Cos, 0),
    ("exp", Func::Exp, 0),
    ("max", Func::Max, 1),
    ("min", Func::Min, 1),
    ("if", Func::If, 0),
    ("not", Func::Not, 0),
    ("eq", Func::Eq, 1),
    ("lt", Func::Lt, 1),
    ("lte", Func::Lte, 1),
    ("gt", Func::Gt, 1),
    ("gte", Func::Gte, 1),
    ("tan", Func::Tan, 0),
    ("sqrt", Func::Sqrt, 0),
    ("sin", Func::Sin, 0),
    ("step", Func::Step, 0),
    ("deg", Func::Deg, 0),
    ("rad", Func::Rad, 0),
    ("log10", Func::Log10, 0),
];

/// Constant table `off_6CDB50`.
const CONSTANTS: [(&str, u32); 2] = [("pi", 0x40490FDB), ("e", 0x402DF854)];

/// `deg()` factor `flt_625CD8`.
const DEG_FACTOR: u32 = 0x42652EE0;
/// `rad()` factor `dword_6253CC`.
const RAD_FACTOR: u32 = 0x3C8EFA35;

impl Func {
    /// number of stack operands consumed (result replaces them)
    fn arity(self) -> usize {
        match self {
            Func::Add | Func::Sub | Func::Mul | Func::Div | Func::Pow => 2,
            Func::Atan2 | Func::Max | Func::Min | Func::Eq | Func::Lt | Func::Lte | Func::Gt | Func::Gte => 2,
            _ => 1,
        }
    }

    /// `AUDIO_Resample__3AAD10`: comma count of a table function, -1 for the bare operators.
    fn table_args(self) -> i32 {
        match self {
            Func::Add | Func::Sub | Func::Mul | Func::Div | Func::Pow | Func::Neg => -1,
            f => f.arity() as i32 - 1,
        }
    }

    fn apply(self, s: &mut Vec<f32>) {
        let b = s.pop().unwrap_or(0.0);
        if self.arity() == 1 {
            s.push(unary(self, b));
            return;
        }
        let a = s.pop().unwrap_or(0.0);
        s.push(binary(self, a, b));
    }
}

fn wrap1(f: fn(f64) -> f64, x: f32) -> f32 {
    f(x as f64) as f32
}

/// `AUDIO_Resample__3A9000` .. `3A9830` (unary entries)
fn unary(f: Func, x: f32) -> f32 {
    match f {
        Func::Neg => 0.0 - x,
        Func::Abs => x.abs(),
        Func::Acos => wrap1(f64::acos, x),
        Func::Asin => wrap1(f64::asin, x),
        Func::Atan => wrap1(f64::atan, x),
        Func::Cos => wrap1(f64::cos, x),
        Func::Exp => wrap1(f64::exp, x),
        Func::Tan => wrap1(f64::tan, x),
        Func::Sqrt => wrap1(f64::sqrt, x),
        Func::Sin => wrap1(f64::sin, x),
        Func::Log10 => wrap1(f64::log10, x),
        Func::If => {
            if x == 0.0 {
                0.0
            } else {
                1.0
            }
        }
        Func::Not => {
            if x == 0.0 {
                1.0
            } else {
                0.0
            }
        }
        Func::Step => {
            if 0.0 > x {
                0.0
            } else {
                1.0
            }
        }
        Func::Deg => x * f32::from_bits(DEG_FACTOR),
        Func::Rad => x * f32::from_bits(RAD_FACTOR),
        _ => x,
    }
}

/// binary entries of the same table; NaN comparisons follow the ucomiss/comiss flag tests
fn binary(f: Func, a: f32, b: f32) -> f32 {
    let t = |c: bool| if c { 1.0 } else { 0.0 };
    match f {
        Func::Add => a + b,
        Func::Sub => a - b,
        Func::Mul => a * b,
        // 3A97A0: fabsf(b) <= 0.001f -> 0
        Func::Div => {
            if b.abs() <= 0.001f32 {
                0.0
            } else {
                a / b
            }
        }
        Func::Pow => (a as f64).powf(b as f64) as f32,
        Func::Atan2 => (a as f64).atan2(b as f64) as f32,
        // 3A92E0: a > b ? a : b
        Func::Max => {
            if a > b {
                a
            } else {
                b
            }
        }
        // 3A9370: b <= a ? b : a  (NaN keeps a)
        Func::Min => {
            if b <= a {
                b
            } else {
                a
            }
        }
        // ucomiss + jp: unordered counts as not equal
        Func::Eq => t(a == b),
        Func::Lt => t(a < b),
        Func::Lte => t(a <= b),
        Func::Gt => t(a > b),
        Func::Gte => t(a >= b),
        _ => b,
    }
}

/// Compiled expression ("image").
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Image {
    pub items: Vec<Item>,
}

/// Parser error codes as returned by `AUDIO_Resample__3AAD60`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExprError {
    #[error("{0}")]
    Syntax(String),
    #[error("Unpaired parenthesis!")]
    Paren,
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Unknown constant: {0}")]
    UnknownConstant(String),
    #[error("Wrong number of arguments in {0}")]
    Args(String),
}

fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

/// `AUDIO_Resample__3AB1B0`
fn is_op(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'*' | b'/' | b'^' | b',' | b'%')
}

/// MSVC `strtod` prefix scan: returns (value, bytes consumed). Accepts `d`/`D` exponents.
pub fn strtod(s: &[u8]) -> (f64, usize) {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let start = i;
    let mut txt = String::new();
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        txt.push(s[i] as char);
        i += 1;
    }
    let mut digits = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        txt.push(s[i] as char);
        i += 1;
        digits += 1;
    }
    if i < s.len() && s[i] == b'.' {
        txt.push('.');
        i += 1;
        while i < s.len() && s[i].is_ascii_digit() {
            txt.push(s[i] as char);
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return (0.0, 0);
    }
    if i < s.len() && matches!(s[i], b'e' | b'E' | b'd' | b'D') {
        let mut j = i + 1;
        let mut e = String::from("e");
        if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
            e.push(s[j] as char);
            j += 1;
        }
        let d0 = j;
        while j < s.len() && s[j].is_ascii_digit() {
            e.push(s[j] as char);
            j += 1;
        }
        if j > d0 {
            txt.push_str(&e);
            i = j;
        }
    }
    let _ = start;
    (txt.parse::<f64>().unwrap_or(0.0), i)
}

struct Parser<'a> {
    s: &'a [u8],
    vars: Vec<&'a [u8]>,
    out: Vec<Item>,
}

/// Split the variable list the way `AUDIO_Resample__3AAD60` does (whitespace separated, max 100).
fn split_vars(v: &str) -> Vec<&[u8]> {
    v.as_bytes()
        .split(|c| is_space(*c))
        .filter(|w| !w.is_empty())
        .take(100)
        .collect()
}

impl<'a> Parser<'a> {
    /// index of the matching ')' for the '(' at `open`, within [open, end)
    fn close_of(&self, open: usize, end: usize) -> Option<usize> {
        let mut d = 0;
        for i in open..end {
            match self.s[i] {
                b'(' => d += 1,
                b')' => {
                    d -= 1;
                    if d == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// rightmost top-level occurrence of a byte satisfying `pred` in (lo, hi) exclusive of lo,
    /// skipping parenthesised groups like the original right-to-left scan does
    fn rscan(&self, lo: usize, hi: usize, pred: &dyn Fn(&Self, usize) -> bool) -> Option<usize> {
        let mut i = hi;
        while i > lo + 1 {
            i -= 1;
            let c = self.s[i];
            if c == b')' {
                let mut d = 1;
                while i > lo {
                    i -= 1;
                    match self.s[i] {
                        b')' => d += 1,
                        b'(' => {
                            d -= 1;
                            if d == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }
            if pred(self, i) {
                return Some(i);
            }
        }
        None
    }

    /// '+'/'-' at `i` is binary: not part of an exponent, previous non-space char is no operator
    fn is_binary_sign(&self, base: usize, i: usize) -> bool {
        let c = self.s[i];
        if c != b'+' && c != b'-' {
            return false;
        }
        let m = i - base;
        if m >= 2 {
            let p = self.s[i - 1];
            let pp = self.s[i - 2];
            if matches!(p, b'e' | b'E' | b'd' | b'D') && (pp.is_ascii_digit() || pp == b'.') {
                return false;
            }
        }
        let mut n = i - 1;
        while is_space(self.s[n]) && n > base {
            n -= 1;
        }
        !is_op(self.s[n]) && !is_space(self.s[n])
    }

    /// emit postfix for s[lo..hi]
    fn expr(&mut self, lo: usize, hi: usize) -> Result<(), ExprError> {
        // the original never treats index 0 of a range as a binary operator: scan (lo, hi)
        if let Some(p) = self.rscan(lo, hi, &|me, i| me.is_binary_sign(lo, i)) {
            self.expr(lo, p)?;
            self.expr(p + 1, hi)?;
            self.out.push(Item::Call(if self.s[p] == b'+' { Func::Add } else { Func::Sub }));
            return Ok(());
        }
        if let Some(p) = self.rscan(lo, hi, &|me, i| me.s[i] == b'*' || me.s[i] == b'/') {
            self.expr(lo, p)?;
            self.expr(p + 1, hi)?;
            self.out.push(Item::Call(if self.s[p] == b'*' { Func::Mul } else { Func::Div }));
            return Ok(());
        }
        if let Some(p) = self.rscan(lo, hi, &|me, i| me.s[i] == b'^') {
            self.expr(lo, p)?;
            self.expr(p + 1, hi)?;
            self.out.push(Item::Call(Func::Pow));
            return Ok(());
        }
        self.operand(lo, hi)
    }

    /// comma separated argument list inside parentheses
    fn args(&mut self, lo: usize, hi: usize) -> Result<usize, ExprError> {
        let mut commas = vec![];
        let mut d = 0;
        for i in lo..hi {
            match self.s[i] {
                b'(' => d += 1,
                b')' => d -= 1,
                b',' if d == 0 => commas.push(i),
                _ => {}
            }
        }
        let mut a = lo;
        for &c in &commas {
            self.expr(a, c)?;
            a = c + 1;
        }
        self.expr(a, hi)?;
        Ok(commas.len())
    }

    fn operand(&mut self, lo: usize, hi: usize) -> Result<(), ExprError> {
        // AUDIO_Resample__3AB560: plain or signed number filling the whole operand
        if let Some(v) = self.number(lo, hi) {
            self.out.push(Item::Const(v));
            return Ok(());
        }
        let mut p = lo;
        while p < hi && is_space(self.s[p]) {
            p += 1;
        }
        let mut neg = false;
        if p < hi && self.s[p] == b'-' {
            neg = true;
            p += 1;
        } else if p < hi && self.s[p] == b'+' {
            p += 1;
        }
        while p < hi && is_space(self.s[p]) {
            p += 1;
        }
        if p >= hi {
            return Err(ExprError::Syntax("Something is missing.".into()));
        }
        if let Some(open) = (p..hi).find(|&i| self.s[i] == b'(') {
            let close = self.close_of(open, hi).ok_or(ExprError::Paren)?;
            if (close + 1..hi).any(|i| !is_space(self.s[i])) {
                return Err(ExprError::Syntax("Must be an operator after ) before ".into()));
            }
            if open == p {
                let n = self.args(open + 1, close)?;
                if n != 0 {
                    return Err(ExprError::Syntax("Wrong comma within these parentheses.".into()));
                }
            } else {
                let f = self.function(p, open)?;
                let want = FUNCTIONS.iter().find(|e| e.1 == f.0).map(|e| e.2).unwrap_or(0);
                let n = self.args(open + 1, close)?;
                if n != want {
                    return Err(ExprError::Args(String::from_utf8_lossy(&self.s[p..open]).into()));
                }
                self.out.push(Item::Call(f.0));
            }
        } else {
            let item = self.ident(p, hi)?;
            self.out.push(item);
        }
        if neg {
            self.out.push(Item::Call(Func::Neg));
        }
        Ok(())
    }

    fn number(&self, lo: usize, hi: usize) -> Option<f32> {
        let s = &self.s[lo..hi];
        let (v, n) = strtod(s);
        if n > 0 && s[n..].iter().all(|c| is_space(*c)) {
            // strtod already took a sign; 3AB560 also accepts "- 5"
            return Some(v as f32);
        }
        if n == 0 {
            let mut i = 0;
            while i < s.len() && is_space(s[i]) {
                i += 1;
            }
            if i >= s.len() || (s[i] != b'-' && s[i] != b'+') {
                return None;
            }
            let neg = s[i] == b'-';
            i += 1;
            while i < s.len() && is_space(s[i]) {
                i += 1;
            }
            if i < s.len() && (s[i] == b'-' || s[i] == b'+') {
                return None;
            }
            let (v, n) = strtod(&s[i..]);
            if n == 0 || !s[i + n..].iter().all(|c| is_space(*c)) {
                return None;
            }
            let v = v as f32;
            return Some(if neg { 0.0 - v } else { v });
        }
        None
    }

    /// `AUDIO_Resample__3AB200`: case-insensitive table match followed only by whitespace
    fn function(&self, p: usize, open: usize) -> Result<(Func, usize), ExprError> {
        for (name, f, n) in FUNCTIONS.iter() {
            let nb = name.as_bytes();
            let mut k = 0;
            while k < nb.len() && p + k < open && self.s[p + k].to_ascii_lowercase() == nb[k] {
                k += 1;
            }
            if k == nb.len() {
                let mut j = p + k;
                while j < open && is_space(self.s[j]) {
                    j += 1;
                }
                if j == open {
                    return Ok((*f, *n));
                }
            }
        }
        Err(ExprError::UnknownFunction(String::from_utf8_lossy(&self.s[p..open]).into()))
    }

    /// variables (`AUDIO_Resample__3AB420`, case sensitive) then constants (`3AB310`)
    fn ident(&self, p: usize, hi: usize) -> Result<Item, ExprError> {
        for (idx, v) in self.vars.iter().enumerate() {
            if self.s[p..].starts_with(v) {
                let mut j = p + v.len();
                while j < hi && is_space(self.s[j]) {
                    j += 1;
                }
                if j == hi {
                    return Ok(Item::Var(idx));
                }
            }
        }
        for (name, bits) in CONSTANTS.iter() {
            let nb = name.as_bytes();
            let mut k = 0;
            while k < nb.len() && p + k < hi && self.s[p + k].to_ascii_lowercase() == nb[k] {
                k += 1;
            }
            if k == nb.len() {
                let mut j = p + k;
                while j < hi && is_space(self.s[j]) {
                    j += 1;
                }
                if j == hi {
                    return Ok(Item::Const(f32::from_bits(*bits)));
                }
            }
        }
        Err(ExprError::UnknownConstant(String::from_utf8_lossy(&self.s[p..hi]).into()))
    }
}

/// `AUDIO_Resample__3AB7B0`: character set and parenthesis/comma pre-check
fn precheck(s: &[u8]) -> Result<(), ExprError> {
    let mut d = 0i32;
    for &c in s {
        if !c.is_ascii_alphanumeric() && c != b'_' && !is_op(c) && !is_space(c) && c != b'(' && c != b')' && c != b'.' {
            return Err(ExprError::Syntax(format!("The character is not allowed: {}", c as char)));
        }
        if d < 0 {
            return Err(ExprError::Paren);
        }
        match c {
            b'(' => d += 1,
            b')' => d -= 1,
            b',' if d == 0 => return Err(ExprError::Syntax("Wrong comma.".into())),
            _ => {}
        }
    }
    if d != 0 {
        return Err(ExprError::Paren);
    }
    Ok(())
}

/// (FastCar expr) 0x003AAD60
pub fn compile(expr: &str, vars: &str) -> Result<Image, ExprError> {
    let s = expr.as_bytes();
    precheck(s)?;
    let mut p = Parser { s, vars: split_vars(vars), out: vec![] };
    p.expr(0, s.len())?;
    Ok(Image { items: p.out })
}

impl Image {
    /// number of variables the image reads (highest index + 1)
    pub fn var_count(&self) -> usize {
        self.items.iter().filter_map(|it| if let Item::Var(i) = *it { Some(i + 1) } else { None }).max().unwrap_or(0)
    }

    /// (FastCar expr) 0x003AAFF0
    pub fn evaluate(&self, vars: &[f32]) -> Option<f32> {
        let mut s: Vec<f32> = Vec::with_capacity(16);
        for it in &self.items {
            match *it {
                Item::Const(c) => s.push(c),
                Item::Var(i) => s.push(*vars.get(i)?),
                Item::Call(f) => f.apply(&mut s),
            }
        }
        s.first().copied()
    }

    /// (FastCar expr) 0x003AB110
    pub fn optimize(&mut self) {
        self.fold_leading_constants();
        loop {
            let a = self.fold_constant_pairs();
            let b = self.fold_unary_constants();
            if !(a || b) {
                break;
            }
        }
    }

    /// (FastCar expr) 0x003ABA60
    fn fold_leading_constants(&mut self) {
        let mut stack: Vec<f32> = vec![];
        let mut k = 0;
        while k < self.items.len() {
            match self.items[k] {
                Item::Const(c) => stack.push(c),
                Item::Call(f) => f.apply(&mut stack),
                Item::Var(_) => break,
            }
            k += 1;
        }
        if k < self.items.len() {
            let mut v: Vec<Item> = stack.into_iter().map(Item::Const).collect();
            v.extend_from_slice(&self.items[k..]);
            self.items = v;
        } else {
            self.items = vec![Item::Const(stack.first().copied().unwrap_or(0.0))];
        }
    }

    /// (FastCar expr) 0x003ABF10: `c1 f1 c2 f2` with both in the same family -> `(c1 g c2) f1`
    fn fold_constant_pairs(&mut self) -> bool {
        let before = self.items.len();
        let mut i: isize = 0;
        while (i as usize) + 3 < self.items.len() {
            let u = i as usize;
            if let (Item::Const(c1), Item::Call(f1), Item::Const(c2), Item::Call(f2)) =
                (self.items[u], self.items[u + 1], self.items[u + 2], self.items[u + 3])
            {
                use Func::*;
                let folded = match (f1, f2) {
                    (Add, Add) | (Sub, Sub) => Some(c1 + c2),
                    (Add, Sub) | (Sub, Add) => Some(c1 - c2),
                    (Mul, Mul) | (Div, Div) | (Pow, Pow) => Some(c1 * c2),
                    (Mul, Div) | (Div, Mul) => Some(c1 / c2),
                    _ => None,
                };
                if let Some(v) = folded {
                    self.items[u] = Item::Const(v);
                    self.items.drain(u + 2..u + 4);
                    i -= 1;
                    if i < 0 {
                        i = -1;
                    }
                }
            }
            i += 1;
        }
        self.items.len() != before
    }

    /// (FastCar expr) 0x003ABC10: fold table functions applied to constants only
    fn fold_unary_constants(&mut self) -> bool {
        let before = self.items.len();
        let mut i: isize = 0;
        // loop bound: items i, i+1, i+2 must exist
        while (i as usize) + 2 < self.items.len() {
            let u = i as usize;
            match (self.items[u], self.items[u + 1], self.items[u + 2]) {
                (Item::Const(a), Item::Const(b), Item::Call(f)) if f.table_args() == 1 => {
                    self.items[u] = Item::Const(binary(f, a, b));
                    self.items.drain(u + 1..u + 3);
                    i -= 1;
                }
                (Item::Const(a), Item::Call(f), _) if f.table_args() == 0 => {
                    self.items[u] = Item::Const(unary(f, a));
                    self.items.remove(u + 1);
                    i -= 1;
                }
                _ => {}
            }
            i += 1;
        }
        let n = self.items.len();
        if n >= 2 {
            if let (Item::Const(a), Item::Call(f)) = (self.items[n - 2], self.items[n - 1]) {
                if f.table_args() == 0 {
                    self.items[n - 2] = Item::Const(unary(f, a));
                    self.items.pop();
                }
            }
        }
        self.items.len() != before
    }
}

/// Compile and evaluate an expression without variables (the config token evaluator's use).
pub fn eval_const(expr: &str) -> Option<f32> {
    compile(expr, "").ok()?.evaluate(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_and_quirks() {
        assert_eq!(eval_const("1+2*3"), Some(7.0));
        assert_eq!(eval_const("(1+2)*3"), Some(9.0));
        assert_eq!(eval_const("8-3-2"), Some(3.0));
        assert_eq!(eval_const("-2^2"), Some(4.0));
        assert_eq!(eval_const("1/0.0005"), Some(0.0));
        assert_eq!(eval_const("lte(0.5,0.5)*2+gt(1,0)"), Some(3.0));
        assert_eq!(eval_const("rad(180)"), Some(180.0 * f32::from_bits(RAD_FACTOR)));
        assert_eq!(eval_const("PI"), Some(f32::from_bits(0x40490FDB)));
        assert_eq!(eval_const("1e-3*2"), Some(0.002));
        assert_eq!(eval_const("2*-3"), Some(-6.0));
        assert_eq!(eval_const("min(3,max(1,2))"), Some(2.0));
        assert!(eval_const("timestep").is_none());
        assert!(eval_const("1,2").is_none());
    }

    #[test]
    fn variables_and_optimizer() {
        let mut im = compile("forwards*weight*((1.3+(0.6-1.3)*(1.0-sin(PI/2.0-abs(skid_angle))))*gt(n,0.5))", "skid_angle forwards weight n").unwrap();
        let v = [0.3f32, 1.0, 2.0, 4.0];
        let a = im.evaluate(&v).unwrap();
        im.optimize();
        let b = im.evaluate(&v).unwrap();
        assert!((a - b).abs() < 1e-5);
        let mut im = compile("x+1+2", "x").unwrap();
        im.optimize();
        assert_eq!(im.items, vec![Item::Var(0), Item::Const(3.0), Item::Call(Func::Add)]);
    }
}
