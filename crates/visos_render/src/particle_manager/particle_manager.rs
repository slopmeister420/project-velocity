//! src\ParticleManager\ParticleManager.cpp — the placed emitters of an area (`particle.pld` of the track, plus the
//! race's own), started through the stateless manager. Finite types fire once (`Spawn`), endless ones get a
//! dynamic emitter that keeps running (DynamicParticleEmitter.cpp).

use super::particle_emitter::PldInstance;
use bevy::math::{Mat4, Vec3, Vec4};

use crate::stateless_particles::stateless_particle_mgr::{CStatelessParticleMgr, NO_EMITTER};

#[derive(Clone, Debug, Default)]
pub struct CParticleManager {
    /// +20 / +24 instances: area first, then the track's
    pub instances: Vec<PldInstance>,
    /// +28 dynamic emitters: (instance, stateless emitter)
    pub dynamic: Vec<(usize, usize)>,
}

/// ParticleManager.cpp 0x00100710: 0 no type, 1 finite (spawned once), 2 endless (dynamic emitter)
pub fn spawn_kind(mgr: &CStatelessParticleMgr, crc: u32) -> u32 {
    match mgr.find_type(crc) {
        None => {
            log::warn!("ERROR: CParticleManager::IsEmitterSpawnable - no registered type found!! ({crc:08x})");
            0
        }
        Some(i) if mgr.types[i].is_endless() => 2,
        Some(_) => 1,
    }
}

impl CParticleManager {
    /// ParticleManager.cpp 0x000FF060 / 0x000FF340 / 0x000FF870 — take the loaded instances and start those enabled in the file
    pub fn process(&mut self, mgr: &mut CStatelessParticleMgr, instances: Vec<PldInstance>) {
        self.stop_all(mgr);
        self.instances = instances;
        for i in 0..self.instances.len() {
            if self.instances[i].enabled {
                self.start_instance(mgr, i);
            }
        }
    }

    /// ParticleManager.cpp 0x001007E0
    pub fn start_instance(&mut self, mgr: &mut CStatelessParticleMgr, i: usize) {
        let inst = &mut self.instances[i];
        if inst.manager != 0 {
            return;
        }
        match spawn_kind(mgr, inst.type_crc) {
            1 => {
                inst.enabled = true;
                mgr.spawn(inst.type_crc, inst.matrix());
            }
            2 => {
                // ParticleEmitter.cpp 0x000FDDA0 / 0x000FDFF0: added and started, looping left on
                inst.enabled = true;
                let idx = mgr.add_emitter(inst.type_crc);
                if idx == NO_EMITTER {
                    log::warn!("ERROR: CParticleEmitter::EnableStateless() - no free emitters!");
                    return;
                }
                let e = &mut mgr.emitters[idx];
                e.matrix = inst.matrix();
                e.start();
                self.dynamic.push((i, idx));
            }
            _ => {}
        }
    }

    /// ParticleManager.cpp 0x000FF900 (StartGroup)
    pub fn start_group(&mut self, mgr: &mut CStatelessParticleMgr, group: u32) {
        for i in 0..self.instances.len() {
            if self.instances[i].group == group && !self.instances[i].enabled {
                self.start_instance(mgr, i);
            }
        }
    }

    /// ParticleManager.cpp 0x000FFA70 (StopGroup): instances off, their dynamic emitters killed
    pub fn stop_group(&mut self, mgr: &mut CStatelessParticleMgr, group: u32) {
        for inst in self.instances.iter_mut().filter(|x| x.group == group) {
            inst.enabled = false;
        }
        let instances = &self.instances;
        self.dynamic.retain(|&(i, e)| {
            if instances[i].group == group {
                mgr.kill_emitter(e);
                false
            } else {
                true
            }
        });
    }

    /// ParticleEmitter.cpp 0x000FE400 / 0x000FF3A0: dynamic emitters killed (the stateless pool is reset by its owner)
    pub fn stop_all(&mut self, mgr: &mut CStatelessParticleMgr) {
        for (_, e) in self.dynamic.drain(..) {
            mgr.kill_emitter(e);
        }
    }
}

/// `dword_686564` (1/140): least collision impulse (record +40) for surface debris
pub const MATERIAL_MIN_IMPULSE: f32 = f32::from_bits(0x3BEA_0EA1);
/// `dword_686560` (10/7): least impulse for physics object debris
pub const OBJECT_MIN_IMPULSE: f32 = f32::from_bits(0x3FB6_DB6E);
/// physics object particle kind with the hydrant dynamic groups (0x000FFC30)
pub const OBJECT_KIND_HYDRANT: u32 = 23;

/// static init 0x00423DE0 (`dword_700BA0`): debris emitter pair per `eSURF_*` material of the hit
pub const MATERIAL_DEBRIS: [(Option<&str>, Option<&str>); 26] = {
    const D: (Option<&str>, Option<&str>) = (Some("ColDustGrey"), None);
    const N: (Option<&str>, Option<&str>) = (None, None);
    const G: (Option<&str>, Option<&str>) = (Some("ColGravel"), Some("ColDustGrey"));
    [
        N, N, D, D, D, D, D, D, D, D, D,
        (Some("ColGrass"), Some("ColDustGrey")), G, G,
        D, D, D, D, D, N, N, D, N, N, N, N,
    ]
};

/// static init 0x004239C0 (`dword_700C70`): debris emitter pair per physics object particle kind (object type +52)
pub const OBJECT_DEBRIS: [(Option<&str>, Option<&str>); 27] = {
    const N: (Option<&str>, Option<&str>) = (None, None);
    const FIRE: (Option<&str>, Option<&str>) = (Some("PSXATMsmoke"), Some("AnimFireSmall"));
    [
        N,
        (Some("ColSplintersWood"), None),
        (Some("ColSplintersWood"), Some("ColLeavesAutumnFall")),
        (Some("ColSplintersWhite"), None),
        (Some("ColSplintersGreen"), None),
        (Some("ColSplintersBlue"), None),
        (Some("ColLeavesAutumn"), Some("ColSplintersWood")),
        (Some("ColLeavesRed"), Some("ColLeavesGreen")),
        (Some("ColDustGrey"), None),
        (Some("ColChunksDark"), None),
        (Some("ColChunksWhite"), None),
        N, N, N,
        FIRE,
        (Some("ColGlassPanel"), None),
        (Some("ColFencePlank"), Some("ColSplintersWood")),
        N, N, N, N, N, N,
        FIRE,
        N, N, N,
    ]
};

/// ParticleManager.cpp 0x00100010 (SpawnParticle): a one-shot of a registered type at a point, identity axes (0x002D3210)
pub fn spawn_at(mgr: &mut CStatelessParticleMgr, name: &str, point: Vec3) {
    let crc = visos::handy::handy_crc::crc32(name.as_bytes(), 0);
    if mgr.find_type(crc).is_none() {
        log::warn!("ERROR: CParticleManager::SpawnParticle - no registered type found! ({name})");
        return;
    }
    mgr.spawn(crc, Mat4::from_translation(point));
}

impl CParticleManager {
    /// ParticleManager.cpp 0x001005C0 — surface debris for a car collision (PhysicsManager 0x001A3A30 / 0x001A7060, with the record
    /// that becomes the collision message): `material` = record +76, `point` = +8, `impulse` = +40
    pub fn collision_particles(&self, mgr: &mut CStatelessParticleMgr, material: u8, point: Vec3, impulse: f32) {
        if mgr.types.is_empty() || MATERIAL_MIN_IMPULSE > impulse {
            return;
        }
        if let Some((a, b)) = MATERIAL_DEBRIS.get(material as usize) {
            for n in [a, b].into_iter().flatten() {
                spawn_at(mgr, n, point);
            }
        }
    }

    /// ParticleManager.cpp 0x000FFC30 — debris for a hit physics object (PhysicsObjectManager 0x001E1F80): `kind` = object type +52,
    /// `object` = its render matrix. The hydrant kind starts dynamic groups (0x00100B40, not ported).
    pub fn object_collision_particles(&self, mgr: &mut CStatelessParticleMgr, kind: u32, object: Mat4, point: Vec3, impulse: f32) {
        if mgr.types.is_empty() || OBJECT_MIN_IMPULSE > impulse {
            return;
        }
        if kind == OBJECT_KIND_HYDRANT {
            log::warn!("hydrant dynamic groups (0x00100B40) not ported");
            return;
        }
        if let Some((a, b)) = OBJECT_DEBRIS.get(kind as usize) {
            for n in [a, b].into_iter().flatten() {
                self.object_particle(mgr, n, object, point);
            }
        }
    }

    /// ParticleManager.cpp 0x000FFEA0 — at the hit point, or at the first placed instance of that type carried by the object's matrix
    /// (any instance of the type, not only the object's own: kept)
    fn object_particle(&self, mgr: &mut CStatelessParticleMgr, name: &str, object: Mat4, point: Vec3) {
        let crc = visos::handy::handy_crc::crc32(name.as_bytes(), 0);
        let mut pos = point;
        if let Some(i) = self.instances.iter().find(|i| i.type_crc == crc) {
            pos = object.transform_vector3(i.position) + object.w_axis.truncate();
        }
        spawn_at(mgr, name, pos);
    }
}

/// ParticleManager.cpp 0x001000E0 (FireOffParticleTrail): event 24 "propane_trail", 26 "wheel_fire" + "wheel_smoke", one-shots every
/// metre (the first after half a metre, at most 201) from `from` to `to` and one half a metre before the end, axes
/// right / up / along the path (PhysicsObjectParticleTracker 0x001E4C30)
pub fn fire_off_particle_trail(mgr: &mut CStatelessParticleMgr, from: Vec3, to: Vec3, event: u32) {
    let (a, b) = match event {
        24 => ("propane_trail", None),
        26 => ("wheel_fire", Some("wheel_smoke")),
        _ => return,
    };
    let step = 1.0f32;
    let half = step * 0.5;
    let dir = (to - from).normalize_or_zero();
    let right = dir.cross(Vec3::Y).normalize_or_zero();
    let up = right.cross(dir).normalize_or_zero();
    let mut m = Mat4::from_cols(right.extend(0.0), up.extend(0.0), dir.extend(0.0), Vec4::W);
    let len = from.distance(to);
    if step > len {
        return;
    }
    let mut spawn = |m: &Mat4| {
        mgr.spawn_named(a, *m);
        if let Some(b) = b {
            mgr.spawn_named(b, *m);
        }
    };
    let (mut d, mut count, mut first) = (0.0f32, 0u32, true);
    loop {
        if d + step > len {
            break;
        }
        if first {
            d += half;
            first = false;
        } else {
            d += step;
        }
        m.w_axis = lerp4(from, to, d / len).extend(1.0);
        spawn(&m);
        count += 1;
        if count > 200 {
            break;
        }
    }
    m.w_axis = lerp4(from, to, (len - half) / len).extend(1.0);
    spawn(&m);
}

/// CarDamage.cpp 0x000762F0: (b - a) * t + a per component
fn lerp4(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    (b - a) * t + a
}

#[cfg(test)]
mod tests {
    use super::*;

    /// which debris types the shipped defs register (missing ones only warn, as in the original)
    #[test]
    fn debris_types_registered() {
        let mut p = crate::stateless_particles::Particles::default();
        p.load("\\Tracks\\LACoast\\", None);
        let names: std::collections::BTreeSet<&str> =
            MATERIAL_DEBRIS.iter().chain(OBJECT_DEBRIS.iter()).flat_map(|(a, b)| [*a, *b]).flatten().collect();
        let found = names.iter().filter(|n| p.stateless.find_type(visos::handy::handy_crc::crc32(n.as_bytes(), 0)).is_some()).count();
        assert!(found > 0, "no debris type registered");
        let q = p.placed.clone();
        q.collision_particles(&mut p.stateless, 11, Vec3::ZERO, 1.0);
        fire_off_particle_trail(&mut p.stateless, Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), 26);
    }
}
