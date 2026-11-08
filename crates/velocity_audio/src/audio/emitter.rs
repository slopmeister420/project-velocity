//! emitters and listeners: AUDIO_CreateEmitter/DestroyEmitter, AUDIO_SetEmitterDirection.cpp (and its
//! position/velocity/volume siblings), AUDIO_SetListenerDirection.cpp, AUDIO_SetListenerVelocity.cpp and
//! the per-frame `_AUDIO_UpdateEmitter3D` / `_AUDIO_UpdateListener`.

use bevy::math::Vec3;

use super::system::*;

use emitter_flags as efl;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

impl AudioSystem {
    /// AUDIO_CreateEmitter.cpp 0x002B9DF0 (pool init `_AUDIO_AllocEmitter` 0x00312170)
    pub fn create_emitter(&mut self, flags: u32) -> Result<EmitterId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let e = Emitter {
            flags,
            samples: None,
            volume: 1.0,
            distance_gain: 1.0,
            position: Vec3::ZERO,
            front: Vec3::Z,
            top: Vec3::Y,
            velocity: Vec3::ZERO,
            min_distance: self.default_min_distance,
            max_distance: self.default_max_distance,
            distance: 0.0,
            angle: 0.0,
            doppler: self.doppler_factor,
        };
        let Some(id) = self.emitters.alloc(e) else { return Err(err::NO_EMITTER) };
        self.emitter_list.insert(0, id);
        if flags & efl::ALIVE != 0 {
            self.emitters.get_mut(id).unwrap().flags |= efl::VOLUME_DIRTY;
        }
        Ok(id)
    }

    /// AUDIO_DestroyEmitter.cpp 0x002BCCD0
    pub fn destroy_emitter(&mut self, e: EmitterId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if !self.emitters.contains(e) {
            return err::BAD_EMITTER;
        }
        let r = self.stop_emitter_samples(e);
        if r != 0 {
            return r;
        }
        if let Some(p) = self.emitter_list.iter().position(|&x| x == e) {
            self.emitter_list.remove(p);
        }
        self.emitters.free(e);
        0
    }

    /// AUDIO 0x00304F30: stop the active events of `hash` on emitter `e` (the first failing stop returns).
    pub fn stop_emitter_event(&mut self, e: EmitterId, hash: u32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let mut cur = self.event_head;
        while let Some(id) = cur {
            let Some(ev) = self.events.get(id) else { break };
            cur = ev.next;
            let h = self.groups.get(ev.event.group).and_then(|g| g.events.get(ev.event.event)).map(|g| g.hash);
            if ev.emitter == e && h == Some(hash) {
                let r = self.stop_event(Some(id));
                if r != 0 {
                    return r;
                }
            }
        }
        0
    }

    /// AUDIO_StopEmitterSamples.cpp 0x002BCF30: stop until the list empties (a failing stop moves on).
    pub fn stop_emitter_samples(&mut self, e: EmitterId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get(e) else { return err::BAD_EMITTER };
        let mut r = 0;
        let mut cur = em.samples;
        while let Some(s) = cur {
            r = self.stop_sample(s);
            cur = if r != 0 {
                self.samples.get(s).and_then(|p| p.next)
            } else {
                self.emitters.get(e).and_then(|x| x.samples)
            };
        }
        if let Some(em) = self.emitters.get_mut(e) {
            em.flags &= !efl::HAS_SAMPLES;
        }
        r
    }

    /// AUDIO_StopAllEmitterSamples.cpp 0x002BCE00
    pub fn stop_all_emitter_samples(&mut self) -> i32 {
        if !self.initialised {
            return not_init();
        }
        for e in self.emitter_list.clone() {
            if self.emitters.get(e).is_some_and(|x| x.flags & efl::ALIVE != 0) {
                let r = self.stop_emitter_samples(e);
                if r != 0 {
                    return r;
                }
            }
        }
        0
    }

    /// AUDIO_SetEmitterPosition.cpp 0x002BE8C0 (non-finite input resets to the origin)
    pub fn set_emitter_position(&mut self, e: EmitterId, p: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        if em.flags & efl::FLAT != 0 && em.flags & efl::PAN == 0 {
            return 0x0900_0004;
        }
        em.position = if finite(p) { p } else { Vec3::ZERO };
        em.flags |= efl::POSITION_DIRTY;
        0
    }

    /// AUDIO_SetEmitterVelocity.cpp 0x002BE7B0
    pub fn set_emitter_velocity(&mut self, e: EmitterId, v: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        if em.flags & efl::FLAT != 0 {
            return 0x0900_0004;
        }
        em.velocity = if finite(v) { v } else { Vec3::ZERO };
        em.flags |= efl::POSITION_DIRTY;
        0
    }

    /// AUDIO_SetEmitterDirection.cpp 0x002BE5B0
    pub fn set_emitter_direction(&mut self, e: EmitterId, front: Vec3, top: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        if em.flags & efl::FLAT != 0 {
            return 0x0900_0004;
        }
        debug_assert!(finite(front) && finite(top), "0");
        em.top = top.normalize_or_zero();
        em.front = front.normalize_or_zero();
        em.flags |= efl::POSITION_DIRTY;
        0
    }

    /// AUDIO_SetEmitterVolume.cpp 0x002BCE70
    pub fn set_emitter_volume(&mut self, e: EmitterId, v: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        if !(0.0..=1.0).contains(&v) {
            return err::BAD_PARAM;
        }
        em.volume = v;
        em.flags |= efl::PITCH_DIRTY;
        0
    }

    /// AUDIO_SetEmitterDistances.cpp 0x002BE540
    pub fn set_emitter_distances(&mut self, e: EmitterId, min: f32, max: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        em.min_distance = min;
        em.max_distance = max;
        0
    }

    /// AUDIO_SetEmitterFlags.cpp 0x002BFE90
    pub fn set_emitter_flags(&mut self, e: EmitterId, flags: u32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(em) = self.emitters.get_mut(e) else { return err::BAD_EMITTER };
        let changed = flags ^ em.flags;
        em.flags = flags | (em.flags & 0x0F00_0000);
        if changed & efl::ALIVE != 0 {
            em.flags |= efl::VOLUME_DIRTY;
        }
        if changed & 7 != 0 {
            em.flags |= 0x40_0000;
        }
        0
    }

    /// AUDIO_CreateListener.cpp 0x002B9F70
    pub fn create_listener(&mut self, flags: u32) -> Result<ListenerId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let l = Listener {
            flags,
            position: Vec3::ZERO,
            front: Vec3::Z,
            top: Vec3::Y,
            velocity: Vec3::ZERO,
            front_x: 0.0,
            front_z: 1.0,
        };
        let Some(id) = self.listeners.alloc(l) else { return Err(err::NO_LISTENER) };
        self.listener_list.insert(0, id);
        if flags & efl::ALIVE != 0 {
            self.listeners.get_mut(id).unwrap().flags |= efl::VOLUME_DIRTY;
        }
        Ok(id)
    }

    /// AUDIO_DestroyListener.cpp 0x002BC840
    pub fn destroy_listener(&mut self, l: ListenerId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if !self.listeners.contains(l) {
            return err::BAD_LISTENER;
        }
        if let Some(p) = self.listener_list.iter().position(|&x| x == l) {
            self.listener_list.remove(p);
        }
        self.listeners.free(l);
        0
    }

    /// AUDIO_SetListenerPosition.cpp 0x002BEFE0
    pub fn set_listener_position(&mut self, l: ListenerId, p: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(li) = self.listeners.get_mut(l) else { return err::BAD_LISTENER };
        li.position = if finite(p) { p } else { Vec3::ZERO };
        li.flags |= efl::POSITION_DIRTY;
        0
    }

    /// AUDIO_SetListenerVelocity.cpp 0x002BEEC0
    pub fn set_listener_velocity(&mut self, l: ListenerId, v: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(li) = self.listeners.get_mut(l) else { return err::BAD_LISTENER };
        debug_assert!(finite(v), "0");
        li.velocity = v;
        li.flags |= efl::POSITION_DIRTY;
        0
    }

    /// AUDIO_SetListenerDirection.cpp 0x002BEC60
    pub fn set_listener_direction(&mut self, l: ListenerId, front: Vec3, top: Vec3) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(li) = self.listeners.get_mut(l) else { return err::BAD_LISTENER };
        debug_assert!(finite(front) && finite(top), "0");
        li.top = top.normalize_or_zero();
        li.front = front.normalize_or_zero();
        let mut len = (li.front.x * li.front.x + li.front.z * li.front.z).sqrt();
        if len < f32::from_bits(0x358637BD) {
            len = f32::from_bits(0x358637BD); // 0.000001
        }
        li.front_x = li.front.x / len;
        li.front_z = li.front.z / len;
        li.flags |= efl::POSITION_DIRTY;
        0
    }

    /// _AUDIO_UpdateListener.cpp 0x00315870
    pub(crate) fn update_listener(&mut self, l: ListenerId) -> i32 {
        let Some(li) = self.listeners.get(l).cloned() else { return 0 };
        self.voices.listener_add(li.flags, li.position, li.front, li.top, li.velocity);
        if li.flags & efl::ALIVE == 0 {
            return 0;
        }
        self.listeners.get_mut(l).unwrap().flags &= 0xFF0F_FFFF;
        0
    }

    /// _AUDIO_UpdateEmitter3D.cpp 0x00315180: distance, linear min/max attenuation, listener-relative angle
    /// and doppler cents against the nearest live listener (consumed by the software mix).
    pub(crate) fn update_emitter_3d(&mut self, e: EmitterId) {
        let Some(em) = self.emitters.get(e).cloned() else { return };
        if self.listener_list.is_empty() || em.flags & efl::FLAT != 0 {
            let em = self.emitters.get_mut(e).unwrap();
            em.distance = f32::from_bits(0x3C23_D70A); // 0.01
            em.distance_gain = 1.0;
            if em.flags & efl::PAN == 0 {
                em.angle = 0.0;
            }
            return;
        }
        let mut best: Option<Listener> = None;
        let mut best_d2 = 1.0e20f32;
        let mut rel = Vec3::ZERO;
        for &l in &self.listener_list {
            let li = self.listeners.get(l).unwrap();
            if li.flags & efl::ALIVE != 0 {
                let d = em.position - li.position;
                let d2 = d.length_squared();
                if d2 < best_d2 {
                    rel = d;
                    best_d2 = d2;
                    best = Some(li.clone());
                }
            }
        }
        let Some(li) = best else {
            let em = self.emitters.get_mut(e).unwrap();
            em.angle = 0.0;
            em.distance = f32::from_bits(0x3C23_D70A);
            em.distance_gain = 1.0;
            return;
        };
        let dist = best_d2.sqrt();
        let (c, f) = (self.speed_of_sound, self.doppler_factor);
        let em_m = self.emitters.get_mut(e).unwrap();
        em_m.distance = dist;
        if dist >= em.min_distance {
            if dist <= em.max_distance {
                em_m.distance_gain = 1.0 - (dist - em.min_distance) / (em.max_distance - em.min_distance);
                em_m.flags |= efl::PITCH_DIRTY;
            } else {
                em_m.distance_gain = 0.0;
            }
        } else {
            em_m.distance_gain = 1.0;
        }
        if em.flags & efl::MOVING != 0 {
            let v12 = li.position - em.position;
            let len = v12.length();
            let (mut v13, mut v10) = (0.0f32, 0.0f32);
            if len > 0.0 {
                v13 = v12.dot(li.velocity) / len;
                v10 = v12.dot(em.velocity) / len;
            }
            let lim = c / f;
            v13 = v13.min(lim);
            v10 = v10.min(lim);
            let ratio = (c - f * v13) / (c - f * v10);
            em_m.doppler = 1200.0 / 2f32.log10() * ratio.log10();
        }
        // angle of the emitter around the listener, in the listener's horizontal frame
        let (x, z) = (rel.x, rel.z);
        let rx = x * li.front_z - z * li.front_x;
        let rz = x * li.front_x + z * li.front_z;
        em_m.angle = if rz.abs() >= f32::from_bits(0x358637BD) {
            rx.atan2(rz)
        } else if rx.abs() >= f32::from_bits(0x358637BD) {
            if rx <= 0.0 { f32::from_bits(0xBFC90FDB) } else { f32::from_bits(0x3FC90FDB) }
        } else {
            0.0
        };
    }
}
