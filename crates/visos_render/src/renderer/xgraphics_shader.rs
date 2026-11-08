//! XDK `XGAssembleShader` replacement (called from ShaderProgramManager.cpp 0x002DEBB0): parses the
//! `vs.1.x` / `xvs.1.x` / `ps.1.1` / `xps.1.1` assembly in `Shaders\*.vxu|*.pxu` and translates it into WGSL
//! functions with the NV2A's semantics (vertex program registers and constants c-96..c95, register combiner
//! ranges and co-issue for pixel programs).
//!
//! Generated vertex code: `fn xvs_program(v: array<vec4<f32>, 16>) -> XvsOut` reading constants through
//! `xvs_c(i)` (index = register + 96, supplied by the wrapper). Generated pixel code:
//! `fn xps_program(i: XpsIn) -> vec4<f32>` reading `xps_c(i)` and sampling through `xps_texN(coord)`.

use std::collections::HashMap;
use std::fmt::Write as _;

#[derive(Debug, thiserror::Error)]
pub enum AsmError {
    #[error("no version token")]
    NoVersion,
    #[error("unknown opcode `{0}`")]
    Opcode(String),
    #[error("bad operand `{0}`")]
    Operand(String),
    #[error("missing operand for `{0}`")]
    Missing(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Vertex,
    Pixel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Input,
    Temp,
    Const,
    Addr,
    OutPos,
    OutD,
    OutT,
    OutFog,
    OutPts,
    Tex,
    Discard,
    /// literal of a `def`
    Literal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Operand {
    pub file: File,
    pub index: i32,
    /// `c[n + a0.x]`
    pub relative: bool,
    /// source swizzle (components 0..3), dest write mask in bits 0..3
    pub swz: [u8; 4],
    pub explicit: bool,
    pub mask: u8,
    /// as written (`rgb`, `a`, `b`, `xyz`...) for the pixel portion rules
    pub swz_text: String,
    pub neg: bool,
    pub bias: bool,
    pub bx2: bool,
    pub invert: bool,
    pub literal: f32,
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub op: String,
    /// `_x2`, `_x4`, `_d2`, `_bx2`, `_sat`, `_bias`
    pub mods: Vec<String>,
    pub coissue: bool,
    pub ops: Vec<Operand>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub unit: Unit,
    pub version: String,
    pub instrs: Vec<Instr>,
    /// `def cN, x, y, z, w`
    pub defs: Vec<(i32, [f32; 4])>,
}

/// operands including the destination(s)
fn operand_count(unit: Unit, op: &str) -> Option<usize> {
    Some(match (unit, op) {
        (_, "def") => 5,
        (Unit::Pixel, "tex" | "texcoord" | "texkill") => 1,
        (_, "mov" | "rsq" | "rcp" | "lit" | "expp" | "logp" | "frc" | "exp" | "log") => 2,
        (_, "add" | "sub" | "mul" | "dp3" | "dp4" | "min" | "max" | "slt" | "sge" | "dst") => 3,
        (Unit::Vertex, "m4x4" | "m3x3" | "m3x4" | "m4x3" | "m3x2") => 3,
        (_, "mad" | "lrp" | "cnd") => 4,
        (Unit::Pixel, "xmma" | "xmmc") => 7,
        (Unit::Pixel, "xdm" | "xdd") => 6,
        (Unit::Pixel, "xfc") => 7,
        (Unit::Vertex, "arl") => 2,
        _ => return None,
    })
}

/// comments and `#define`s removed; returns the remaining text with defines substituted.
fn preprocess(src: &str) -> String {
    let text = src.replace("\r\n", "\n").replace('\r', "\n");
    let mut defs: Vec<(String, String)> = Vec::new();
    let mut out = String::new();
    for line in text.lines() {
        let mut l = line.to_string();
        for cut in ["//", ";"] {
            if let Some(i) = l.find(cut) {
                l.truncate(i);
            }
        }
        let t = l.trim();
        if let Some(rest) = t.strip_prefix("#define") {
            let mut it = rest.split_whitespace();
            if let (Some(n), Some(v)) = (it.next(), it.next()) {
                defs.push((n.to_string(), v.to_string()));
            }
            continue;
        }
        if t.starts_with('#') {
            continue;
        }
        // whole-word substitution
        let mut s = String::new();
        let b = l.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i].is_ascii_alphabetic() || b[i] == b'_' {
                let st = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let w = &l[st..i];
                match defs.iter().find(|(n, _)| n == w) {
                    Some((_, v)) => s.push_str(v),
                    None => s.push_str(w),
                }
            } else {
                s.push(b[i] as char);
                i += 1;
            }
        }
        out.push_str(&s);
        out.push('\n');
    }
    out
}

/// tokens: words (brackets kept whole, spaces inside dropped), "," and a leading "+"
fn tokenize(text: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '[' => {
                depth += 1;
                cur.push(ch);
            }
            ']' => {
                depth -= 1;
                cur.push(ch);
            }
            _ if depth > 0 => {
                if !ch.is_whitespace() {
                    cur.push(ch);
                }
            }
            ',' => {
                if !cur.is_empty() {
                    toks.push(std::mem::take(&mut cur));
                }
                toks.push(",".into());
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    toks.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        toks.push(cur);
    }
    toks
}

fn swz_index(c: char) -> Option<u8> {
    match c {
        'x' | 'r' => Some(0),
        'y' | 'g' => Some(1),
        'z' | 'b' => Some(2),
        'w' | 'a' => Some(3),
        _ => None,
    }
}

fn parse_operand(unit: Unit, raw: &str, dest: bool) -> Result<Operand, AsmError> {
    let bad = || AsmError::Operand(raw.to_string());
    let mut s = raw.to_ascii_lowercase();
    let mut o = Operand {
        file: File::Temp,
        index: 0,
        relative: false,
        swz: [0, 1, 2, 3],
        explicit: false,
        mask: 0xF,
        swz_text: String::new(),
        neg: false,
        bias: false,
        bx2: false,
        invert: false,
        literal: 0.0,
    };
    if let Some(r) = s.strip_prefix("1-") {
        o.invert = true;
        s = r.to_string();
    }
    if let Some(r) = s.strip_prefix('-') {
        o.neg = true;
        s = r.to_string();
    }
    // swizzle after the last '.' outside brackets
    let close = s.rfind(']').map(|i| i + 1).unwrap_or(0);
    let (mut base, swz) = match s[close..].rfind('.') {
        Some(i) if s[close + i + 1..].chars().all(|c| swz_index(c).is_some()) && !s[close + i + 1..].is_empty() => {
            (s[..close + i].to_string(), s[close + i + 1..].to_string())
        }
        _ => (s.clone(), String::new()),
    };
    for (suf, flag) in [("_bias", 0), ("_bx2", 1)] {
        if let Some(b) = base.strip_suffix(suf) {
            base = b.to_string();
            if flag == 0 {
                o.bias = true;
            } else {
                o.bx2 = true;
            }
        }
    }
    let num = |t: &str| t.parse::<i32>().map_err(|_| bad());
    if let Some(inner) = base.strip_prefix("c[").and_then(|r| r.strip_suffix(']')) {
        o.file = File::Const;
        let mut idx = 0i32;
        let mut sign = 1;
        let mut term = String::new();
        let mut flush = |term: &mut String, sign: i32, o: &mut Operand| -> Result<(), AsmError> {
            if term.is_empty() {
                return Ok(());
            }
            if term == "a0.x" || term == "a0" {
                o.relative = true;
            } else {
                idx += sign * term.parse::<i32>().map_err(|_| AsmError::Operand(raw.to_string()))?;
            }
            term.clear();
            Ok(())
        };
        for ch in inner.chars() {
            match ch {
                '+' => {
                    flush(&mut term, sign, &mut o)?;
                    sign = 1;
                }
                '-' => {
                    flush(&mut term, sign, &mut o)?;
                    sign = -1;
                }
                _ => term.push(ch),
            }
        }
        flush(&mut term, sign, &mut o)?;
        o.index = idx;
    } else if base == "opos" {
        o.file = File::OutPos;
    } else if base == "ofog" {
        o.file = File::OutFog;
    } else if base == "opts" {
        o.file = File::OutPts;
    } else if base == "a0" {
        o.file = File::Addr;
    } else if base == "discard" {
        o.file = File::Discard;
    } else if let Some(n) = base.strip_prefix("od") {
        o.file = File::OutD;
        o.index = num(n)?;
    } else if let Some(n) = base.strip_prefix("ot") {
        o.file = File::OutT;
        o.index = num(n)?;
    } else if let Some(n) = base.strip_prefix('v') {
        o.file = File::Input;
        o.index = num(n)?;
    } else if let Some(n) = base.strip_prefix('r') {
        o.file = File::Temp;
        o.index = num(n)?;
    } else if let Some(n) = base.strip_prefix('c') {
        o.file = File::Const;
        o.index = num(n)?;
    } else if let (Unit::Pixel, Some(n)) = (unit, base.strip_prefix('t')) {
        o.file = File::Tex;
        o.index = num(n)?;
    } else if let Ok(f) = base.parse::<f32>() {
        o.file = File::Literal;
        o.literal = if o.neg { -f } else { f };
        o.neg = false;
    } else {
        return Err(bad());
    }
    if !swz.is_empty() {
        o.swz_text = swz.clone();
        let c: Vec<u8> = swz.chars().filter_map(swz_index).collect();
        if dest {
            o.mask = c.iter().fold(0, |m, &i| m | (1 << i));
        } else {
            o.explicit = true;
            for k in 0..4 {
                o.swz[k] = c[k.min(c.len() - 1)];
            }
        }
    }
    Ok(o)
}

/// parse a whole program file.
pub fn parse(src: &str) -> Result<Program, AsmError> {
    let toks = tokenize(&preprocess(src));
    let mut it = toks.into_iter().peekable();
    let version = it.next().ok_or(AsmError::NoVersion)?.to_ascii_lowercase();
    let unit = if version.starts_with("vs") || version.starts_with("xvs") {
        Unit::Vertex
    } else if version.starts_with("ps") || version.starts_with("xps") {
        Unit::Pixel
    } else {
        return Err(AsmError::NoVersion);
    };
    let mut prog = Program { unit, version, instrs: Vec::new(), defs: Vec::new() };
    let mut coissue = false;
    while let Some(t) = it.next() {
        if t == "+" {
            coissue = true;
            continue;
        }
        let t = t.trim_start_matches('+').to_ascii_lowercase();
        let mut parts = t.split('_');
        let op = parts.next().unwrap_or_default().to_string();
        let mods: Vec<String> = parts.map(str::to_string).collect();
        let n = operand_count(unit, &op).ok_or_else(|| AsmError::Opcode(t.clone()))?;
        let mut raw = Vec::new();
        for k in 0..n {
            if k > 0 {
                match it.next() {
                    Some(c) if c == "," => {}
                    _ => return Err(AsmError::Missing(op.clone())),
                }
            }
            raw.push(it.next().ok_or_else(|| AsmError::Missing(op.clone()))?);
        }
        if op == "def" {
            let r = parse_operand(unit, &raw[0], true)?;
            let mut v = [0.0; 4];
            for k in 0..4 {
                v[k] = raw[k + 1].parse::<f32>().map_err(|_| AsmError::Operand(raw[k + 1].clone()))?;
            }
            prog.defs.push((r.index, v));
            continue;
        }
        let dests = match op.as_str() {
            "xmma" | "xmmc" => 3,
            "xdm" | "xdd" => 2,
            "xfc" => 0,
            _ => 1,
        };
        let mut ops = Vec::new();
        for (k, r) in raw.iter().enumerate() {
            ops.push(parse_operand(unit, r, k < dests)?);
        }
        prog.instrs.push(Instr { op, mods, coissue, ops });
        coissue = false;
    }
    Ok(prog)
}

/// per-object values the vertex wrapper computes from bevy (game globals resolved on the gpu).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DynInput {
    /// matWorldViewProj, 4 registers (rows of the transposed d3d matrix)
    WorldViewProj,
    /// matWorld
    World,
    /// matWorldView
    WorldView,
    /// matViewProj
    ViewProj,
    /// matView
    View,
    /// cameraDir (camera +0x78)
    CameraDir,
}

impl DynInput {
    pub fn registers(self) -> i32 {
        if self == DynInput::CameraDir { 1 } else { 4 }
    }

    /// wgsl expression of register `k` (the wrapper defines `dyn_*` matrices in game convention)
    fn expr(self, k: i32) -> String {
        let m = match self {
            DynInput::WorldViewProj => "dyn_wvp",
            DynInput::World => "dyn_world",
            DynInput::WorldView => "dyn_wv",
            DynInput::ViewProj => "dyn_vp",
            DynInput::View => "dyn_view",
            DynInput::CameraDir => return "dyn_camera_dir".into(),
        };
        // register k = row k of the matrix as a column-vector transform
        format!("vec4<f32>({m}[0][{k}], {m}[1][{k}], {m}[2][{k}], {m}[3][{k}])")
    }
}

const SW: [char; 4] = ['x', 'y', 'z', 'w'];

fn swz_str(o: &Operand) -> String {
    if o.swz == [0, 1, 2, 3] {
        String::new()
    } else {
        format!(".{}", o.swz.iter().map(|&i| SW[i as usize]).collect::<String>())
    }
}

fn wgsl_f32(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") { s } else { format!("{s}.0") }
}

/// vertex program translation. `dynamic` = (register, input) pairs resolved in the wrapper.
pub fn vertex_wgsl(p: &Program, dynamic: &[(i32, DynInput)]) -> String {
    let mut w = String::new();
    let mut defs: HashMap<i32, [f32; 4]> = HashMap::new();
    for (r, v) in &p.defs {
        defs.insert(*r, *v);
    }
    let const_expr = |idx: i32| -> String {
        for &(r, d) in dynamic {
            if idx >= r && idx < r + d.registers() {
                return d.expr(idx - r);
            }
        }
        if let Some(v) = defs.get(&idx) {
            return format!("vec4<f32>({}, {}, {}, {})", wgsl_f32(v[0]), wgsl_f32(v[1]), wgsl_f32(v[2]), wgsl_f32(v[3]));
        }
        format!("xvs_c({})", idx + 96)
    };
    let src = |o: &Operand| -> String {
        let base = match o.file {
            File::Input => format!("v[{}]", o.index),
            File::Temp => format!("r{}", o.index),
            File::Const if o.relative => format!("xvs_c_rel({} + a0)", o.index + 96),
            File::Const => const_expr(o.index),
            File::Literal => format!("vec4<f32>({})", wgsl_f32(o.literal)),
            _ => "vec4<f32>(0.0)".into(),
        };
        let e = format!("{base}{}", swz_str(o));
        if o.neg { format!("(-{e})") } else { e }
    };
    let dst_name = |o: &Operand| -> Option<String> {
        Some(match o.file {
            File::Temp => format!("r{}", o.index),
            File::OutPos => "o.pos".into(),
            File::OutD => format!("o.d{}", o.index),
            File::OutT => format!("o.t{}", o.index),
            File::OutFog => "o_fog".into(),
            File::OutPts => "o_pts".into(),
            _ => return None,
        })
    };
    let _ = writeln!(w, "fn xvs_program(v: array<vec4<f32>, 16>) -> XvsOut {{");
    for i in 0..12 {
        let _ = writeln!(w, "    var r{i} = vec4<f32>(0.0);");
    }
    let _ = writeln!(w, "    var a0: i32 = 0;\n    var o_fog = vec4<f32>(0.0);\n    var o_pts = vec4<f32>(0.0);");
    let _ = writeln!(w, "    var o: XvsOut;\n    o.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);");
    for k in 0..2 {
        let _ = writeln!(w, "    o.d{k} = vec4<f32>(0.0, 0.0, 0.0, 1.0);");
    }
    for k in 0..4 {
        let _ = writeln!(w, "    o.t{k} = vec4<f32>(0.0, 0.0, 0.0, 1.0);");
    }
    for ins in &p.instrs {
        let d = &ins.ops[0];
        let s: Vec<String> = ins.ops[1..].iter().map(src).collect();
        let scalar = |k: usize| -> String {
            let o = &ins.ops[k + 1];
            // scalar ops take the replicated component, .w when no swizzle is given
            let comp = if o.explicit { SW[o.swz[0] as usize] } else { 'w' };
            let base = src(&Operand { swz: [0, 1, 2, 3], neg: false, ..o.clone() });
            let e = format!("{base}.{comp}");
            if o.neg { format!("(-{e})") } else { e }
        };
        let val = match ins.op.as_str() {
            "mov" => s[0].clone(),
            "add" => format!("({} + {})", s[0], s[1]),
            "sub" => format!("({} - {})", s[0], s[1]),
            "mul" => format!("({} * {})", s[0], s[1]),
            "mad" => format!("({} * {} + {})", s[0], s[1], s[2]),
            "min" => format!("min({}, {})", s[0], s[1]),
            "max" => format!("max({}, {})", s[0], s[1]),
            "slt" => format!("select(vec4<f32>(0.0), vec4<f32>(1.0), {} < {})", s[0], s[1]),
            "sge" => format!("select(vec4<f32>(0.0), vec4<f32>(1.0), {} >= {})", s[0], s[1]),
            "dp3" => format!("vec4<f32>(dot(({}).xyz, ({}).xyz))", s[0], s[1]),
            "dp4" => format!("vec4<f32>(dot({}, {}))", s[0], s[1]),
            "rsq" => format!("vec4<f32>(xvs_rsq({}))", scalar(0)),
            "rcp" => format!("vec4<f32>(xvs_rcp({}))", scalar(0)),
            "lit" => format!("xvs_lit({})", s[0]),
            "frc" => format!("fract({})", s[0]),
            "expp" => format!("vec4<f32>(exp2({}))", scalar(0)),
            "logp" => format!("vec4<f32>(log2(abs({})))", scalar(0)),
            "dst" => format!("vec4<f32>(1.0, ({0}).y * ({1}).y, ({0}).z, ({1}).w)", s[0], s[1]),
            "m4x4" | "m4x3" | "m3x4" | "m3x3" | "m3x2" => {
                let o = &ins.ops[2];
                let (n, three) = match ins.op.as_str() {
                    "m4x4" => (4, false),
                    "m4x3" => (3, false),
                    "m3x4" => (4, true),
                    "m3x3" => (3, true),
                    _ => (2, true),
                };
                let mut comps = Vec::new();
                for k in 0..4 {
                    if k < n {
                        let row = Operand { index: o.index + k as i32, ..o.clone() };
                        let r = src(&row);
                        comps.push(if three {
                            format!("dot(({}).xyz, ({r}).xyz)", s[0])
                        } else {
                            format!("dot({}, {r})", s[0])
                        });
                    } else {
                        comps.push("0.0".into());
                    }
                }
                format!("vec4<f32>({})", comps.join(", "))
            }
            "arl" => s[0].clone(),
            _ => "vec4<f32>(0.0)".into(),
        };
        let mask = match ins.op.as_str() {
            "m3x3" | "m4x3" => d.mask & 0x7,
            "m3x2" => d.mask & 0x3,
            _ => d.mask,
        };
        let _ = writeln!(w, "    {{ let t = {val};");
        if d.file == File::Addr {
            let _ = writeln!(w, "      a0 = i32(floor(t.x)); }}");
            continue;
        }
        if let Some(n) = dst_name(d) {
            for k in 0..4 {
                if mask & (1 << k) != 0 {
                    let _ = writeln!(w, "      {n}.{c} = t.{c};", c = SW[k]);
                }
            }
        }
        let _ = writeln!(w, "    }}");
    }
    let _ = writeln!(w, "    o.d0 = clamp(o.d0, vec4<f32>(0.0), vec4<f32>(1.0));");
    let _ = writeln!(w, "    o.d1 = clamp(o.d1, vec4<f32>(0.0), vec4<f32>(1.0));");
    let _ = writeln!(w, "    return o;\n}}");
    w
}

/// pixel program translation (register combiner semantics: values held in [-1, 1], rgb and alpha portions,
/// co-issued pairs read the registers before either writes).
pub fn pixel_wgsl(p: &Program) -> String {
    let mut w = String::new();
    let mut defs: HashMap<i32, [f32; 4]> = HashMap::new();
    for (r, v) in &p.defs {
        defs.insert(*r, *v);
    }
    let reg = |o: &Operand| -> String {
        match o.file {
            File::Temp => format!("r{}", o.index),
            File::Tex => format!("t{}", o.index),
            File::Input => format!("v{}", o.index),
            File::Const => match defs.get(&o.index) {
                Some(v) => format!("vec4<f32>({}, {}, {}, {})", wgsl_f32(v[0]), wgsl_f32(v[1]), wgsl_f32(v[2]), wgsl_f32(v[3])),
                None => format!("xps_c({})", o.index),
            },
            File::Literal => format!("vec4<f32>({})", wgsl_f32(o.literal)),
            _ => "vec4<f32>(0.0)".into(),
        }
    };
    let modify = |o: &Operand, e: String, vec: bool| -> String {
        let mut e = e;
        if o.bias {
            e = format!("({e} - {})", if vec { "vec3<f32>(0.5)" } else { "0.5" });
        }
        if o.bx2 {
            e = format!("({e} * 2.0 - {})", if vec { "vec3<f32>(1.0)" } else { "1.0" });
        }
        if o.invert {
            e = format!("({} - {e})", if vec { "vec3<f32>(1.0)" } else { "1.0" });
        }
        if o.neg {
            e = format!("(-{e})");
        }
        e
    };
    // rgb portion: .a / .b replicate, anything else the colour; alpha portion: blue for .rgb/.b, else alpha
    let rgb = |o: &Operand| -> String {
        let r = reg(o);
        let e = match o.swz_text.as_str() {
            "a" | "w" => format!("vec3<f32>(({r}).a)"),
            "b" | "z" => format!("vec3<f32>(({r}).b)"),
            "" | "rgb" | "xyz" | "rgba" | "xyzw" => format!("({r}).rgb"),
            _ => format!("({r}){}.xyz", swz_str(o)),
        };
        modify(o, e, true)
    };
    let alpha = |o: &Operand| -> String {
        let r = reg(o);
        let e = match o.swz_text.as_str() {
            "rgb" | "xyz" | "b" | "z" => format!("({r}).b"),
            "r" | "x" => format!("({r}).r"),
            "g" | "y" => format!("({r}).g"),
            _ => format!("({r}).a"),
        };
        modify(o, e, false)
    };
    let _ = writeln!(w, "fn xps_program(i: XpsIn) -> vec4<f32> {{");
    let _ = writeln!(w, "    let v0 = clamp(i.d0, vec4<f32>(0.0), vec4<f32>(1.0));");
    let _ = writeln!(w, "    let v1 = clamp(i.d1, vec4<f32>(0.0), vec4<f32>(1.0));");
    for k in 0..4 {
        let _ = writeln!(w, "    var t{k} = vec4<f32>(0.0);");
    }
    let _ = writeln!(w, "    var r0 = vec4<f32>(0.0);\n    var r1 = vec4<f32>(0.0);");
    // texture address instructions first
    let mut t0_used = false;
    for ins in &p.instrs {
        let d = &ins.ops[0];
        match ins.op.as_str() {
            "tex" => {
                let _ = writeln!(w, "    t{0} = xps_tex{0}(i.t{0});", d.index);
                t0_used |= d.index == 0;
            }
            "texcoord" => {
                let _ = writeln!(w, "    t{0} = clamp(i.t{0}, vec4<f32>(0.0), vec4<f32>(1.0));", d.index);
                t0_used |= d.index == 0;
            }
            _ => {}
        }
    }
    if t0_used {
        // the combiners start with r0.a = t0.a
        let _ = writeln!(w, "    r0.a = t0.a;");
    }
    let mut k = 0usize;
    let ins_list: Vec<&Instr> = p.instrs.iter().filter(|i| !matches!(i.op.as_str(), "tex" | "texcoord" | "texkill")).collect();
    while k < ins_list.len() {
        // a co-issued pair executes as one combiner stage
        let group: Vec<&Instr> =
            if k + 1 < ins_list.len() && ins_list[k + 1].coissue { vec![ins_list[k], ins_list[k + 1]] } else { vec![ins_list[k]] };
        let mut commits = Vec::new();
        for (gi, ins) in group.iter().enumerate() {
            let tag = format!("s{k}_{gi}");
            let scale = |e: String, vec: bool| -> String {
                let mut e = e;
                for m in &ins.mods {
                    e = match m.as_str() {
                        "x2" => format!("({e} * 2.0)"),
                        "x4" => format!("({e} * 4.0)"),
                        "d2" => format!("({e} * 0.5)"),
                        "bx2" => format!("({e} * 2.0 - {})", if vec { "vec3<f32>(1.0)" } else { "1.0" }),
                        "bias" => format!("({e} - {})", if vec { "vec3<f32>(0.5)" } else { "0.5" }),
                        _ => e,
                    };
                }
                let (lo, hi) = if ins.mods.iter().any(|m| m == "sat") { ("0.0", "1.0") } else { ("-1.0", "1.0") };
                if vec {
                    format!("clamp({e}, vec3<f32>({lo}), vec3<f32>({hi}))")
                } else {
                    format!("clamp({e}, {lo}, {hi})")
                }
            };
            let n_dest = match ins.op.as_str() {
                "xmma" | "xmmc" => 3,
                "xdm" | "xdd" => 2,
                _ => 1,
            };
            let srcs = &ins.ops[n_dest..];
            // portions this instruction writes (from its first non-discard destination mask)
            let m = ins.ops[..n_dest].iter().find(|o| o.file != File::Discard).map(|o| o.mask).unwrap_or(0xF);
            let do_rgb = m & 0x7 != 0;
            let do_a = m & 0x8 != 0;
            for (portion, vec) in [(do_rgb, true), (do_a, false)] {
                if !portion {
                    continue;
                }
                let s: Vec<String> = srcs.iter().map(|o| if vec { rgb(o) } else { alpha(o) }).collect();
                let one = if vec { "vec3<f32>(1.0)" } else { "1.0" };
                let dot3 = |a: &str, b: &str| {
                    if vec { format!("vec3<f32>(dot({a}, {b}))") } else { format!("dot(({}).rgb, ({}).rgb)", a, b) }
                };
                let pn = if vec { "c" } else { "a" };
                let results: Vec<String> = match ins.op.as_str() {
                    "mov" => vec![s[0].clone()],
                    "add" => vec![format!("({} + {})", s[0], s[1])],
                    "sub" => vec![format!("({} - {})", s[0], s[1])],
                    "mul" => vec![format!("({} * {})", s[0], s[1])],
                    "mad" => vec![format!("({} * {} + {})", s[0], s[1], s[2])],
                    "lrp" => vec![format!("({0} * {1} + ({one} - {0}) * {2})", s[0], s[1], s[2])],
                    "cnd" => vec![format!("select({}, {}, {} > 0.5)", s[2], s[1], alpha(&srcs[0]))],
                    "dp3" => {
                        let (a, b) = (rgb(&srcs[0]), rgb(&srcs[1]));
                        vec![if vec { dot3(&a, &b) } else { format!("dot({a}, {b})") }]
                    }
                    "dp4" => {
                        let d4 = format!("(dot({}, {}) + {} * {})", rgb(&srcs[0]), rgb(&srcs[1]), alpha(&srcs[0]), alpha(&srcs[1]));
                        vec![if vec { format!("vec3<f32>{d4}") } else { d4 }]
                    }
                    "xmma" | "xmmc" => {
                        let a = format!("({} * {})", s[0], s[1]);
                        let b = format!("({} * {})", s[2], s[3]);
                        let sum = if ins.op == "xmma" {
                            format!("({a} + {b})")
                        } else {
                            format!("select({b}, {a}, r0.a > 0.5)")
                        };
                        vec![a, b, sum]
                    }
                    "xdm" | "xdd" => {
                        let (a0, a1) = (rgb(&srcs[0]), rgb(&srcs[1]));
                        let a = if vec { dot3(&a0, &a1) } else { format!("dot({a0}, {a1})") };
                        let b = if ins.op == "xdd" {
                            let (b0, b1) = (rgb(&srcs[2]), rgb(&srcs[3]));
                            if vec { dot3(&b0, &b1) } else { format!("dot({b0}, {b1})") }
                        } else {
                            format!("({} * {})", s[2], s[3])
                        };
                        vec![a, b]
                    }
                    _ => vec![s.first().cloned().unwrap_or_else(|| one.to_string())],
                };
                for (di, e) in results.iter().enumerate() {
                    let d = &ins.ops[di];
                    let v = format!("{tag}_{pn}{di}");
                    let _ = writeln!(w, "    let {v} = {};", scale(e.clone(), vec));
                    let name = match d.file {
                        File::Temp => format!("r{}", d.index),
                        File::Tex => format!("t{}", d.index),
                        File::Input => continue,
                        _ => continue,
                    };
                    let dm = d.mask;
                    if vec {
                        for (c, bit) in [('r', 0), ('g', 1), ('b', 2)] {
                            if dm & (1 << bit) != 0 {
                                commits.push(format!("    {name}.{c} = {v}.{c};"));
                            }
                        }
                    } else if dm & 8 != 0 {
                        commits.push(format!("    {name}.a = {v};"));
                    }
                }
            }
        }
        for c in commits {
            let _ = writeln!(w, "{c}");
        }
        k += group.len();
    }
    let _ = writeln!(w, "    return clamp(r0, vec4<f32>(0.0), vec4<f32>(1.0));\n}}");
    w
}

/// shared declarations used by both translations (output/input structs and helpers).
pub const COMMON_WGSL: &str = r#"
struct XvsOut {
    pos: vec4<f32>,
    d0: vec4<f32>,
    d1: vec4<f32>,
    t0: vec4<f32>,
    t1: vec4<f32>,
    t2: vec4<f32>,
    t3: vec4<f32>,
};
struct XpsIn {
    d0: vec4<f32>,
    d1: vec4<f32>,
    t0: vec4<f32>,
    t1: vec4<f32>,
    t2: vec4<f32>,
    t3: vec4<f32>,
};
fn xvs_rsq(x: f32) -> f32 {
    let a = abs(x);
    if (a == 0.0) { return 3.4e38; }
    if (a == 1.0) { return 1.0; }
    return inverseSqrt(a);
}
fn xvs_rcp(x: f32) -> f32 {
    if (x == 0.0) { return 3.4e38; }
    if (x == 1.0) { return 1.0; }
    return 1.0 / x;
}
// d3d lit: (1, max(x, 0), x > 0 && y > 0 ? y^clamp(w, -128, 128) : 0, 1)
fn xvs_lit(s: vec4<f32>) -> vec4<f32> {
    var r = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    if (s.x > 0.0) {
        r.y = s.x;
        if (s.y > 0.0) {
            let p = clamp(s.w, -128.0, 128.0);
            r.z = select(pow(s.y, p), 1.0, p == 0.0);
        }
    }
    return r;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn shader_files() -> Vec<(String, String)> {
        let dir = visos::file::game_dir().join("Shaders");
        let mut v = Vec::new();
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            let ext = p.extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if ext == "vxu" || ext == "pxu" {
                let b = std::fs::read(&p).unwrap();
                v.push((p.file_name().unwrap().to_string_lossy().into_owned(), String::from_utf8_lossy(&b).into_owned()));
            }
        }
        v.sort();
        v
    }

    /// stand-in bindings so the translated functions can be validated on their own
    const TEST_ENV: &str = r#"
fn xvs_c(i: i32) -> vec4<f32> { return vec4<f32>(f32(i)); }
fn xvs_c_rel(i: i32) -> vec4<f32> { return vec4<f32>(f32(i)); }
fn xps_c(i: i32) -> vec4<f32> { return vec4<f32>(f32(i)); }
fn xps_tex0(c: vec4<f32>) -> vec4<f32> { return c; }
fn xps_tex1(c: vec4<f32>) -> vec4<f32> { return c; }
fn xps_tex2(c: vec4<f32>) -> vec4<f32> { return c; }
fn xps_tex3(c: vec4<f32>) -> vec4<f32> { return c; }
const dyn_wvp = mat4x4<f32>();
const dyn_world = mat4x4<f32>();
const dyn_wv = mat4x4<f32>();
const dyn_vp = mat4x4<f32>();
const dyn_view = mat4x4<f32>();
const dyn_camera_dir = vec4<f32>();
"#;

    fn validate(src: &str) -> Result<(), String> {
        let m = naga::front::wgsl::parse_str(src).map_err(|e| e.emit_to_string(src))?;
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&m)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }

    #[test]
    fn translate_every_program() {
        let files = shader_files();
        assert!(files.len() >= 34, "{} shader files", files.len());
        for (name, src) in files {
            let p = parse(&src).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!p.instrs.is_empty(), "{name}: empty");
            let body = match p.unit {
                Unit::Vertex => vertex_wgsl(&p, &[(0, DynInput::WorldViewProj), (4, DynInput::World), (17, DynInput::CameraDir)]),
                Unit::Pixel => pixel_wgsl(&p),
            };
            let full = format!("{COMMON_WGSL}{TEST_ENV}{body}");
            validate(&full).unwrap_or_else(|e| panic!("{name}: {e}\n{body}"));
        }
    }

    #[test]
    fn parse_split_and_coissue() {
        // BlurH1.vxu splits an instruction over two lines; Default.pxu co-issues an alpha op
        let p = parse("xvs.1.0\nmov\n oT0, v9\nadd oT1, c0.yw, v9").unwrap();
        assert_eq!(p.instrs.len(), 2);
        assert_eq!(p.instrs[1].ops[1].swz, [1, 3, 3, 3]);
        let p = parse("xps.1.1\ntex t0\nmov r0.rgb, t0\n+ mul r0.a, t0.a, c12.a").unwrap();
        assert!(p.instrs[2].coissue);
        let p = parse("xvs.1.1\nmul r0, v0.y, c[-35 + a0.x]").unwrap();
        assert!(p.instrs[0].ops[2].relative && p.instrs[0].ops[2].index == -35);
    }
}
