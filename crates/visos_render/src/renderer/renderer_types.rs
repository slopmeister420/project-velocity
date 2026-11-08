//! include\Renderer\RendererTypes.h — the in-memory model layout (`XMF2`) shared by `.xmf` files and the models
//! embedded in compartments (`.cpt`). On disk every pointer is an offset from the model start; the game relocates
//! in place (GameDataLoading.cpp 0x0009EF30). Here the structures are read without relocation.

/// `'XMF2'`
pub const XMF2_ID: u32 = 0x584D_4632;

pub const SIZEOF_MODEL_OBJECT: usize = 96;
pub const SIZEOF_PART: usize = 16;
pub const SIZEOF_PRIMITIVE: usize = 16;
pub const SIZEOF_DRAW: usize = 60;
pub const SIZEOF_VERTEX_DESC: usize = 260;
pub const SIZEOF_VERTEX_BUFFER: usize = 24;
pub const SIZEOF_SHADER: usize = 20;
/// stage header; the command stream follows it. every shared shader has exactly one stage (RendererTypes.h:443)
pub const SIZEOF_STAGE: usize = 20;

/// D3DPT_TRIANGLELIST / D3DPT_TRIANGLESTRIP (xbox numbering)
pub const D3DPT_TRIANGLELIST: u32 = 5;
pub const D3DPT_TRIANGLESTRIP: u32 = 6;

/// xbox `D3DVSDT_*` vertex attribute formats
pub mod vsdt {
    pub const FLOAT1: u32 = 0x12;
    pub const FLOAT2: u32 = 0x22;
    pub const FLOAT3: u32 = 0x32;
    pub const FLOAT4: u32 = 0x42;
    pub const D3DCOLOR: u32 = 0x40;
    pub const NORMSHORT1: u32 = 0x11;
    pub const NORMSHORT2: u32 = 0x21;
    pub const NORMSHORT3: u32 = 0x31;
    pub const NORMSHORT4: u32 = 0x41;
    pub const SHORT1: u32 = 0x15;
    pub const SHORT2: u32 = 0x25;
    pub const SHORT3: u32 = 0x35;
    pub const PBYTE1: u32 = 0x14;
    pub const PBYTE2: u32 = 0x24;
    pub const PBYTE3: u32 = 0x34;
    pub const NORMPACKED3: u32 = 0x16;
    pub const PBYTE4: u32 = 0x44;
    pub const SHORT4: u32 = 0x45;
    pub const NONE: u32 = 0x02;
}

/// vertex shader input registers used by the data
pub mod vreg {
    pub const POSITION: usize = 0;
    pub const BLENDWEIGHT: usize = 1;
    pub const NORMAL: usize = 2;
    pub const DIFFUSE: usize = 3;
    pub const SPECULAR: usize = 4;
    pub const TEXCOORD0: usize = 9;
    pub const TEXCOORD1: usize = 10;
    pub const BLENDINDICES: usize = 15;
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("not an XMF2 model (id {0:08x})")]
    BadId(u32),
    #[error("model offset {0:#x} out of range")]
    Range(usize),
}

pub(crate) fn rd32(b: &[u8], o: usize) -> Result<u32, ModelError> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(ModelError::Range(o))
}
pub(crate) fn rd16(b: &[u8], o: usize) -> Result<u16, ModelError> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or(ModelError::Range(o))
}
pub(crate) fn rdf(b: &[u8], o: usize) -> Result<f32, ModelError> {
    rd32(b, o).map(f32::from_bits)
}
pub(crate) fn cstr(b: &[u8], o: usize) -> Result<String, ModelError> {
    let s = b.get(o..).ok_or(ModelError::Range(o))?;
    let n = s.iter().position(|&c| c == 0).ok_or(ModelError::Range(o))?;
    Ok(String::from_utf8_lossy(&s[..n]).into_owned())
}

/// `D3DVERTEXSHADERINPUT` (16 bytes)
#[derive(Clone, Copy, Debug, Default)]
pub struct VertexInput {
    pub stream: u32, // +0x0
    pub offset: u32, // +0x4
    pub format: u32, // +0x8
    pub tess: u32,   // +0xC
}

/// vertex description (260 bytes): stride then `D3DVERTEXATTRIBUTEFORMAT`.
#[derive(Clone, Debug)]
pub struct VertexDesc {
    pub stride: u32,               // +0x00
    pub inputs: [VertexInput; 16], // +0x04
}

/// vertex buffer record (24 bytes): self pointer, `D3DVertexBuffer` (12 bytes), data, size.
#[derive(Clone, Debug)]
pub struct VertexBuffer {
    pub data: u32, // +0x10
    pub size: u32, // +0x14
}

/// one draw call (60 bytes), consumed by Renderer.cpp 0x002A9160.
#[derive(Clone, Debug)]
pub struct Draw {
    pub vertex_desc: u32,    // +0x00 index, pointer after relocation
    pub vertex_buffer: u32,  // +0x04 index, pointer after relocation
    pub stream_offset: u32,  // +0x08 bytes into the vertex buffer
    pub primitive_type: u32, // +0x0C D3DPT_*
    pub index_kind: u32,     // +0x10 0 none, 1 index buffer, 2 push buffer
    pub index_count: u32,    // +0x2C
    pub index_data: u32,     // +0x30
    pub min_index: u16,      // +0x34
    pub max_index: u16,      // +0x36
    pub unk38: u16,          // +0x38
    pub start_index: u16,    // +0x3A
}

/// texture stage: 20-byte header followed by a variable-length command stream
#[derive(Clone, Debug)]
pub struct ShaderStage {
    pub texture_name: String, // +0x00 name pointer, replaced by the texture handle at load
    pub handle: u32,          // +0x04
    pub flags: u32,           // +0x08
    pub unk0c: u32,           // +0x0C
    pub unk10: u32,           // +0x10
    /// +0x14 stage command stream (`eMS_*` records: code u8, size u8, payload), terminated by code 0
    pub commands: Vec<u8>,
}

/// material (20 bytes): program, flags, name, stages.
#[derive(Clone, Debug)]
pub struct Shader {
    pub program: u32, // +0x00
    pub flags: u32,   // +0x04
    pub name: String, // +0x08
    pub stages: Vec<ShaderStage>,
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub shader: usize, // index into `Model::shaders`
    /// +0x04: the car shader type GameDataLoading.cpp 0x0009EAF0 starts from (and overwrites)
    pub kind: u32,
    pub draws: Vec<Draw>,
    /// skinned models: skeleton bone per blend index slot (the {count, offset} table after the part's primitives,
    /// relocated by GameDataLoading.cpp 0x0009EF30)
    pub palette: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct Part {
    pub matrix_at: u32, // +0x00 offset of the part's mesh -> model matrix (VertexIterator users read it)
    pub flags: u32, // +0x04
    pub primitives: Vec<Primitive>,
}

#[derive(Clone, Debug)]
pub struct ModelObject {
    pub matrix: [f32; 16], // +0x00 row-major, d3d convention (translation in the last row)
    pub sphere: [f32; 4],  // +0x40
    pub name: String,      // +0x58
    pub unk5c: u16,        // +0x5C
    pub parts: Vec<Part>,  // +0x54, count at +0x5E
}

/// a whole XMF2 model; `bytes` keeps the source for vertex/index reads.
#[derive(Clone, Debug)]
pub struct Model<'a> {
    pub bytes: &'a [u8],
    pub version: u32, // +0x04
    pub size: u32,    // +0x08
    pub flags: u32,   // +0x0C
    pub sphere: [f32; 4],
    pub vertex_descs: Vec<VertexDesc>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub objects: Vec<ModelObject>,
    pub shaders: Vec<Shader>,
    pub skin_count: u32, // +0x2C
}

impl<'a> Model<'a> {
    pub fn parse(b: &'a [u8]) -> Result<Self, ModelError> {
        let id = rd32(b, 0)?;
        if id != XMF2_ID {
            return Err(ModelError::BadId(id));
        }
        let vt = rd32(b, 32)? as usize;
        let (nd, ds, nv, vs) = (rd32(b, vt)?, rd32(b, vt + 4)? as usize, rd32(b, vt + 8)?, rd32(b, vt + 12)? as usize);
        let mut vertex_descs = Vec::new();
        for i in 0..nd as usize {
            let o = ds + SIZEOF_VERTEX_DESC * i;
            let mut inputs = [VertexInput::default(); 16];
            for (r, inp) in inputs.iter_mut().enumerate() {
                let p = o + 4 + 16 * r;
                *inp = VertexInput { stream: rd32(b, p)?, offset: rd32(b, p + 4)?, format: rd32(b, p + 8)?, tess: rd32(b, p + 12)? };
            }
            vertex_descs.push(VertexDesc { stride: rd32(b, o)?, inputs });
        }
        let mut vertex_buffers = Vec::new();
        for i in 0..nv as usize {
            let o = vs + SIZEOF_VERTEX_BUFFER * i;
            vertex_buffers.push(VertexBuffer { data: rd32(b, o + 16)?, size: rd32(b, o + 20)? });
        }
        let mut shaders: Vec<Shader> = Vec::new();
        let mut shader_at: Vec<u32> = Vec::new();
        let mut objects = Vec::new();
        let (nobj, objs) = (rd32(b, 36)?, rd32(b, 40)? as usize);
        for i in 0..nobj as usize {
            let o = objs + SIZEOF_MODEL_OBJECT * i;
            let mut matrix = [0f32; 16];
            for (k, m) in matrix.iter_mut().enumerate() {
                *m = rdf(b, o + 4 * k)?;
            }
            let sphere = [rdf(b, o + 64)?, rdf(b, o + 68)?, rdf(b, o + 72)?, rdf(b, o + 76)?];
            let parts_off = rd32(b, o + 84)? as usize;
            let name_off = rd32(b, o + 88)? as usize;
            let name = if name_off != 0 { cstr(b, name_off)? } else { String::new() };
            let nparts = *b.get(o + 94).ok_or(ModelError::Range(o + 94))?;
            let mut parts = Vec::new();
            for j in 0..nparts as usize {
                let po = parts_off + SIZEOF_PART * j;
                let nprim = rd16(b, po + 8)?;
                let prims = rd32(b, po + 12)? as usize;
                let skinned = rd32(b, 44)? == 1;
                let mut primitives = Vec::new();
                for k in 0..nprim as usize {
                    let qo = prims + SIZEOF_PRIMITIVE * k;
                    let sh = rd32(b, qo)?;
                    let shader = match shader_at.iter().position(|&s| s == sh) {
                        Some(p) => p,
                        None => {
                            shaders.push(Self::read_shader(b, sh as usize)?);
                            shader_at.push(sh);
                            shaders.len() - 1
                        }
                    };
                    let (nd, dr) = (rd32(b, qo + 8)?, rd32(b, qo + 12)? as usize);
                    let mut draws = Vec::new();
                    for l in 0..nd as usize {
                        let d = dr + SIZEOF_DRAW * l;
                        draws.push(Draw {
                            vertex_desc: rd32(b, d)?,
                            vertex_buffer: rd32(b, d + 4)?,
                            stream_offset: rd32(b, d + 8)?,
                            primitive_type: rd32(b, d + 12)?,
                            index_kind: rd32(b, d + 16)?,
                            index_count: rd32(b, d + 44)?,
                            index_data: rd32(b, d + 48)?,
                            min_index: rd16(b, d + 52)?,
                            max_index: rd16(b, d + 54)?,
                            unk38: rd16(b, d + 56)?,
                            start_index: rd16(b, d + 58)?,
                        });
                    }
                    let mut palette = Vec::new();
                    if skinned {
                        let t = prims + SIZEOF_PRIMITIVE * nprim as usize + 8 * k;
                        let (n, at) = (rd32(b, t)? as usize, rd32(b, t + 4)? as usize);
                        for i in 0..n {
                            palette.push(rd32(b, at + 4 * i)?);
                        }
                    }
                    let kind = rd32(b, qo + 4)?;
                    primitives.push(Primitive { shader, kind, draws, palette });
                }
                parts.push(Part { matrix_at: rd32(b, po)?, flags: rd32(b, po + 4)?, primitives });
            }
            objects.push(ModelObject { matrix, sphere, name, unk5c: rd16(b, o + 92)?, parts });
        }
        Ok(Self {
            bytes: b,
            version: rd32(b, 4)?,
            size: rd32(b, 8)?,
            flags: rd32(b, 12)?,
            sphere: [rdf(b, 16)?, rdf(b, 20)?, rdf(b, 24)?, rdf(b, 28)?],
            vertex_descs,
            vertex_buffers,
            objects,
            shaders,
            skin_count: rd32(b, 44)?,
        })
    }

    /// GameDataLoading.cpp 0x000A3CB0 — locator matrix by name (flags bit 0; +0x34 -> {count, matrices, names[]})
    pub fn locator(&self, name: &str) -> Option<[f32; 16]> {
        if self.flags & 1 == 0 {
            return None;
        }
        let b = self.bytes;
        let t = rd32(b, 52).ok()? as usize;
        let (n, mats) = (rd32(b, t).ok()? as usize, rd32(b, t + 4).ok()? as usize);
        for i in 0..n {
            let at = rd32(b, t + 8 + 4 * i).ok()? as usize;
            if cstr(b, at).ok()? == name {
                let mut m = [0f32; 16];
                for (k, v) in m.iter_mut().enumerate() {
                    *v = rdf(b, mats + 64 * i + 4 * k).ok()?;
                }
                return Some(m);
            }
        }
        None
    }

    /// ShaderManagerBase.cpp 0x002A31C0 points at stage+0x14; records are walked until a zero code.
    fn read_commands(b: &[u8], o: usize) -> Result<Vec<u8>, ModelError> {
        let mut p = o;
        loop {
            let code = *b.get(p).ok_or(ModelError::Range(p))?;
            if code == 0 {
                return Ok(b[o..=p].to_vec());
            }
            let size = *b.get(p + 1).ok_or(ModelError::Range(p + 1))? as usize;
            if size == 0 {
                return Err(ModelError::Range(p));
            }
            p += size;
        }
    }

    fn read_shader(b: &[u8], o: usize) -> Result<Shader, ModelError> {
        let name_off = rd32(b, o + 8)? as usize;
        let (n, st) = (rd32(b, o + 12)?, rd32(b, o + 16)? as usize);
        let mut stages = Vec::new();
        for i in 0..n as usize {
            let s = st + SIZEOF_STAGE * i;
            let tn = rd32(b, s)? as usize;
            stages.push(ShaderStage {
                texture_name: if tn != 0 { cstr(b, tn)? } else { String::new() },
                handle: rd32(b, s + 4)?,
                flags: rd32(b, s + 8)?,
                unk0c: rd32(b, s + 12)?,
                unk10: rd32(b, s + 16)?,
                commands: Self::read_commands(b, s + 20)?,
            });
        }
        Ok(Shader {
            program: rd32(b, o)?,
            flags: rd32(b, o + 4)?,
            name: if name_off != 0 { cstr(b, name_off)? } else { String::new() },
            stages,
        })
    }

    /// triangle-list indices of a draw (relative to its stream offset); strips are expanded with alternating winding.
    pub fn draw_indices(&self, d: &Draw) -> Result<Vec<u32>, ModelError> {
        let n = d.index_count as usize;
        let base = d.index_data as usize + 2 * d.start_index as usize;
        let mut raw = Vec::with_capacity(n);
        for i in 0..n {
            raw.push(rd16(self.bytes, base + 2 * i)? as u32);
        }
        Ok(match d.primitive_type {
            D3DPT_TRIANGLESTRIP => {
                let mut out = Vec::new();
                for i in 0..n.saturating_sub(2) {
                    let (a, b, c) = (raw[i], raw[i + 1], raw[i + 2]);
                    if a == b || b == c || a == c {
                        continue;
                    }
                    if i & 1 == 0 { out.extend([a, b, c]) } else { out.extend([b, a, c]) }
                }
                out
            }
            _ => raw,
        })
    }

    /// read attribute register `reg` of vertex `i` of a draw as up to 4 floats (`None` if absent).
    pub fn read_attr(&self, d: &Draw, reg: usize, i: u32) -> Result<Option<[f32; 4]>, ModelError> {
        let desc = self.vertex_descs.get(d.vertex_desc as usize).ok_or(ModelError::Range(d.vertex_desc as usize))?;
        let vb = self.vertex_buffers.get(d.vertex_buffer as usize).ok_or(ModelError::Range(d.vertex_buffer as usize))?;
        let inp = desc.inputs[reg];
        let o = vb.data as usize + d.stream_offset as usize + (i * desc.stride + inp.offset) as usize;
        let b = self.bytes;
        Ok(Some(match inp.format {
            vsdt::FLOAT1 => [rdf(b, o)?, 0.0, 0.0, 1.0],
            vsdt::FLOAT2 => [rdf(b, o)?, rdf(b, o + 4)?, 0.0, 1.0],
            vsdt::FLOAT3 => [rdf(b, o)?, rdf(b, o + 4)?, rdf(b, o + 8)?, 1.0],
            vsdt::FLOAT4 => [rdf(b, o)?, rdf(b, o + 4)?, rdf(b, o + 8)?, rdf(b, o + 12)?],
            // D3DCOLOR is argb in a dword: b, g, r, a in memory. returned as r, g, b, a in 0..1
            vsdt::D3DCOLOR => {
                let c = rd32(b, o)?;
                let f = |s: u32| ((c >> s) & 0xFF) as f32 / 255.0;
                [f(16), f(8), f(0), f(24)]
            }
            vsdt::NORMPACKED3 => {
                let c = rd32(b, o)?;
                let x = ((c << 21) as i32 >> 21) as f32 / 1023.0;
                let y = (((c >> 11) << 21) as i32 >> 21) as f32 / 1023.0;
                let z = ((c >> 22) << 22) as i32 >> 22;
                [x, y, z as f32 / 511.0, 1.0]
            }
            // NV2A normalised shorts: i16 / 32767; missing components 0, w 1
            vsdt::NORMSHORT1 | vsdt::NORMSHORT2 | vsdt::NORMSHORT3 | vsdt::NORMSHORT4 => {
                let n = (inp.format >> 4) as usize;
                let r = |k: usize| if k < n { rd16(b, o + 2 * k).map(|v| v as i16 as f32 / 32767.0) } else { Ok(if k == 3 { 1.0 } else { 0.0 }) };
                [r(0)?, r(1)?, r(2)?, r(3)?]
            }
            vsdt::PBYTE1 | vsdt::PBYTE2 | vsdt::PBYTE3 | vsdt::PBYTE4 => {
                let n = (inp.format >> 4) as usize;
                let s = b.get(o..o + n).ok_or(ModelError::Range(o))?;
                let g = |k: usize| if k < n { s[k] as f32 / 255.0 } else if k == 3 { 1.0 } else { 0.0 };
                [g(0), g(1), g(2), g(3)]
            }
            vsdt::SHORT1 | vsdt::SHORT2 | vsdt::SHORT3 | vsdt::SHORT4 => {
                let n = (inp.format >> 4) as usize;
                let r = |k: usize| if k < n { rd16(b, o + 2 * k).map(|v| v as i16 as f32) } else { Ok(if k == 3 { 1.0 } else { 0.0 }) };
                [r(0)?, r(1)?, r(2)?, r(3)?]
            }
            _ => return Ok(None),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compartment_sys::Compartment;

    /// every XMF2 model in the install (loose `.xmf` in VCDs and models inside compartments) parses, all draws
    /// are indexed lists whose indices stay inside their vertex buffer.
    #[test]
    fn parse_every_model() {
        let (mut models, mut draws, mut xmf1) = (0, 0, 0);
        let mut bad = Vec::new();
        // packed-uv census (uv0 NORMSHORT2 = GDL part flag 8 → DefaultPacked uv × 8) -> research/gfx/packed_uv_models.txt
        let packed = std::cell::RefCell::new(std::collections::BTreeSet::new());
        let check = |name: &str, b: &[u8], bad: &mut Vec<String>| -> usize {
            match Model::parse(b) {
                Ok(m) => {
                    let mut n = 0;
                    for o in &m.objects {
                        for p in &o.parts {
                            for q in &p.primitives {
                                for d in &q.draws {
                                    n += 1;
                                    let idx = match m.draw_indices(d) {
                                        Ok(i) => i,
                                        Err(e) => {
                                            bad.push(format!("{name}: {e}"));
                                            continue;
                                        }
                                    };
                                    let max = idx.iter().copied().max().unwrap_or(0);
                                    if let Err(e) = m.read_attr(d, vreg::POSITION, max) {
                                        bad.push(format!("{name}: vertex {max}: {e}"));
                                    }
                                    // every declared attribute must decode (no format falls to None)
                                    if let Some(desc) = m.vertex_descs.get(d.vertex_desc as usize) {
                                        if desc.inputs[vreg::TEXCOORD0].format == vsdt::NORMSHORT2 {
                                            packed.borrow_mut().insert(name.split(['#']).next().unwrap_or(name).to_string());
                                        }
                                        for (reg, inp) in desc.inputs.iter().enumerate() {
                                            if inp.format == vsdt::NONE || inp.format == 0 {
                                                continue;
                                            }
                                            if !matches!(m.read_attr(d, reg, 0), Ok(Some(_))) {
                                                bad.push(format!("{name}: reg {reg} format {:#x} undecoded", inp.format));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    n
                }
                Err(e) => {
                    bad.push(format!("{name}: {e}"));
                    0
                }
            }
        };
        for p in crate::test_util::all_vcds() {
            let Ok(vcd) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            for e in vcd.entries() {
                let l = e.name.to_ascii_lowercase();
                let Some(d) = vcd.data(&e) else { continue };
                if l.ends_with(".xmf") {
                    if d.starts_with(b"1FMX") {
                        xmf1 += 1;
                        continue;
                    }
                    models += 1;
                    draws += check(&format!("{}:{}", p.display(), e.name), d, &mut bad);
                } else if l.ends_with(".cpt") {
                    let c = Compartment::parse(d).unwrap();
                    for r in c.models() {
                        models += 1;
                        draws += check(&format!("{}:{}#{}", p.display(), e.name, r.index), r.data, &mut bad);
                    }
                }
            }
        }
        let pk: Vec<String> = packed.borrow().iter().cloned().collect();
        let _ = std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/packed_uv_models.txt"), pk.join("\n") + "\n");
        assert!(bad.is_empty(), "{} failures: {:#?}", bad.len(), &bad[..bad.len().min(20)]);
        assert!(models > 8000, "{models}");
        eprintln!("{models} models, {draws} draws ({xmf1} legacy XMF1 skipped)");
    }

    /// census of `eMS_Stencil` records (glow marks) and destination-alpha blend modes in the data
    #[test]
    #[ignore]
    fn stencil_census() {
        use crate::renderer::shader_manager::{ems, StageState};
        use std::collections::BTreeMap;
        let mut stencil: BTreeMap<String, usize> = BTreeMap::new();
        let mut dest_alpha: BTreeMap<String, usize> = BTreeMap::new();
        let mut scan = |src: &str, b: &[u8]| {
            let Ok(m) = Model::parse(b) else { return };
            for sh in &m.shaders {
                for st in &sh.stages {
                    let c = &st.commands;
                    let s = StageState::from_commands(c);
                    let mut p = 0usize;
                    while let Some(&code) = c.get(p) {
                        let size = c.get(p + 1).copied().unwrap_or(0) as usize;
                        if code == 0 || size == 0 {
                            break;
                        }
                        if code == ems::STENCIL {
                            *stencil.entry(format!("flag {} blend {} {} ({src})", c.get(p + 2).copied().unwrap_or(0), s.blend_mode, st.texture_name)).or_default() += 1;
                        }
                        p += size;
                    }
                    if matches!(s.blend_mode, 5 | 6 | 8 | 10 | 13) {
                        *dest_alpha.entry(format!("blend {} {} ({src})", s.blend_mode, st.texture_name)).or_default() += 1;
                    }
                }
            }
        };
        for p in crate::test_util::all_vcds() {
            let Ok(vcd) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            let src = p.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
            for e in vcd.entries() {
                let l = e.name.to_ascii_lowercase();
                let Some(d) = vcd.data(&e) else { continue };
                if l.ends_with(".xmf") && !d.starts_with(b"1FMX") {
                    scan(&src, d);
                } else if l.ends_with(".cpt") {
                    if let Ok(c) = Compartment::parse(d) {
                        for r in c.models() {
                            scan(&src, r.data);
                        }
                    }
                }
            }
        }
        let out: String = stencil.iter().map(|(k, n)| format!("stencil {n} {k}\n")).chain(dest_alpha.iter().map(|(k, n)| format!("destalpha {n} {k}\n"))).collect();
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/stencil_census.txt"), out).unwrap();
    }

    /// shader +4 flags = DrawList masks (DrawList.cpp 0x002C2650); writes research/gfx/shader_flags_census.txt
    #[test]
    #[ignore]
    fn shader_flags_census() {
        use std::collections::BTreeMap;
        let mut flags: BTreeMap<String, usize> = BTreeMap::new();
        let mut scan = |src: &str, b: &[u8]| {
            let Ok(m) = Model::parse(b) else { return };
            for sh in &m.shaders {
                let tex = sh.stages.first().map(|s| s.texture_name.to_ascii_lowercase()).unwrap_or_default();
                *flags.entry(format!("flags {:#x} {src}", sh.flags)).or_default() += 1;
                // GDL_RelocateModel 0x0009EF30: these names get +4 |= 0x10 (the road detail draw list)
                if sh.name.contains("c_road") || sh.name.contains("Specular_Road") {
                    *flags.entry(format!("road {} {src} {tex}", sh.name)).or_default() += 1;
                }
                // "_reflect" -> 0x80 (list `dword_6D5330`), "windowenv_" -> 0x100, "Overlay_" -> 0x20 / 0x200
                for k in ["_reflect", "windowenv_", "Overlay_"] {
                    if sh.name.contains(k) {
                        *flags.entry(format!("name{k} {} {src} {tex}", sh.name)).or_default() += 1;
                    }
                }
            }
        };
        for p in crate::test_util::all_vcds() {
            let Ok(vcd) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            let src = p.strip_prefix(visos::file::game_dir()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
            for e in vcd.entries() {
                let l = e.name.to_ascii_lowercase();
                let Some(d) = vcd.data(&e) else { continue };
                if l.ends_with(".xmf") && !d.starts_with(b"1FMX") {
                    scan(&src, d);
                } else if l.ends_with(".cpt") {
                    if let Ok(c) = Compartment::parse(d) {
                        for r in c.models() {
                            scan(&src, r.data);
                        }
                    }
                }
            }
        }
        let out: String = flags.iter().map(|(k, n)| format!("{n} {k}\n")).collect();
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/shader_flags_census.txt"), out).unwrap();
    }

    /// dev dump (`--ignored`): hangout.xmf shaders, textures (alpha / black share), per-object facing vs the main-menu
    /// camera and the floors' strip winding runs -> `research/gfx/hangout_dump.txt`
    #[test]
    #[ignore]
    fn hangout_dump() {
        let b = std::fs::read(visos::file::resolve(r"\Foyers\hangout\Model\hangout.xmf")).unwrap();
        let m = Model::parse(&b).unwrap();
        let mut out = String::new();
        for (i, s) in m.shaders.iter().enumerate() {
            for st in &s.stages {
                out += &format!("shader {i} prog {:#x} flags {:#x} name {} stage tex {} flags {:#x} cmds {:02x?}\n", s.program, s.flags, s.name, st.texture_name, st.flags, st.commands);
            }
        }
        let (mut sum, mut n, mut amin, mut amax) = ([0.0f32; 4], 0u32, 1.0f32, 0.0f32);
        for o in &m.objects {
            for p in &o.parts {
                for q in &p.primitives {
                    for d in &q.draws {
                        let idx = m.draw_indices(d).unwrap();
                        let desc = &m.vertex_descs[d.vertex_desc as usize];
                        out += &format!("obj {} shader {} draw desc {} stride {} inputs {:?}\n", o.name, q.shader, d.vertex_desc, desc.stride,
                            desc.inputs.iter().enumerate().filter(|(_, x)| x.format != 2).map(|(r, x)| (r, x.offset, x.format)).collect::<Vec<_>>());
                        for &v in &idx {
                            if let Ok(Some(c)) = m.read_attr(d, vreg::DIFFUSE, v as u32) {
                                for k in 0..4 { sum[k] += c[k]; }
                                n += 1;
                                amin = amin.min(c[3]);
                                amax = amax.max(c[3]);
                            }
                        }
                    }
                }
            }
        }
        out += &format!("diffuse avg {:?} n {n} alpha {amin}..{amax}\n", sum.map(|x| x / n as f32));
        let v = visos::file::Vcd::open(r"\Foyers\hangout\Model\HANGOUTT.VCD").unwrap();
        let names: Vec<String> = v.entries().map(|e| e.name.to_string()).collect();
        out += &format!("vcd entries {names:?}\n");
        for s in &m.shaders {
            for st in &s.stages {
                let e = v.entries().find(|e| e.name.trim_end_matches(".vtf").eq_ignore_ascii_case(&st.texture_name));
                match e.and_then(|e| v.data(&e).map(|d| d.to_vec())) {
                    None => out += &format!("tex {} MISSING\n", st.texture_name),
                    Some(d) => {
                        let fmt = d.get(0x14).copied().unwrap_or(0xff);
                        match crate::renderer::texture_manager::decode_any_vtf(&d) {
                            Ok(t) => {
                                let l = &t.levels[0];
                                let px = l.len() / 4;
                                let (mut lo, mut amin, mut black) = (0, 255u8, 0);
                                for p in l.chunks(4) {
                                    amin = amin.min(p[3]);
                                    if (p[3] as f32 * 0.50196) <= 40.0 { lo += 1; }
                                    if p[0] < 8 && p[1] < 8 && p[2] < 8 { black += 1; }
                                }
                                out += &format!("tex {} fmt {fmt} {}x{} amin {amin} alpha-fail {:.3} black {:.3}\n", st.texture_name, t.width, t.height, lo as f32 / px as f32, black as f32 / px as f32);
                            }
                            Err(e) => out += &format!("tex {} fmt {fmt} DECODE ERR {e}\n", st.texture_name),
                        }
                    }
                }
            }
        }
        for o in &m.objects {
            let a = &o.matrix;
            let det = a[0] * (a[5] * a[10] - a[6] * a[9]) - a[1] * (a[4] * a[10] - a[6] * a[8]) + a[2] * (a[4] * a[9] - a[5] * a[8]);
            let sh: Vec<usize> = o.parts.iter().flat_map(|p| p.primitives.iter().map(|q| q.shader)).collect();
            // facing vs the main-menu camera: ccw-front triangles towards / away
            let cam = [0.25f32, 2.94, 6.61];
            let (mut front, mut back, mut up, mut down) = (0, 0, 0, 0);
            for p in &o.parts {
                for q in &p.primitives {
                    for d in &q.draws {
                        let idx = m.draw_indices(d).unwrap();
                        let w = |i: u32| {
                            let v = m.read_attr(d, vreg::POSITION, i).unwrap().unwrap();
                            [0, 1, 2].map(|c| v[0] * a[c] + v[1] * a[4 + c] + v[2] * a[8 + c] + a[12 + c])
                        };
                        for t in idx.chunks(3) {
                            let (p0, p1, p2) = (w(t[0]), w(t[1]), w(t[2]));
                            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                            let nrm = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                            let dd = (cam[0] - p0[0]) * nrm[0] + (cam[1] - p0[1]) * nrm[1] + (cam[2] - p0[2]) * nrm[2];
                            if dd > 0.0 { front += 1 } else { back += 1 }
                            let l = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
                            if o.name.starts_with("floor") && l > 0.0 && (nrm[1] / l).abs() > 0.9 {
                                if nrm[1] > 0.0 { up += 1 } else { down += 1 }
                            }
                        }
                    }
                }
            }
            if o.name.starts_with("floor") {
                // duplicate test: flat triangles keyed by their sorted rounded positions
                let mut keys: std::collections::HashMap<Vec<[i32; 3]>, (u32, u32)> = std::collections::HashMap::new();
                for p in &o.parts {
                    for q in &p.primitives {
                        for d in &q.draws {
                            let idx = m.draw_indices(d).unwrap();
                            let w = |i: u32| {
                                let v = m.read_attr(d, vreg::POSITION, i).unwrap().unwrap();
                                [0, 1, 2].map(|c| v[0] * a[c] + v[1] * a[4 + c] + v[2] * a[8 + c] + a[12 + c])
                            };
                            for t in idx.chunks(3) {
                                let ps = [w(t[0]), w(t[1]), w(t[2])];
                                let ny = (ps[1][2] - ps[0][2]) * (ps[2][0] - ps[0][0]) - (ps[1][0] - ps[0][0]) * (ps[2][2] - ps[0][2]);
                                let mut k: Vec<[i32; 3]> = ps.iter().map(|p| p.map(|x| (x * 100.0).round() as i32)).collect();
                                k.sort();
                                let e = keys.entry(k).or_default();
                                if ny > 0.0 { e.0 += 1 } else { e.1 += 1 }
                            }
                        }
                    }
                }
                let both = keys.values().filter(|e| e.0 > 0 && e.1 > 0).count();
                out += &format!("object {} distinct tris {} with both windings {both}\n", o.name, keys.len());
                // raw strip: per draw, the up/down sign sequence of the flat triangles (u/d, '.' degenerate or not flat)
                for p in &o.parts {
                    for q in &p.primitives {
                        for d in &q.draws {
                            let base = d.index_data as usize + 2 * d.start_index as usize;
                            let raw: Vec<u32> = (0..d.index_count as usize).map(|i| rd16(m.bytes, base + 2 * i).unwrap() as u32).collect();
                            let w = |i: u32| {
                                let v = m.read_attr(d, vreg::POSITION, i).unwrap().unwrap();
                                [0, 1, 2].map(|c| v[0] * a[c] + v[1] * a[4 + c] + v[2] * a[8 + c] + a[12 + c])
                            };
                            let mut s = String::new();
                            for i in 0..raw.len().saturating_sub(2) {
                                let (x, y, z) = (raw[i], raw[i + 1], raw[i + 2]);
                                if x == y || y == z || x == z { s.push('.'); continue; }
                                let ps = [w(x), w(y), w(z)];
                                let ny = (ps[1][2] - ps[0][2]) * (ps[2][0] - ps[0][0]) - (ps[1][0] - ps[0][0]) * (ps[2][2] - ps[0][2]);
                                // unswapped winding sign, flipped on odd i = the d3d strip rule
                                let up = (ny > 0.0) != (i & 1 == 1);
                                s.push(if ny.abs() < 1e-6 { '0' } else if up { 'u' } else { 'd' });
                            }
                            out += &format!("  strip n {} start {} : {}\n", raw.len(), d.start_index, &s[..s.len().min(160)]);
                        }
                    }
                }
            }
            out += &format!("object {} facing cam front {front} back {back} flat up {up} down {down}\n", o.name);
            out += &format!("object {} det {det:.4} diag {:.3} {:.3} {:.3} prims {sh:?} pt {:?}\n", o.name, a[0], a[5], a[10],
                o.parts.iter().flat_map(|p| p.primitives.iter().flat_map(|q| q.draws.iter().map(|d| d.primitive_type))).collect::<Vec<_>>());
        }
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/hangout_dump.txt"), out).unwrap();
    }

    /// dev dump (`--ignored`): hangout.xmf per-shader uv range / texel density and texture means, level-0 rgba of
    /// every texture -> `research/gfx/hangout_tex/` (raw, `name_WxH.rgba`) + `hangout_uv.txt`
    #[test]
    #[ignore]
    fn hangout_uv_dump() {
        let b = std::fs::read(visos::file::resolve(r"\Foyers\hangout\Model\hangout.xmf")).unwrap();
        let m = Model::parse(&b).unwrap();
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/hangout_tex");
        std::fs::create_dir_all(dir).unwrap();
        let v = visos::file::Vcd::open(r"\Foyers\hangout\Model\HANGOUTT.VCD").unwrap();
        let mut out = String::new();
        for (si, s) in m.shaders.iter().enumerate() {
            let (mut lo, mut hi, mut n) = ([f32::MAX; 2], [f32::MIN; 2], 0);
            for o in &m.objects {
                for p in &o.parts {
                    for q in p.primitives.iter().filter(|q| q.shader == si) {
                        for d in &q.draws {
                            for &i in &m.draw_indices(d).unwrap() {
                                if let Ok(Some(t)) = m.read_attr(d, vreg::TEXCOORD0, i as u32) {
                                    for k in 0..2 { lo[k] = lo[k].min(t[k]); hi[k] = hi[k].max(t[k]); }
                                    n += 1;
                                }
                            }
                        }
                    }
                }
            }
            let st = &s.stages[0];
            let mut tex = String::new();
            if let Some(e) = v.entries().find(|e| e.name.trim_end_matches(".vtf").eq_ignore_ascii_case(&st.texture_name)) {
                let d = v.data(&e).unwrap().to_vec();
                if let Ok(t) = crate::renderer::texture_manager::decode_any_vtf(&d) {
                    let l = &t.levels[0];
                    let mut sum = [0u64; 4];
                    for p in l.chunks(4) { for k in 0..4 { sum[k] += p[k] as u64; } }
                    let px = (l.len() / 4) as u64;
                    tex = format!("{}x{} levels {} mean {:?} hdr {:02x?}", t.width, t.height, t.levels.len(), sum.map(|x| x / px), &d[..0x20.min(d.len())]);
                    std::fs::write(format!("{dir}/{}_{}x{}.rgba", st.texture_name, t.width, t.height), l).unwrap();
                }
            }
            out += &format!("shader {si} {} tex {} n {n} uv {lo:?}..{hi:?} {tex}\n", s.name, st.texture_name);
        }
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/hangout_uv.txt"), out).unwrap();
    }
}
