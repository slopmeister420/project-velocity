//! Render\skidsmoke.cpp (0x00139BD0 / 0x00139D50) — tyre smoke: every sliding wheel fires a one-shot stateless
//! emitter chosen by the surface under it, rate limited per wheel. A front wheel on a flame mark fires
//! "TrailBlaze" instead.

use bevy::math::{Mat4, Vec3, Vec4};

use crate::race_effects::{EffectCar, EffectCars};
use crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr;

/// `flt_730A4C` (static init 0x00425ED0: 2.0 * 0.5): smoke moved back along the direction of travel
pub const BACK_OFFSET: f32 = 1.0;
/// delay between puffs (0x3CA3D70A), and at walking pace on a flame mark (0x3D4CCCCD)
pub const PUFF_DELAY: f32 = f32::from_bits(0x3CA3_D70A);
pub const SLOW_FLAME_DELAY: f32 = f32::from_bits(0x3D4C_CCCD);

/// skidsmoke.cpp 0x0013A300: road surfaces where a slow wheel still smokes when spinning
fn smokes_when_spinning(surface: u8) -> bool {
    surface == 12 || surface == 13
}

/// smoke state of the cars (VehicleEntity +5024: per wheel time to the next puff)
#[derive(Clone, Debug, Default)]
pub struct SkidSmoke {
    pub next_puff: Vec<[f32; 4]>,
}

impl SkidSmoke {
    /// skidsmoke.cpp 0x00139BD0 (GameManager::InGameUpdate) — `flames[car][wheel]` = the wheel's start line flame timer
    pub fn update(&mut self, cars: &EffectCars, flames: &[[f32; 4]], mgr: &mut CStatelessParticleMgr) {
        if self.next_puff.len() < cars.cars.len() {
            self.next_puff.resize(cars.cars.len(), [0.0; 4]);
        }
        for (i, car) in cars.cars.iter().enumerate() {
            let f = flames.get(i).copied().unwrap_or([0.0; 4]);
            self.update_car(i, car, cars.dt, f, mgr);
        }
    }

    /// skidsmoke.cpp 0x00139D50
    fn update_car(&mut self, idx: usize, car: &EffectCar, dt: f32, flames: [f32; 4], mgr: &mut CStatelessParticleMgr) {
        if car.smoke_off {
            return;
        }
        for w in 0..4 {
            let mut delay = PUFF_DELAY;
            let flame = flames[w] > 0.5;
            if flame && car.speed <= 1.0 {
                delay = SLOW_FLAME_DELAY;
            }
            self.next_puff[idx][w] -= dt;
            let wh = &car.wheels[w];
            if wh.no_marks {
                continue;
            }
            let fire = self.next_puff[idx][w] <= 0.0
                && (flame || wh.sliding || (smokes_when_spinning(wh.surface) && car.speed > 2.0 && wh.contacts > 0));
            if !fire {
                continue;
            }
            let mut pos = Vec3::new(wh.centre.x, wh.centre.y - wh.radius * 0.5, wh.centre.z);
            let dir = if car.speed <= 1.0 { car.heading } else { car.velocity };
            let dir = dir.normalize_or_zero();
            if car.speed > 5.0 {
                pos -= dir * BACK_OFFSET;
            }
            let up = Vec3::Y;
            // VsAITools 0x000293D0: x = dir x up
            let x = dir.cross(up);
            let m = Mat4::from_cols(x.extend(0.0), up.extend(0.0), dir.extend(0.0), Vec4::new(pos.x, pos.y, pos.z, 1.0));
            let name = if flame {
                "TrailBlaze"
            } else {
                match wh.surface.wrapping_sub(11) {
                    0 => "GrassSmoke",
                    1 | 2 => "GravelSmoke",
                    _ if car.speed >= 5.0 => "SkidSmoke",
                    _ => "SkidSmoke_stop",
                }
            };
            mgr.spawn_named(name, m);
            self.next_puff[idx][w] = delay;
        }
    }
}
