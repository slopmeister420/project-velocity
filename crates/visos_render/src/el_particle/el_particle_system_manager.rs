//! elParticleSystemManager.cpp (0x002C5A60..0x002CECC0; most of its 398 functions are the std::map / list /
//! smart-pointer templates of its two containers, replaced by Vecs here): the registered types (+16), the systems
//! (+28, a map keyed by the system pointer → {keep, removed}), the shared particle buffer (+60 size set by
//! VsCommonMain 0x0016E11D to 0x32000, +64 users, +68 pool), tick 0x002C5B70 and render 0x002C6660 (both from
//! GameManager 0x000B9010 once per rendered frame). The second system container (+40, 0x00319xxx systems) has no
//! user in the game code and is not ported; the map's pointer order is replaced by insertion order.

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use super::el_particle_renderer::{draw_system, CameraBasis, TrailState};
use super::el_particle_system::{ElParticleSystem, ElParticleType, Particle, Sacrifice, SystemCtx};
use super::vs_pool_allocator::{BlockBuffer, ParticlePool, VsRand};
use crate::effect_draw::EffectDraw;

/// VsCommonMain 0x0016E111: `elParticleSystemManager::SetBufferSize(0x32000)`
pub const BUFFER_BYTES: usize = 0x32000;

/// a type the manager knows: listed (0x002C5D20 add / 0x002C5DC0 remove) and referenced by systems
#[derive(Clone, Debug)]
pub struct TypeSlot {
    pub ty: ElParticleType,
    pub listed: bool,
    pub refs: u32,
}

/// a system map entry: +4 removed by the manager once finished (`add(system, keep)`), +5 removed once empty
#[derive(Clone, Debug)]
pub struct SystemEntry {
    pub id: u64,
    pub sys: ElParticleSystem,
    pub auto_remove: bool,
    pub remove_when_empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(pub u64);

#[derive(Resource, Debug)]
pub struct ElParticleSystemManager {
    pub types: Vec<Option<TypeSlot>>,
    pub systems: Vec<SystemEntry>,
    pub buffer: BlockBuffer,
    /// `dword_6C2430` generator and the CRT `rand` (frame jumps)
    pub rng: VsRand,
    pub crt: CrtRand,
    /// the trail renderer's statics (`dword_7B44C8` ..)
    pub trail: TrailState,
    next_id: u64,
}

impl Default for ElParticleSystemManager {
    fn default() -> Self {
        Self {
            types: Vec::new(),
            systems: Vec::new(),
            buffer: BlockBuffer::new(BUFFER_BYTES),
            rng: VsRand::default(),
            crt: CrtRand::default(),
            trail: TrailState::default(),
            next_id: 1,
        }
    }
}

/// 0x002C6840 seen from the system at `before.len()`: the first system (in order) with the most blocks loses its
/// first block
struct Others<'a> {
    before: &'a mut [SystemEntry],
    after: &'a mut [SystemEntry],
}

impl Sacrifice for Others<'_> {
    fn sacrifice(&mut self, own: &mut ParticlePool<Particle>, buf: &mut BlockBuffer) {
        #[derive(Clone, Copy)]
        enum V {
            None,
            Before(usize),
            Own,
            After(usize),
        }
        let mut best = 0;
        let mut v = V::None;
        for (i, e) in self.before.iter().enumerate() {
            if e.sys.pool.block_count() > best {
                best = e.sys.pool.block_count();
                v = V::Before(i);
            }
        }
        if own.block_count() > best {
            best = own.block_count();
            v = V::Own;
        }
        for (i, e) in self.after.iter().enumerate() {
            if e.sys.pool.block_count() > best {
                best = e.sys.pool.block_count();
                v = V::After(i);
            }
        }
        // 0x002D01E0
        match v {
            V::None => {}
            V::Before(i) => {
                self.before[i].sys.pool.free_block(buf);
            }
            V::Own => {
                own.free_block(buf);
            }
            V::After(i) => {
                self.after[i].sys.pool.free_block(buf);
            }
        }
    }
}

impl ElParticleSystemManager {
    /// elParticleSystemManager.cpp 0x002C5D20 `add(elParticleType)`: refused (false) when a listed type has the same name crc (0x002C5E10); the
    /// type is kept for the caller's handle either way
    pub fn add_type(&mut self, ty: ElParticleType) -> (TypeId, bool) {
        let dup = self.types.iter().flatten().any(|s| s.listed && s.ty.data.crc == ty.data.crc);
        let slot = TypeSlot { ty, listed: !dup, refs: 0 };
        let i = match self.types.iter().position(Option::is_none) {
            Some(i) => {
                self.types[i] = Some(slot);
                i
            }
            None => {
                self.types.push(Some(slot));
                self.types.len() - 1
            }
        };
        (TypeId(i), !dup)
    }

    /// elParticleSystemManager.cpp 0x002C5DC0 `remove(elParticleType)` (the caller's reference goes with it)
    pub fn remove_type(&mut self, id: TypeId) {
        if let Some(Some(s)) = self.types.get_mut(id.0) {
            s.listed = false;
            if s.refs == 0 {
                self.types[id.0] = None;
            }
        }
    }

    pub fn type_ref(&self, id: TypeId) -> Option<&ElParticleType> {
        self.types.get(id.0)?.as_ref().map(|s| &s.ty)
    }

    pub fn type_mut(&mut self, id: TypeId) -> Option<&mut ElParticleType> {
        self.types.get_mut(id.0)?.as_mut().map(|s| &mut s.ty)
    }

    /// 0x002CECC0 `new elParticleSystem(name, type)` + 0x002C6380 `add(system, keep)`
    pub fn add_system(&mut self, name: &str, ty: TypeId, auto_remove: bool) -> Option<SystemId> {
        let slot = self.types.get_mut(ty.0)?.as_mut()?;
        slot.refs += 1;
        let sys = ElParticleSystem::new(name, ty.0, &slot.ty);
        let id = self.next_id;
        self.next_id += 1;
        self.systems.push(SystemEntry { id, sys, auto_remove, remove_when_empty: false });
        Some(SystemId(id))
    }

    fn index(&self, id: SystemId) -> Option<usize> {
        self.systems.iter().position(|e| e.id == id.0)
    }

    pub fn system(&self, id: SystemId) -> Option<&ElParticleSystem> {
        self.index(id).map(|i| &self.systems[i].sys)
    }

    pub fn system_mut(&mut self, id: SystemId) -> Option<&mut ElParticleSystem> {
        self.index(id).map(|i| &mut self.systems[i].sys)
    }

    /// elParticleSystemManager.cpp 0x002C63E0 `remove(system, immediate)`: immediate = erase now, else stop it and erase it once empty
    pub fn remove_system(&mut self, id: SystemId, immediate: bool) {
        let Some(i) = self.index(id) else {
            log::warn!("Warning: elParticleSystemManager::remove: no such particle system.");
            return;
        };
        if immediate {
            self.erase(i);
        } else {
            self.systems[i].sys.stop();
            self.systems[i].remove_when_empty = true;
        }
    }

    fn erase(&mut self, i: usize) {
        let mut e = self.systems.remove(i);
        e.sys.pool.clear(&mut self.buffer);
        let t = e.sys.ty;
        if let Some(Some(s)) = self.types.get_mut(t) {
            s.refs = s.refs.saturating_sub(1);
            if !s.listed && s.refs == 0 {
                self.types[t] = None;
            }
        }
    }

    /// run `f` on system `i` with everything an emission needs
    fn with_system(&mut self, i: usize, f: impl FnOnce(&mut ElParticleSystem, &mut SystemCtx, &mut BlockBuffer)) {
        let Self { types, systems, buffer, rng, crt, .. } = self;
        let (before, rest) = systems.split_at_mut(i);
        let Some((cur, after)) = rest.split_first_mut() else { return };
        let Some(Some(slot)) = types.get(cur.sys.ty) else { return };
        let mut others = Others { before, after };
        let mut ctx = SystemCtx { ty: &slot.ty, rng, crt, others: &mut others };
        f(&mut cur.sys, &mut ctx, buffer);
    }

    /// 0x002CF100 `elParticleSystem::start`
    pub fn start(&mut self, id: SystemId) {
        if let Some(i) = self.index(id) {
            self.with_system(i, |s, ctx, buf| s.start(ctx, buf));
        }
    }

    /// 0x002CF170 `elParticleSystem::stop`
    pub fn stop(&mut self, id: SystemId) {
        if let Some(s) = self.system_mut(id) {
            s.stop();
        }
    }

    /// elParticleSystemManager.cpp 0x002C5B70 `tick(dt)`: every system ticks; then it is erased when removed and empty, or when kept-for-
    /// removal and finished
    pub fn tick(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.systems.len() {
            self.with_system(i, |s, ctx, buf| s.tick(ctx, buf, dt));
            let e = &self.systems[i];
            let finished = self.types.get(e.sys.ty).and_then(|s| s.as_ref()).is_some_and(|s| e.sys.finished(&s.ty));
            if (e.remove_when_empty && e.sys.pool.is_empty()) || (e.auto_remove && finished) {
                self.erase(i);
            } else {
                i += 1;
            }
        }
    }

    /// elParticleSystemManager.cpp 0x002C6660 `render`: every system through the Xbox renderer (0x00317A00)
    pub fn render(&mut self, cam: &CameraBasis, draw: &mut EffectDraw) {
        let Self { types, systems, trail, .. } = self;
        for e in systems.iter() {
            if let Some(Some(slot)) = types.get(e.sys.ty) {
                draw_system(&e.sys, &slot.ty, cam, trail, draw);
            }
        }
    }

    /// everything gone (race shutdown)
    pub fn clear(&mut self) {
        *self = Self { rng: self.rng, ..Self::default() };
    }

    /// particles alive over all systems
    pub fn particle_count(&self) -> usize {
        self.systems.iter().map(|e| e.sys.pool.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::el_particle::el_particle_control_curve::ControlCurve;

    fn ty(name: &str, rate: f32) -> ElParticleType {
        let mut t = ElParticleType::new(150, 1, 0, name);
        t.set_emission_rate(ControlCurve::constant(rate), 0.3);
        t.set_lifetime(4.0);
        t
    }

    #[test]
    fn manager_lifecycle_and_sacrifice() {
        let mut m = ElParticleSystemManager::default();
        let (a, listed) = m.add_type(ty("a", 60.0));
        assert!(listed);
        assert!(!m.add_type(ty("a", 1.0)).1);
        let (b, _) = m.add_type(ty("b", 60.0));
        let sa = m.add_system("a", a, false).unwrap();
        let sb = m.add_system("b", b, false).unwrap();
        m.start(sa);
        m.start(sb);
        // a small buffer: 10 blocks of 6
        m.buffer = BlockBuffer::new(10 * 496);
        for _ in 0..120 {
            m.tick(1.0 / 60.0);
        }
        assert!(m.buffer.used <= 10);
        assert!(m.particle_count() <= 60 && m.particle_count() > 40);
        // stop + remove once empty
        m.remove_system(sa, false);
        for _ in 0..300 {
            m.tick(1.0 / 60.0);
        }
        assert!(m.system(sa).is_none() && m.system(sb).is_some());
        m.remove_type(a);
        assert!(m.type_ref(a).is_none());
        m.remove_type(b);
        assert!(m.type_ref(b).is_some(), "kept while its system lives");
        m.remove_system(sb, true);
        assert!(m.type_ref(b).is_none());
        assert_eq!(m.buffer.used, 0);
    }
}
