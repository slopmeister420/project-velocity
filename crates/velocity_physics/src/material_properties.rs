//! src/Physics/MaterialProperties.cpp — surface material table (friction, drag, bump height).
//!
//! The table is a static initialised array (`g_materialProperties` 0x006879C0, 26 x 12 bytes); the only
//! runtime override is the debug tweak tree (`Materials.<NAME>`, `CConfig::AddMaterialTweaks` 0x0019BEC0),
//! which no shipped file sets. `Shaders\material.txt` is an exporter-side table the game never opens.

/// eSURF_TYPE_MAX_TYPES
pub const SURF_TYPE_MAX_TYPES: usize = 26;

/// `g_materialNames` 0x00687AF8, in eSURF order
pub const SURF_NAMES: [&str; SURF_TYPE_MAX_TYPES] = [
    "DEFAULT", "WATER", "COBBLES", "COBBLES_ICY", "COBBLES_WET", "CONCRETE", "CONCRETE_OLD", "CONCRETE_ICY",
    "CONCRETE_WET", "DIRT", "DIRT_WET", "GRASS", "GRAVEL_DEEP", "GRAVEL_LOOSE", "TARMAC", "TARMAC_ICY",
    "TARMAC_OLD", "TARMAC_WET", "BRICK", "METAL", "PLASTIC", "BARRIER", "ROOFTILE", "MARBLE", "CARPET",
    "NOT_DRIVABLE",
];

/// one 12-byte entry
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialProperty {
    pub friction_multiplier: f32, // +0
    pub drag: f32,                // +4
    pub bump_map_height: f32,     // +8
}

const fn mp(f: u32, d: u32, b: u32) -> MaterialProperty {
    MaterialProperty {
        friction_multiplier: f32::from_bits(f),
        drag: f32::from_bits(d),
        bump_map_height: f32::from_bits(b),
    }
}
const STD: MaterialProperty = mp(0x3F800000, 0, 0x3C75C28F); // 1.0, 0.0, 0.015

/// initial contents of `g_materialProperties` 0x006879C0 (bit-exact)
pub const DEFAULT_MATERIALS: [MaterialProperty; SURF_TYPE_MAX_TYPES] = [
    STD, STD, STD, STD, STD, STD, STD, STD, STD, STD, STD,
    mp(0x3F4CCCCD, 0x3BC49BA6, 0x3D4CCCCD), // GRASS 0.8, 0.006, 0.05
    STD,
    mp(0x3F000000, 0, 0x3D8F5C29), // GRAVEL_LOOSE 0.5, 0, 0.07
    STD, STD, STD,
    mp(0x3E99999A, 0, 0x3C75C28F), // TARMAC_WET 0.3, 0, 0.015
    STD, STD, STD, STD, STD, STD, STD, STD,
];

/// the material table (`g_materialProperties`)
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialProperties {
    pub table: [MaterialProperty; SURF_TYPE_MAX_TYPES],
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self { table: DEFAULT_MATERIALS }
    }
}

impl MaterialProperties {
    /// MaterialProperties::GetName 0x0019C110
    pub fn get_name(ty: u32) -> &'static str {
        assert!((ty as usize) < SURF_TYPE_MAX_TYPES, "type >= 0 && type < eSURF_TYPE_MAX_TYPES");
        SURF_NAMES[ty as usize]
    }

    /// MaterialProperties::Get 0x0019C1A0
    pub fn get(&self, ty: u32) -> &MaterialProperty {
        assert!((ty as usize) < SURF_TYPE_MAX_TYPES, "type >= 0 && type < eSURF_TYPE_MAX_TYPES");
        &self.table[ty as usize]
    }

    /// set a value by its tweak path (`Materials.<NAME>` / `Friction multiplier|Drag|Bump map height`),
    /// the effect of `tweak.set` on the tree `CConfig::AddMaterialTweaks` registers (entries 1..25 only)
    pub fn set_tweak(&mut self, name: &str, field: &str, v: f32) -> bool {
        let Some(i) = (1..SURF_TYPE_MAX_TYPES).find(|&i| SURF_NAMES[i].eq_ignore_ascii_case(name)) else {
            return false;
        };
        let e = &mut self.table[i];
        match field {
            "Friction multiplier" => e.friction_multiplier = v,
            "Drag" => e.drag = v,
            "Bump map height" => e.bump_map_height = v,
            _ => return false,
        }
        true
    }
}

/// `Shaders\material.txt`: tab-separated rows `friction` / `drag` / `bump map height`, one column per
/// surface, headed by a `#`-comment naming the columns (lowercase, spaces). Tool data, not read by the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialTxt {
    pub columns: Vec<String>,
    pub rows: Vec<(String, Vec<f32>)>,
}

impl MaterialTxt {
    pub fn parse(text: &str) -> Option<Self> {
        let mut t = MaterialTxt::default();
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix('#') {
                let cols: Vec<&str> = rest.split('\t').collect();
                if t.columns.is_empty() && cols.len() > 2 && cols[0].is_empty() {
                    t.columns = cols[1..].iter().map(|s| s.to_string()).collect();
                }
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            let mut it = line.split('\t');
            let key = it.next()?.to_string();
            let vals = it.map(|s| s.trim().parse::<f32>().ok()).collect::<Option<Vec<_>>>()?;
            t.rows.push((key, vals));
        }
        Some(t)
    }

    /// the eSURF index of a column (`cobbles icy` -> COBBLES_ICY)
    pub fn surf_index(column: &str) -> Option<usize> {
        let n = column.trim().replace(' ', "_");
        SURF_NAMES.iter().position(|s| s.eq_ignore_ascii_case(&n))
    }
}

/// read `len` bytes at virtual address `va` of an XBE image (static data the port pulls from `default.xbe`)
pub fn xbe_read(xbe: &[u8], va: u32, len: usize) -> Option<&[u8]> {
    let u = |o: usize| xbe.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    if xbe.get(0..4)? != b"XBEH" {
        return None;
    }
    let base = u(0x104)?;
    let count = u(0x11C)? as usize;
    let hdrs = u(0x120)?.checked_sub(base)? as usize;
    for s in 0..count {
        let h = hdrs + 56 * s;
        let (vaddr, vsize, raw, rsize) = (u(h + 4)?, u(h + 8)?, u(h + 12)?, u(h + 16)?);
        if va >= vaddr && (va as u64 + len as u64) <= vaddr as u64 + vsize.min(rsize) as u64 {
            let o = (raw + (va - vaddr)) as usize;
            return xbe.get(o..o + len);
        }
    }
    None
}
