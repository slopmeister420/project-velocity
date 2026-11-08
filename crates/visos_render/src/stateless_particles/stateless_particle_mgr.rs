//! StatelessParticles\StatelessParticleMgr.cpp — the emitter type registry and the pool of 384 emitters
//! (`unk_7A0700`, active list `dword_79ED50`, free stack `dword_79F358`).

use bevy::camera::primitives::{Frustum, Sphere};
use bevy::math::Mat4;

use super::stateless_particle_emitter::{flags, CEmitter, SEED_START};
use super::stateless_particle_render_inst::CRenderInst;
use super::stateless_particle_type::{parse_file, CType};

/// `SLPARTICLEMGR_MAXEMITTERS`
pub const MAX_EMITTERS: usize = 384;
/// returned by `add_emitter` when the pool is full (0x002D4A70 returns 385)
pub const NO_EMITTER: usize = 385;

#[derive(Clone, Debug)]
pub struct CStatelessParticleMgr {
    /// `unk_7A06EC`: registered types in load order
    pub types: Vec<CType>,
    pub emitters: Vec<CEmitter>,
    /// `dword_79ED50` / `dword_79F350`
    pub active: Vec<usize>,
    /// `dword_79F358` / `dword_7A0560`
    free: Vec<usize>,
    /// `dword_6C1DC0`
    seed_source: u32,
}

impl Default for CStatelessParticleMgr {
    fn default() -> Self {
        let mut m = Self { types: Vec::new(), emitters: Vec::new(), active: Vec::new(), free: Vec::new(), seed_source: SEED_START };
        m.reset(true);
        m
    }
}

impl CStatelessParticleMgr {
    /// StatelessParticleMgr.cpp 0x002D2F80: every emitter re-initialised and freed; `clear_types` drops the registered types as well
    pub fn reset(&mut self, clear_types: bool) {
        self.emitters = (0..MAX_EMITTERS).map(|_| CEmitter::init(&mut self.seed_source)).collect();
        self.free = (0..MAX_EMITTERS).collect();
        self.active.clear();
        if clear_types {
            self.types.clear();
        }
    }

    /// 0x002D4470 — register the types of a .slp / PartFX.bin file; `reset` first clears everything (0x002D4370's
    /// second argument)
    pub fn load_emitter_types(&mut self, bytes: &[u8], reset: bool) -> bool {
        if reset {
            self.reset(true);
        }
        match parse_file(bytes) {
            Some((_, t)) => {
                self.types.extend(t);
                true
            }
            None => false,
        }
    }

    /// StatelessParticleMgr.cpp 0x002D4DD0: first type whose name crc matches
    pub fn find_type(&self, crc: u32) -> Option<usize> {
        self.types.iter().position(|t| t.name_crc == crc)
    }

    /// StatelessParticleMgr.cpp 0x002D4A70 — take a free emitter, re-initialise it for the type
    pub fn add_emitter(&mut self, crc: u32) -> usize {
        let Some(ty) = self.find_type(crc) else {
            log::warn!("[CStatelessParticleMgr::AddEmitter] Unregistered emitter type {crc:x}");
            return NO_EMITTER;
        };
        let Some(idx) = self.free.pop() else { return NO_EMITTER };
        self.active.push(idx);
        let mut e = CEmitter::init(&mut self.seed_source);
        e.type_index = ty as i16;
        self.emitters[idx] = e;
        idx
    }

    /// StatelessParticleMgr.cpp 0x002D3240 — one-shot emitter of a type at a matrix (row-vector d3d matrix as glam)
    pub fn spawn(&mut self, crc: u32, matrix: Mat4) -> usize {
        if self.find_type(crc).is_none() {
            log::warn!("ParticleMgr::Spawn - Invalid Emitter type ({crc:08x})");
            return NO_EMITTER;
        }
        let idx = self.add_emitter(crc);
        if idx != NO_EMITTER {
            let e = &mut self.emitters[idx];
            e.matrix = matrix;
            e.set_looping(false);
            e.start();
        }
        idx
    }

    /// `spawn` by type name
    pub fn spawn_named(&mut self, name: &str, matrix: Mat4) -> usize {
        self.spawn(visos::handy::handy_crc::crc32(name.as_bytes(), 0), matrix)
    }

    /// StatelessParticleMgr.cpp 0x002D4CD0: removed on the next update
    pub fn kill_emitter(&mut self, idx: usize) {
        if let Some(e) = self.emitters.get_mut(idx) {
            e.set_kill(true);
        }
    }

    /// StatelessParticleMgr.cpp 0x002D32C0 (`D3::StatelessParticleMgr`) — age every emitter; finished one-shots and killed ones are freed
    pub fn update(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.active.len() {
            let idx = self.active[i];
            let mut remove = false;
            let e = &mut self.emitters[idx];
            e.advance(dt);
            if e.flags & flags::KILL != 0 {
                remove = true;
            } else if e.type_index >= 0 {
                let ty = &self.types[e.type_index as usize];
                let mut total = ty.total_time();
                if e.stop_time > 0.0 {
                    total = ty.life + e.stop_time;
                }
                if e.time >= total {
                    e.reset();
                    if e.flags & flags::LOOPING == 0 {
                        remove = true;
                    }
                }
            }
            if remove {
                self.free.push(idx);
                let last = self.active.pop().unwrap();
                if i < self.active.len() {
                    self.active[i] = last;
                }
            } else {
                i += 1;
            }
        }
    }

    /// StatelessParticleMgr.cpp 0x002D4080 + 0x002D4280 + 0x002D3460 — visible running emitters sorted by (sort key, texture), each with
    /// its copies' parameter blocks. `texture_id` maps a type to its texture's load index.
    pub fn draw_list(&self, frustum: Option<&Frustum>, texture_id: impl Fn(&CType) -> u32) -> Vec<(usize, Vec<CRenderInst>)> {
        let mut list: Vec<(usize, i32)> = Vec::new();
        for &idx in &self.active {
            let e = &self.emitters[idx];
            if e.flags & flags::VISIBLE == 0 || e.flags & flags::RUNNING == 0 || e.type_index < 0 {
                continue;
            }
            let ty = &self.types[e.type_index as usize];
            // 0x002D4080 (cull = 1 from every caller of 0x002D3460): the emitter's bounding sphere against the view
            if let Some(f) = frustum {
                let s = ty.bounding_sphere(e.scale);
                let c = e.matrix.transform_point3(s.truncate());
                if !f.intersects_sphere(&Sphere { center: c.into(), radius: s.w }, true) {
                    continue;
                }
            }
            list.push((idx, ((ty.sort_key << 24) | (texture_id(ty) & 0xFF_FFFF)) as i32));
        }
        // insertion sort, stable
        for i in 1..list.len() {
            let cur = list[i];
            let mut j = i;
            while j > 0 && list[j - 1].1 > cur.1 {
                list[j] = list[j - 1];
                j -= 1;
            }
            list[j] = cur;
        }
        list.into_iter()
            .map(|(idx, _)| {
                let e = &self.emitters[idx];
                let ty = &self.types[e.type_index as usize];
                let (mut alpha, mut size, mut t) = (ty.copy_alpha, ty.copy_size, e.time);
                let mut copies = Vec::new();
                for _ in 0..ty.num_copies {
                    copies.push(CRenderInst::setup(ty, e, t, size, alpha));
                    alpha -= ty.copy_alpha_step;
                    size -= ty.copy_size_step;
                    t -= ty.copy_time_step;
                    if alpha < 0.0 || size < 0.0 || t < 0.0 {
                        break;
                    }
                }
                (idx, copies)
            })
            .collect()
    }

    /// number of seeds drawn so far, for tests
    pub fn seed_source(&self) -> u32 {
        self.seed_source
    }
}

#[cfg(test)]
mod tests {
    use super::super::stateless_particle_emitter::next_seed;
    use super::*;

    #[test]
    fn seed_generator_wraps() {
        let mut s = SEED_START;
        let a = next_seed(&mut s);
        assert_eq!(a, 48271u32.wrapping_mul(SEED_START % 44488).wrapping_sub(3399u32.wrapping_mul(SEED_START / 44488)));
    }

    /// a one-shot skid smoke emitter yields particles, then frees itself after its total time
    #[test]
    fn bounding_sphere_covers_every_type() {
        let Ok(v) = visos::file::Vcd::open("\\Particle\\PARTDEF.VCD") else { return };
        let mut m = CStatelessParticleMgr::default();
        for e in v.entries() {
            if e.name.ends_with(".slp") {
                assert!(m.load_emitter_types(v.data(&e).unwrap(), false));
            }
        }
        assert!(!m.types.is_empty());
        for t in &m.types {
            let s = t.bounding_sphere(1.0);
            assert!(s.is_finite() && s.w >= 1.0, "{} {s:?}", t.name);
        }
    }

    #[test]
    fn skid_smoke_one_shot() {
        let Ok(v) = visos::file::Vcd::open("\\Particle\\PARTDEF.VCD") else { return };
        let mut m = CStatelessParticleMgr::default();
        for e in v.entries() {
            if e.name.ends_with(".slp") {
                assert!(m.load_emitter_types(v.data(&e).unwrap(), false));
            }
        }
        let idx = m.spawn_named("SkidSmoke", Mat4::IDENTITY);
        assert_ne!(idx, NO_EMITTER);
        m.update(0.1);
        let list = m.draw_list(None, |_| 0);
        assert_eq!(list.len(), 1);
        let n: usize = list[0].1.iter().map(|c| c.particles().len()).sum();
        assert!(n > 0, "no particles");
        for _ in 0..40 {
            m.update(0.1);
        }
        assert!(m.active.is_empty());
    }
}
