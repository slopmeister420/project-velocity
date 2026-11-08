//! VsPoolAllocator.cpp range (0x00318D00..0x00321xxx; the region also holds the particle random helpers and the
//! emitter shapes elParticle calls): the block pool every elParticleSystem keeps its particles in, the 15-bit
//! random generator and the emitter shape sampling.

use std::collections::VecDeque;

use bevy::math::{Mat4, Vec3};

/// `dword_6C2430` start value (data)
pub const RAND_SEED: u32 = 0x39D4_C6BA;

/// the particle random generator (0x00320BB0 / 0x00320C60 / 0x00320E10)
#[derive(Clone, Copy, Debug)]
pub struct VsRand {
    pub seed: u32,
}

impl Default for VsRand {
    fn default() -> Self {
        Self { seed: RAND_SEED }
    }
}

impl VsRand {
    /// 0x00320BB0: `s = (-24883·(u16)s − 3561) & 0x7FFF`
    pub fn next(&mut self) -> i32 {
        self.seed = ((self.seed as u16 as i32).wrapping_mul(-24883).wrapping_sub(3561) & 0x7FFF) as u32;
        self.seed as i32
    }

    /// 0x00320C60: `rand · (hi − lo) / 32767 + lo` (x87, stored as float)
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        let r = self.next() as f64;
        (r * (hi as f64 - lo as f64) / 32767.0 + lo as f64) as f32
    }

    /// 0x00320E10: a point in the unit ball (x, y, z drawn in [-1, 1] until |p|² ≤ 1), times `r`
    pub fn in_sphere(&mut self, r: f32) -> Vec3 {
        loop {
            let x = self.range(-1.0, 1.0);
            let y = self.range(-1.0, 1.0);
            let z = self.range(-1.0, 1.0);
            let v = Vec3::new(x, y, z);
            if v.length_squared() <= 1.0 {
                return v * r;
            }
        }
    }
}

/// the emitter shape: 80 bytes at elParticleSystem +24 (copied from the type's +64 default, `CF::elParticleSystem`
/// 0x002CF2E0): a D3D matrix (glam columns = D3D rows), +64 size x/y/z, +76 kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitterShape {
    pub matrix: Mat4,
    pub size: Vec3,
    /// 0 point, 1 ball, 2 disc (x/z radius size.x, y ± size.y), 3 box (± size)
    pub kind: u32,
}

impl Default for EmitterShape {
    /// elParticleSystem.cpp 0x002D02D0: identity, size 0, point
    fn default() -> Self {
        Self { matrix: Mat4::IDENTITY, size: Vec3::ZERO, kind: 0 }
    }
}

impl EmitterShape {
    /// CarDamageFX.cpp-range 0x00079080: `(kind, matrix, size)`
    pub fn new(kind: u32, matrix: Mat4, size: Vec3) -> Self {
        Self { matrix, size, kind }
    }

    /// 0x00320F30 — a spawn point: 0 = the translation (0x00321000), 1 = translation + ball of radius size.x
    /// (0x00320FC0), 2 = disc x/z · size.x with y in ±size.y (0x00321030), 3 = box ±size (0x00321130); 2 and 3
    /// are transformed by the matrix (D3DXVec3Transform), anything else = 0
    pub fn sample(&self, rng: &mut VsRand) -> Vec3 {
        let t = self.matrix.w_axis.truncate();
        match self.kind {
            0 => t,
            1 => rng.in_sphere(self.size.x) + t,
            2 => {
                let (mut x, mut z);
                loop {
                    x = rng.range(-1.0, 1.0);
                    z = rng.range(-1.0, 1.0);
                    if x * x + z * z <= 1.0 {
                        break;
                    }
                }
                x *= self.size.x;
                z *= self.size.x;
                let y = rng.range(-self.size.y, self.size.y);
                self.matrix.transform_point3(Vec3::new(x, y, z))
            }
            3 => {
                let x = rng.range(-self.size.x, self.size.x);
                let y = rng.range(-self.size.y, self.size.y);
                let z = rng.range(-self.size.z, self.size.z);
                self.matrix.transform_point3(Vec3::new(x, y, z))
            }
            _ => Vec3::ZERO,
        }
    }
}

/// the shared particle buffer (elParticleSystemManager +60 size, +68 `VsPoolAllocator` 0x00320720): `size / (480 +
/// 16)` blocks of 480 bytes
#[derive(Clone, Copy, Debug)]
pub struct BlockBuffer {
    pub blocks: usize,
    pub used: usize,
}

/// block payload (0x002C69A0: `VsPoolAllocator(size, 480)`)
pub const BLOCK_BYTES: usize = 480;

impl BlockBuffer {
    pub fn new(bytes: usize) -> Self {
        Self { blocks: bytes / (BLOCK_BYTES + 16), used: 0 }
    }
}

/// one system's pool (`VsPoolAllocator` 0x00320780, 14 bytes at elParticleSystem +140): a FIFO of blocks {+0 next,
/// +4 u8 first live, +5 u8 count} holding `480 / element size` elements each; the live elements are kept here in
/// block order
#[derive(Clone, Debug)]
pub struct ParticlePool<T> {
    pub items: VecDeque<T>,
    /// (first live, count) per block, oldest first
    pub blocks: VecDeque<(u8, u8)>,
    pub per_block: usize,
}

impl<T> ParticlePool<T> {
    pub fn new(element_bytes: usize) -> Self {
        Self { items: VecDeque::new(), blocks: VecDeque::new(), per_block: (BLOCK_BYTES / element_bytes.max(1)).max(1) }
    }

    /// +10: live elements (0x002C6C30)
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// +12: blocks held (0x002C6D10)
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// 0x00320830: append to the last block while it has room, else take a new block from the buffer (`None` when
    /// it is exhausted)
    pub fn alloc(&mut self, buf: &mut BlockBuffer, v: T) -> Option<usize> {
        match self.blocks.back_mut() {
            Some(b) if (b.1 as usize) < self.per_block => b.1 += 1,
            _ => {
                if buf.used >= buf.blocks {
                    return None;
                }
                buf.used += 1;
                self.blocks.push_back((0, 1));
            }
        }
        self.items.push_back(v);
        Some(self.items.len() - 1)
    }

    /// 0x00320950: free element `i` — the oldest element (front of the first block) is copied into its slot and the
    /// front is dropped; an emptied block goes back to the buffer
    pub fn free(&mut self, buf: &mut BlockBuffer, i: usize) {
        if self.items.is_empty() {
            return;
        }
        self.items.swap(0, i);
        self.items.pop_front();
        if let Some(b) = self.blocks.front_mut() {
            b.0 += 1;
            if b.0 >= b.1 {
                self.blocks.pop_front();
                buf.used -= 1;
            }
        }
    }

    /// 0x00320A40: drop the whole first block
    pub fn free_block(&mut self, buf: &mut BlockBuffer) -> bool {
        let Some((s, c)) = self.blocks.pop_front() else { return false };
        for _ in s..c {
            self.items.pop_front();
        }
        buf.used -= 1;
        true
    }

    pub fn clear(&mut self, buf: &mut BlockBuffer) {
        while self.free_block(buf) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_and_pool() {
        let mut r = VsRand::default();
        // (-24883 * 0xC6BA - 3561) & 0x7FFF
        let first = ((0xC6BAi32).wrapping_mul(-24883) - 3561) & 0x7FFF;
        assert_eq!(r.next(), first);
        for _ in 0..1000 {
            let v = r.range(-2.0, 3.0);
            assert!((-2.0..=3.0).contains(&v));
        }
        let mut buf = BlockBuffer::new(0x32000);
        assert_eq!(buf.blocks, 412);
        let mut p = ParticlePool::new(80);
        assert_eq!(p.per_block, 6);
        for i in 0..13 {
            p.alloc(&mut buf, i).unwrap();
        }
        assert_eq!((p.block_count(), buf.used), (3, 3));
        // freeing element 4 moves the oldest (0) into its slot
        p.free(&mut buf, 4);
        assert_eq!(p.items[3], 0);
        assert_eq!(p.items.len(), 12);
        p.free_block(&mut buf);
        assert_eq!((p.items.len(), p.block_count(), buf.used), (7, 2, 2));
        buf.used = buf.blocks;
        let mut q = ParticlePool::new(80);
        assert!(q.alloc(&mut buf, 0).is_none());
    }
}
