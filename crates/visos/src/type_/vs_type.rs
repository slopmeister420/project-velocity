//! VSTYPE — the engine's runtime type system (`VSTYPE::CType`, `CEnvironment`). no assert file of its own;
//! the parser sits at 0x2E4340–0x2E6250 (between PrimitiveManager.cpp and CompartmentSys.cpp) and the type
//! classes at 0x322C60–0x325400. bundles carry their structs as C-like text in a Typelib chunk, e.g.
//! `struct F3HeatHaze {VsVector FocalPoint; float Radius; bool Enabled; }`; the loader parses them into
//! types and lays the instance data out with these sizes and alignments.

use std::collections::HashMap;
use std::sync::Arc;

use crate::chunk::file_chunk::{cstr, rd32};

/// `CType::getType()` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CTypeKind {
    Float = 1,
    Integer = 2,
    String = 4,
    Enum = 5,
    Bool = 6,
    Array = 7,
    VarArray = 8,
    Struct = 9,
}

/// `CCompoundElement` — a struct member.
#[derive(Clone, Debug)]
pub struct CCompoundElement {
    pub name: String,
    pub ty: Arc<CType>,
    pub offset: u32,
}

/// a parsed type. `size` is vtable+12, `alignment` vtable+16 (field +0x10, default 4).
#[derive(Clone, Debug)]
pub struct CType {
    pub name: String,
    pub kind: CTypeKind,
    pub size: u32,
    pub alignment: u32,
    /// struct members in declaration order
    pub members: Vec<CCompoundElement>,
    /// array / vararray element type and its padded stride
    pub element: Option<(Arc<CType>, u32)>,
    /// array element count
    pub count: u32,
    /// enum (name, value) pairs
    pub enumerators: Vec<(String, i32)>,
}

impl CType {
    fn primitive(name: &str, kind: CTypeKind, size: u32) -> Self {
        CType {
            name: name.into(),
            kind,
            size,
            alignment: 4,
            members: Vec::new(),
            element: None,
            count: 0,
            enumerators: Vec::new(),
        }
    }

    /// member lookup by exact name.
    pub fn member(&self, name: &str) -> Option<&CCompoundElement> {
        self.members.iter().find(|m| m.name == name)
    }

    /// PrimitiveManager.cpp 0x002E4580 helper — bytes needed to align `v` to `a`.
    pub fn pad(v: u32, a: u32) -> u32 {
        if v % a != 0 { a - v % a } else { 0 }
    }

    /// VSTYPE 0x00323C60 — CStructType::addMember.
    fn add_member(&mut self, name: &str, ty: Arc<CType>) {
        if name.is_empty() {
            return;
        }
        self.size += Self::pad(self.size, ty.alignment);
        let offset = self.size;
        self.size += ty.size;
        self.members.push(CCompoundElement { name: name.into(), ty, offset });
    }

    /// VSTYPE 0x00322CA0 — CArrayType.
    fn array(name: String, elem: Arc<CType>, count: u32) -> Self {
        let stride = elem.size + Self::pad(elem.size, elem.alignment);
        CType {
            name,
            kind: CTypeKind::Array,
            size: stride * count,
            alignment: elem.alignment,
            members: Vec::new(),
            element: Some((elem, stride)),
            count,
            enumerators: Vec::new(),
        }
    }

    /// VSTYPE 0x00323210 — CVarArrayType (`VsVarArray<T>::SelfPtr`): {u32 count, u32 offset}.
    fn var_array(name: String, elem: Arc<CType>) -> Self {
        let stride = elem.size + Self::pad(elem.size, elem.alignment);
        CType {
            name,
            kind: CTypeKind::VarArray,
            size: 8,
            alignment: 4,
            members: Vec::new(),
            element: Some((elem, stride)),
            count: 0,
            enumerators: Vec::new(),
        }
    }
}

/// parser errors (`ERROR_*` strings at 0x664FC0).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TypeParseError {
    #[error("ERROR_PARSE at {0:?}")]
    Parse(String),
    #[error("ERROR_UNEXPECTED_CHAR {1} at {0:?}")]
    UnexpectedChar(String, &'static str),
    #[error("ERROR_UNEXPECTED_TOKEN {1} at {0:?}")]
    UnexpectedToken(String, &'static str),
    #[error("ERROR_UNKNOWN_TYPE {1} at {0:?}")]
    UnknownType(String, String),
}

/// `CEnvironment` — the type registry. lookups fall through to the parent (0x002E45B0).
#[derive(Clone, Debug, Default)]
pub struct CEnvironment {
    types: Vec<Arc<CType>>,
    index: HashMap<String, usize>,
}

impl CEnvironment {
    /// the global environment as `VSTYPE` init (0x00303340) sets it up: int, float, string, bool,
    /// VsVector / VsQuaternion / VsMatrix with 16-byte alignment.
    pub fn with_builtins() -> Self {
        let mut env = CEnvironment::default();
        env.add(Arc::new(CType::primitive("int", CTypeKind::Integer, 4)));
        env.add(Arc::new(CType::primitive("float", CTypeKind::Float, 4)));
        env.add(Arc::new(CType::primitive("string", CTypeKind::String, 4)));
        env.add(Arc::new(CType::primitive("bool", CTypeKind::Bool, 1)));
        for d in [
            "struct VsVector { float x; float y; float z; float w; };",
            "struct VsQuaternion { float x; float y; float z; float w; };",
            "struct VsMatrix { VsVector row0; VsVector row1; VsVector row2; VsVector row3; };",
        ] {
            let mut t = env.parse(d, false).expect("builtin type").0;
            Arc::make_mut(&mut t).alignment = 16;
            env.add(t);
        }
        env
    }

    /// PrimitiveManager.cpp 0x002E4530 — CEnvironment::add (later adds shadow earlier ones).
    pub fn add(&mut self, t: Arc<CType>) {
        self.index.insert(t.name.clone(), self.types.len());
        self.types.push(t);
    }

    /// PrimitiveManager.cpp 0x002E45B0 — CType::find (strcmp).
    pub fn find(&self, name: &str) -> Option<Arc<CType>> {
        self.index.get(name).map(|&i| self.types[i].clone())
    }

    /// PrimitiveManager.cpp 0x002E5330 — parse one declaration; returns the type and the rest of the text.
    /// `find_existing` reuses an already registered struct/enum of the same name (the bundle loader passes 0).
    pub fn parse<'s>(&mut self, text: &'s str, find_existing: bool) -> Result<(Arc<CType>, &'s str), TypeParseError> {
        let mut p = Parser { s: text.as_bytes(), i: 0 };
        p.skip_ws();
        let tok = p.ident();
        let t = match tok.as_str() {
            "enum" => self.parse_enum(&mut p, find_existing)?,
            "struct" => self.parse_struct(&mut p, find_existing)?,
            "VsVarArray" => self.parse_var_array_decl(&mut p)?,
            _ => {
                let base = self.find(&tok).ok_or_else(|| TypeParseError::UnknownType(p.rest(), tok.clone()))?;
                let mut t = base.clone();
                if p.peek() != Some(b';') {
                    t = self.parse_array_typedef(&mut p, base.clone())?;
                }
                if Arc::ptr_eq(&t, &base) && !matches!(p.peek(), Some(b';') | None) {
                    let alias = p.ident();
                    if self.find(&alias).is_none() {
                        let mut c = (*base).clone();
                        c.name = alias;
                        t = Arc::new(c);
                    }
                }
                t
            }
        };
        if p.peek() == Some(b';') {
            p.i += 1;
        }
        Ok((t, std::str::from_utf8(&p.s[p.i..]).unwrap_or("")))
    }

    /// PrimitiveManager.cpp 0x002E4A80
    fn parse_struct(&mut self, p: &mut Parser, find_existing: bool) -> Result<Arc<CType>, TypeParseError> {
        let name = p.ident();
        if find_existing {
            if let Some(t) = self.find(&name) {
                if t.kind == CTypeKind::Struct {
                    return Ok(t);
                }
                return Err(TypeParseError::UnexpectedToken(p.rest(), "type already exists"));
            }
        }
        let mut st = CType::primitive(&name, CTypeKind::Struct, 0);
        if p.peek() != Some(b'{') {
            return Err(TypeParseError::UnexpectedChar(p.rest(), "expected {"));
        }
        p.i += 1;
        loop {
            p.skip_ws();
            if p.peek() == Some(b'}') || p.peek().is_none() {
                break;
            }
            let start = p.i;
            let tname = p.ident();
            if p.i == start {
                return Err(TypeParseError::Parse(p.rest()));
            }
            let mut ty = if tname != "VsVarArray" { self.find(&tname) } else { Some(self.parse_var_array(p)?) }
                .ok_or_else(|| TypeParseError::UnknownType(p.rest(), tname.clone()))?;
            let member = p.ident();
            if p.peek() == Some(b'[') {
                ty = self.parse_array_suffix(p, ty)?;
            }
            if p.peek() == Some(b';') {
                p.i += 1;
            } else {
                return Err(TypeParseError::UnexpectedChar(p.rest(), "exepected ;"));
            }
            st.add_member(&member, ty);
        }
        if p.peek() == Some(b'}') {
            p.i += 1;
        }
        p.skip_ws();
        Ok(Arc::new(st))
    }

    /// PrimitiveManager.cpp 0x002E4DC0
    fn parse_enum(&mut self, p: &mut Parser, find_existing: bool) -> Result<Arc<CType>, TypeParseError> {
        let name = p.ident();
        if find_existing {
            if let Some(t) = self.find(&name) {
                if t.kind == CTypeKind::Enum {
                    return Ok(t);
                }
                return Err(TypeParseError::UnexpectedToken(p.rest(), "type already exists"));
            }
        }
        let mut en = CType::primitive(&name, CTypeKind::Enum, 4);
        if p.peek() != Some(b'{') {
            return Err(TypeParseError::UnexpectedChar(p.rest(), "expected {"));
        }
        p.i += 1;
        let mut value: i32 = -1;
        loop {
            p.skip_ws();
            if p.peek() == Some(b'}') || p.peek().is_none() {
                break;
            }
            let start = p.i;
            let e = p.ident();
            if p.i == start {
                return Err(TypeParseError::Parse(p.rest()));
            }
            if p.peek() == Some(b'=') {
                p.i += 1;
                value = atoi(&p.ident());
            } else {
                value = value.wrapping_add(1);
            }
            p.skip_ws();
            if p.peek() == Some(b',') {
                p.i += 1;
            }
            en.enumerators.push((e, value));
        }
        if p.peek() == Some(b'}') {
            p.i += 1;
        }
        Ok(Arc::new(en))
    }

    /// PrimitiveManager.cpp 0x002E46E0 — `[N][M]...`, registering `elem-N` array types.
    fn parse_array_suffix(&mut self, p: &mut Parser, mut ty: Arc<CType>) -> Result<Arc<CType>, TypeParseError> {
        while p.peek() == Some(b'[') {
            p.i += 1;
            let start = p.i;
            let n = p.ident();
            if p.i == start {
                return Err(TypeParseError::Parse(p.rest()));
            }
            if p.peek() != Some(b']') {
                return Err(TypeParseError::UnexpectedChar(p.rest(), "expected ]"));
            }
            p.i += 1;
            let count = atoi(&n) as u32;
            let name = format!("{}-{}", ty.name, count);
            ty = match self.find(&name) {
                Some(t) => t,
                None => {
                    let t = Arc::new(CType::array(name, ty, count));
                    self.add(t.clone());
                    t
                }
            };
            p.skip_ws();
        }
        Ok(ty)
    }

    /// PrimitiveManager.cpp 0x002E5130 — `base name[N]` at top level.
    fn parse_array_typedef(&mut self, p: &mut Parser, base: Arc<CType>) -> Result<Arc<CType>, TypeParseError> {
        let save = p.i;
        let name = p.ident();
        if p.peek() == Some(b'[') {
            let arr = self.parse_array_suffix(p, base.clone())?;
            if self.find(&name).is_none() {
                let mut c = (*arr).clone();
                c.name = name;
                return Ok(Arc::new(c));
            }
            return Ok(arr);
        }
        p.i = save;
        Ok(base)
    }

    /// PrimitiveManager.cpp 0x002E4870 — `<T>::SelfPtr`.
    fn parse_var_array(&mut self, p: &mut Parser) -> Result<Arc<CType>, TypeParseError> {
        p.skip_ws();
        if p.peek() != Some(b'<') {
            return Err(TypeParseError::UnexpectedChar(p.rest(), "expected <"));
        }
        p.i += 1;
        let elem_name = p.ident();
        if !p.rest().starts_with(">::") {
            return Err(TypeParseError::UnexpectedToken(p.rest(), "expected >::"));
        }
        p.i += 3;
        let tag = p.ident();
        if !tag.eq_ignore_ascii_case("selfptr") {
            return Err(TypeParseError::UnexpectedToken(p.rest(), "expected SelfPtr"));
        }
        let elem = self.find(&elem_name).ok_or_else(|| TypeParseError::UnknownType(p.rest(), elem_name.clone()))?;
        let name = format!("_vararray_{}", elem_name);
        Ok(match self.find(&name) {
            Some(t) => t,
            None => Arc::new(CType::var_array(name, elem)),
        })
    }

    /// PrimitiveManager.cpp 0x002E5270 — top-level `VsVarArray<T>::SelfPtr alias;`.
    fn parse_var_array_decl(&mut self, p: &mut Parser) -> Result<Arc<CType>, TypeParseError> {
        let t = self.parse_var_array(p)?;
        let alias = p.ident();
        let mut c = (*t).clone();
        c.name = alias;
        Ok(Arc::new(c))
    }
}

struct Parser<'s> {
    s: &'s [u8],
    i: usize,
}

impl Parser<'_> {
    /// PrimitiveManager.cpp 0x002E57E0
    fn skip_ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C) {
            self.i += 1;
        }
    }
    /// PrimitiveManager.cpp 0x002E5810 — alnum/underscore run (127 chars max), whitespace skipped both sides.
    fn ident(&mut self) -> String {
        self.skip_ws();
        let start = self.i;
        while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_') {
            self.i += 1;
        }
        let end = self.i.min(start + 127);
        let r = String::from_utf8_lossy(&self.s[start..end]).into_owned();
        self.skip_ws();
        r
    }
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied().filter(|&c| c != 0)
    }
    fn rest(&self) -> String {
        String::from_utf8_lossy(&self.s[self.i..self.s.len().min(self.i + 40)]).into_owned()
    }
}

/// crt atoi: leading digits with optional sign.
fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (neg, d) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let mut v: i32 = 0;
    for c in d.bytes().take_while(|c| c.is_ascii_digit()) {
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i32);
    }
    if neg { v.wrapping_neg() } else { v }
}

/// a decoded value of some `CType`.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Float(f32),
    Int(i32),
    String(Option<String>),
    Enum(i32),
    Bool(bool),
    Array(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

impl Value {
    pub fn as_f32(&self) -> Option<f32> {
        match *self {
            Value::Float(f) => Some(f),
            Value::Int(i) | Value::Enum(i) => Some(i as f32),
            _ => None,
        }
    }
    pub fn as_i32(&self) -> Option<i32> {
        match *self {
            Value::Int(i) | Value::Enum(i) => Some(i),
            Value::Bool(b) => Some(b as i32),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            Value::Int(i) => Some(i != 0),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => s.as_deref(),
            _ => None,
        }
    }
    /// struct field by name.
    pub fn field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct(f) => f.iter().find(|(n, _)| n == name).map(|(_, v)| v),
            _ => None,
        }
    }
    /// VsVector fields as an array.
    pub fn as_vec4(&self) -> Option<[f32; 4]> {
        Some([
            self.field("x")?.as_f32()?,
            self.field("y")?.as_f32()?,
            self.field("z")?.as_f32()?,
            self.field("w")?.as_f32()?,
        ])
    }
}

/// decode `ty` at `at` in `base`; strings and vararrays hold offsets relative to `base` (the fix-up base,
/// file_type.cpp 0x002ED740).
pub fn decode(ty: &CType, base: &[u8], at: usize) -> Value {
    match ty.kind {
        CTypeKind::Float => Value::Float(f32::from_bits(rd32(base, at))),
        CTypeKind::Integer => Value::Int(rd32(base, at) as i32),
        CTypeKind::Enum => Value::Enum(rd32(base, at) as i32),
        CTypeKind::Bool => Value::Bool(base.get(at).copied().unwrap_or(0) != 0),
        CTypeKind::String => Value::String(cstr(base, rd32(base, at) as usize).map(str::to_owned)),
        CTypeKind::Struct => {
            Value::Struct(ty.members.iter().map(|m| (m.name.clone(), decode(&m.ty, base, at + m.offset as usize))).collect())
        }
        CTypeKind::Array => {
            let (e, stride) = ty.element.as_ref().unwrap();
            Value::Array((0..ty.count as usize).map(|i| decode(e, base, at + i * *stride as usize)).collect())
        }
        CTypeKind::VarArray => {
            let (e, stride) = ty.element.as_ref().unwrap();
            let n = rd32(base, at) as usize;
            let off = rd32(base, at + 4) as usize;
            Value::Array((0..n).map(|i| decode(e, base, off + i * *stride as usize)).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_haze_layout() {
        let mut env = CEnvironment::with_builtins();
        let (t, _) = env.parse("struct F3HeatHaze {VsVector FocalPoint; float Radius; bool Enabled; }", false).unwrap();
        assert_eq!(t.member("Radius").unwrap().offset, 16);
        assert_eq!(t.member("Enabled").unwrap().offset, 20); // heathaze loader copies +20 to +21
        assert_eq!(t.size, 21);
        let (c, _) = env
            .parse("struct VSCamera {float HorizontalFOV; float NearClip; float FarClip; float FocusDistance; float FocusDistanceNear; float FocusDistanceFar; bool DOF; bool MotionBlur; bool FocusOnLookAt; string LookAt; }", false)
            .unwrap();
        assert_eq!(c.member("MotionBlur").unwrap().offset, 28); // bools align to 4
        assert_eq!(c.member("LookAt").unwrap().offset, 36);
        let (a, _) = env.parse("struct A { float v[3]; int n; };", false).unwrap();
        assert_eq!(a.member("n").unwrap().offset, 12);
        assert!(env.parse("struct B { nosuch x; };", false).is_err());
        assert!(env.find("float-3").is_some());
        let (e, _) = env.parse("enum E { A, B = 5, C };", false).unwrap();
        assert_eq!(e.enumerators, vec![("A".into(), 0), ("B".into(), 5), ("C".into(), 6)]);
    }
}
