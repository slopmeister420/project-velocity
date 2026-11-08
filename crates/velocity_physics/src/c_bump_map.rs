//! Include/Physics/CBumpMap.h — the terrain bump map: a 30x30 tile of heights repeated over the
//! world (mirrored about the origin), added to wheel contact depth scaled by the surface's
//! "Bump map height". The heights are static data in the executable (`CBumpMap_height` 0x00687B60),
//! the tile size the float after it (`CBumpMap_tileSize` 0x00688970, 100.0); both are read from
//! `default.xbe` at runtime. The inline methods are compiled into PhysicsManager.cpp / VehicleEntity.cpp.

use crate::material_properties::xbe_read;

pub const BUMP_MAP_SIZE: usize = 30;
pub const HEIGHT_VA: u32 = 0x00687B60;
pub const TILE_SIZE_VA: u32 = 0x00688970;

#[derive(Clone, Debug, PartialEq)]
pub struct CBumpMap {
    pub array_size: i32, // +0x0
    pub tile_size: f32,  // +0x4
    pub scale: f32,      // +0x8 (never read)
    pub enabled: i32,    // +0xC
    /// `CBumpMap_height[30][30]`, indexed `[x cell][z cell]`
    pub height: Vec<f32>,
}

impl CBumpMap {
    /// CBumpMap::CBumpMap 0x001B3BD0
    pub fn new(height: Vec<f32>) -> Self {
        Self { array_size: 30, tile_size: 0.0, scale: 0.0, enabled: 1, height }
    }

    /// CBumpMap::Init 0x001B3C10 (called as `Init(CBumpMap_tileSize, 30, 1.0f)`)
    pub fn init(&mut self, tile_size: f32, array_size: i32, scale: f32) {
        self.tile_size = tile_size;
        self.scale = scale;
        assert!(array_size == 30, "the preallocated array size has to be changed");
    }

    /// the static height table and tile size out of an XBE image
    pub fn from_xbe(xbe: &[u8]) -> Option<Self> {
        let h = xbe_read(xbe, HEIGHT_VA, 4 * BUMP_MAP_SIZE * BUMP_MAP_SIZE)?;
        let height = h.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
        let t = f32::from_le_bytes(xbe_read(xbe, TILE_SIZE_VA, 4)?.try_into().unwrap());
        let mut m = Self::new(height);
        m.init(t, 30, 1.0);
        Some(m)
    }

    /// `PhysicsManager::Init` 0x0019CA80 "Building bump map", from the install's `default.xbe`
    pub fn load() -> std::io::Result<Self> {
        let xbe = visos::file::platform::read(r"\default.xbe")?;
        Self::from_xbe(&xbe)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "default.xbe: no bump map table"))
    }

    // the original indexes the static array unchecked; out-of-range reads return 0 here
    #[inline]
    fn h(&self, i: i32, j: i32) -> f32 {
        if (0..30).contains(&i) && (0..30).contains(&j) {
            self.height[(i * 30 + j) as usize]
        } else {
            0.0
        }
    }

    /// CBumpMap::GetHeight 0x001DA2D0
    pub fn get_height(&self, mut x: f32, mut z: f32) -> f32 {
        if 0.0 > x {
            x = 0.0 - x;
        }
        if 0.0 > z {
            z = 0.0 - z;
        }
        let tx = (x / self.tile_size) as i32;
        let tz = (z / self.tile_size) as i32;
        let lx = x - tx as f32 * self.tile_size;
        let lz = z - tz as f32 * self.tile_size;
        let cell = self.tile_size / self.array_size as f32;
        let i = (lx / cell) as i32;
        let j = (lz / cell) as i32;
        let i1 = if i >= self.array_size - 1 { 0 } else { i + 1 };
        let j1 = if j >= self.array_size - 1 { 0 } else { j + 1 };
        let fx = lx - i as f32 * cell;
        let fz = lz - j as f32 * cell;
        if fx > fz {
            ((self.h(i1, j) - self.h(i, j)) * fx + (self.h(i1, j1) - self.h(i1, j)) * fz) / cell + self.h(i, j)
        } else {
            ((self.h(i1, j1) - self.h(i, j1)) * fx + (self.h(i, j1) - self.h(i, j)) * fz) / cell + self.h(i, j)
        }
    }
}
