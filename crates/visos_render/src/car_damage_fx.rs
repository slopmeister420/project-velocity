//! src\CarDamage\CarDamageFX.cpp — the player's impact particles (`Car_impacts_{90|45}_*`, 0x00078880) and the
//! glass particles of broken lights and cracked windows (0x000781C0 / 0x00078270 -> 0x00078320), whose emitters
//! follow the car every rendered frame (0x000784A0, run from CarDamage 0x00076E80 via GameManager 0x000B9010).
//!
//! API: the physics damage events arrive as `CarDamageRenderEvent`s; `CollisionFx { vehicle, point, impulse }`
//! (CarDamage 0x00070F40: impulse already halved) drives the impact particles, `LightBroken` / `WindowCrack` the
//! glass. The object is `byte_6D6808` (`CarDamageFx` resource).

use bevy::prelude::*;

use crate::car_damage_render::{CarDamageRenderEvent, CarDamageRenderKind, CarDamageSource, CarLightDamage};
use crate::car_render::CarRender;
use crate::stateless_particles::stateless_particle_emitter::flags;
use crate::stateless_particles::stateless_particle_mgr::{CStatelessParticleMgr, NO_EMITTER};
use crate::stateless_particles::Particles;

/// `MAX_EMITTERS_PER_VEHICLE`: slots 0..3 windows, 4..7 lights
pub const MAX_EMITTERS_PER_VEHICLE: usize = 8;
/// the glass table holds 6 cars (0x000781C0 `a2 < 6`)
pub const MAX_CARS: usize = 6;

/// `dword_685370`: frames between two impact bursts (counted down per rendered frame in 0x00076E80)
const IMPACT_COOLDOWN_FRAMES: f32 = 5.0;
/// the original counts rendered frames at the TV rate (60 Hz NTSC, see car_render's XBOX_FRAME_RATE)
const XBOX_FRAME_RATE: f32 = 60.0;
/// `dword_685394`: minimum (halved) impulse
const IMPACT_MIN_IMPULSE: f32 = 20.0;
/// `flt_685390` / `flt_68538C`: |cos| of the hit direction against the car's z axis for any / the "90" set
const IMPACT_MIN_DOT: f32 = 0.5;
const IMPACT_90_DOT: f32 = f32::from_bits(0x3F5D_B22D);
/// `dword_685388`: particle count forced on the Plast_rear type
const IMPACT_PLAST_COUNT: u32 = 32;
/// `byte_68537D`: alpha of the paint coloured plastic (before VISOS doubling)
const IMPACT_PLAST_ALPHA: u8 = 0x60;
/// `dword_685380`: paint bytes are divided by this before VISOS doubles them
const IMPACT_PAINT_DIV: u8 = 2;

/// `dword_6D69B8`: window emitter positions (static init 0x00421E60): front, rear, left, right
const WINDOW_POS: [[f32; 3]; 4] = [
    [0.0, f32::from_bits(0x3F48_B439), f32::from_bits(0x3EB3_3333)],
    [0.0, f32::from_bits(0x3F48_B439), -1.0],
    [f32::from_bits(0x3F33_3333), f32::from_bits(0x3F26_6666), f32::from_bits(0xBE4C_CCCD)],
    [f32::from_bits(0xBF33_3333), f32::from_bits(0x3F26_6666), f32::from_bits(0xBE4C_CCCD)],
];
/// `dword_685330`: window emitter rotation (x, y angle)
const WINDOW_ANGLES: [(f32, f32); 4] = [(f32::from_bits(0x3F00_ADFD), std::f32::consts::PI), (1.0, 0.0), (f32::from_bits(0x3E99_999A), -1.0), (f32::from_bits(0x3E99_999A), 1.0)];
/// `dword_685350`: light emitter rotation (x, y angle): left head, right head, left brake, right brake
const LIGHT_ANGLES: [(f32, f32); 4] =
    [(0.0, f32::from_bits(0xC013_3333)), (0.0, f32::from_bits(0x4013_3333)), (0.0, -0.5), (0.0, 0.5)];

/// the CarDamageFX object `byte_6D6808` (only the stateless particle part; the elParticle smoke systems of
/// 0x00076E80 are not ported)
#[derive(Resource, Debug)]
pub struct CarDamageFx {
    /// `dword_6D67BC` in seconds
    cooldown: f32,
    /// +164: per car { car index, 8 emitter slots (NO_EMITTER = -1) }
    slots: [(u64, [usize; MAX_EMITTERS_PER_VEHICLE]); MAX_CARS],
    /// +380: any glass emitter alive
    any: bool,
    /// `flt_6D6CA0`
    angle: f32,
}

impl Default for CarDamageFx {
    fn default() -> Self {
        Self { cooldown: 0.0, slots: [(0, [NO_EMITTER; MAX_EMITTERS_PER_VEHICLE]); MAX_CARS], any: false, angle: 0.0 }
    }
}

/// Renderer.cpp 0x002AB330 — rigid inverse (transposed rotation, -R^T t)
fn rigid_inverse(m: Mat4) -> Mat4 {
    let r = Mat3::from_mat4(m).transpose();
    let t = -(r * m.w_axis.truncate());
    Mat4::from_cols(r.x_axis.extend(0.0), r.y_axis.extend(0.0), r.z_axis.extend(0.0), t.extend(1.0))
}

/// Renderer.cpp 0x002AB7E0 — RotationX(a) * RotationY(b) * RotationZ(c) (d3d order), no translation
pub(crate) fn rotation_xyz(a: f32, b: f32, c: f32) -> Mat4 {
    Mat4::from_rotation_z(c) * Mat4::from_rotation_y(b) * Mat4::from_rotation_x(a)
}

/// VISOS_ColourXBOX 0x002B11C0 bytes (b, g, r, a), each doubled and saturated
pub(crate) fn visos_colour(a: u8, r: u8, g: u8, b: u8) -> [u8; 4] {
    let d = |x: u8| (2 * x as u32).min(255) as u8;
    [d(b), d(g), d(r), d(a)]
}

impl CarDamageFx {
    /// CarDamageFX.cpp 0x00078880 — `GenerateCollisionParticles`: car 0 only, one burst per cooldown. `car` = the
    /// car's world matrix (car data +0), `paint` = D3DCOLOR bytes at car data +2236 (b, g, r, a), `point` = world hit
    /// point, `impulse` = the halved impulse (+40).
    pub fn generate_collision_particles(&mut self, mgr: &mut CStatelessParticleMgr, vehicle: i32, car: Mat4, paint: [u8; 4], point: Vec3, impulse: f32) {
        if vehicle != 0 || self.cooldown > 0.0 {
            return;
        }
        self.cooldown = IMPACT_COOLDOWN_FRAMES / XBOX_FRAME_RATE;
        let local = rigid_inverse(car).transform_point3(point);
        if impulse < IMPACT_MIN_IMPULSE {
            return;
        }
        let d = Vec3::Z.dot(local.normalize_or_zero());
        if d.abs() < IMPACT_MIN_DOT {
            return;
        }
        let set = if d.abs() >= IMPACT_90_DOT { "90" } else { "45" };
        let e: Vec<usize> =
            ["Plast_rear", "smoke", "spots"].iter().map(|n| mgr.add_emitter(visos::handy::handy_crc::crc32(format!("Car_impacts_{set}_{n}").as_bytes(), 0))).collect();
        if e.contains(&NO_EMITTER) {
            // original: the emitters already taken are not given back
            log::warn!("CCarDamageFX::GenerateCollisionParticles - no free emitters");
            return;
        }
        let ty: Vec<usize> = e.iter().map(|&i| mgr.emitters[i].type_index as usize).collect();
        // the shared types are changed for good (dword_6D6CB8)
        mgr.types[ty[0]].num_particles = IMPACT_PLAST_COUNT;
        mgr.types[ty[2]].colour_min = Vec4::new(0.0, 0.0, 0.0, 100.0);
        mgr.types[ty[2]].colour_max = Vec4::new(0.0, 0.0, 0.0, 128.0);
        mgr.types[ty[2]].accel = Vec4::new(0.0, -20.0, 4.0, 1.0);
        // byte_685384: the plastic takes the car's paint
        let (r, g, b) = (paint[2] / IMPACT_PAINT_DIV, paint[1] / IMPACT_PAINT_DIV, paint[0] / IMPACT_PAINT_DIV);
        let end = visos_colour(0, r, g, b);
        let c = visos_colour(IMPACT_PLAST_ALPHA, r, g, b);
        let v = |c: [u8; 4]| Vec4::new(c[2] as f32, c[1] as f32, c[0] as f32, c[3] as f32);
        // StatelessParticleType 0x002D77C0 / 0x002D78D0
        mgr.types[ty[0]].colour_min = v(c);
        mgr.types[ty[0]].colour_max = v(c);
        mgr.types[ty[0]].colour_end = v(end);
        let world = car.transform_point3(local);
        // byte_68537C: turned to face the hit
        self.angle = (0.0 - d).acos();
        if local.x > 0.0 {
            self.angle = 0.0 - self.angle;
        }
        let mut m = car * Mat4::from_rotation_y(self.angle);
        m.w_axis = world.extend(1.0);
        log::debug!(
            "car damage fx: Car_impacts_{set} at {world} (emitters {e:?}, life/duration/window {:?})",
            ty.iter().map(|&t| (mgr.types[t].life, mgr.types[t].duration, mgr.types[t].emit_time)).collect::<Vec<_>>()
        );
        for i in e {
            let em = &mut mgr.emitters[i];
            em.set_looping(false);
            em.start();
            em.matrix = m;
        }
    }

    /// CarDamage.cpp 0x000781C0 (light 0..3: left head, right head, left brake, right brake) / 0x00078270 (window
    /// 0..3: front, rear, left, right) -> CarDamageFX.cpp 0x00078320 `GenerateGlassParticles`. `light_bits` = the
    /// car's +2250 light damage bits before this break. Returns true where the original then plays the
    /// PlayPhysicsImpact glass sound (after the emitter started; the caller sends `GlassFxFired`).
    pub fn generate_glass_particles(&mut self, mgr: &mut CStatelessParticleMgr, car: u64, light: Option<u32>, window: Option<u32>, light_bits: u8) -> bool {
        if car >= MAX_CARS as u64 {
            return false;
        }
        let (name, slot) = match (light, window) {
            (Some(l), _) if l < 4 => {
                if light_bits & (1 << l) != 0 {
                    return false;
                }
                ("Car_Glas_Light", l as usize + 4)
            }
            (_, Some(w)) if w < 4 => (if w > 1 { "Car_Glas_side" } else { "Car_Glas_Front" }, w as usize),
            _ => return false,
        };
        let idx = mgr.add_emitter(visos::handy::handy_crc::crc32(name.as_bytes(), 0));
        if idx == NO_EMITTER {
            log::warn!("CCarDamageFX::GenerateGlassParticles - no free emitters");
            return false;
        }
        let em = &mut mgr.emitters[idx];
        em.set_looping(false);
        em.start();
        log::debug!("car damage fx: {name} on car {car} slot {slot} (emitter {idx})");
        let s = &mut self.slots[car as usize];
        s.1[slot] = idx;
        s.0 = car;
        self.any = true;
        true
    }

    /// CarDamageFX.cpp 0x000784A0 — move every live glass emitter with its car; `car(i)` = (world matrix, head, brake
    /// locators) of the car with that index. Emitters that stopped running drop out.
    pub fn update_glass(&mut self, mgr: &mut CStatelessParticleMgr, car: impl Fn(u64) -> Option<(Mat4, Vec3, Vec3)>) {
        let mut any = false;
        for (index, slots) in self.slots.iter_mut() {
            for (k, slot) in slots.iter_mut().enumerate() {
                if *slot == NO_EMITTER {
                    continue;
                }
                let Some(em) = mgr.emitters.get(*slot) else { continue };
                if em.flags & flags::RUNNING == 0 {
                    *slot = NO_EMITTER;
                    continue;
                }
                any = true;
                let Some((m, head, brake)) = car(*index) else { continue };
                let (mut pos, (a, b)) = if k >= 4 {
                    let l = k - 4;
                    (if l < 2 { head } else { brake }, LIGHT_ANGLES[l])
                } else {
                    (Vec3::from_array(WINDOW_POS[k]), WINDOW_ANGLES[k])
                };
                if k == 5 || k == 7 {
                    pos.x = 0.0 - pos.x;
                }
                let mut w = m * rotation_xyz(a, b, 0.0);
                w.w_axis = m.transform_point3(pos).extend(1.0);
                mgr.emitters[*slot].matrix = w;
            }
        }
        self.any = any;
    }
}

/// CarDamageFX.cpp 0x00078320 tail: a glass emitter of car `request` (the car index `a2`) was started; the original
/// then calls `VsAudio::PlayPhysicsImpact(crc 0x3B5B1A82, volume rand%700+100)` at that car's position
/// (CarDamage 0x000761B0 -> vehicle position). Core forwards it as `AudioCommand::GlassImpact`.
#[derive(Message, Clone, Copy, Debug)]
pub struct GlassFxFired {
    pub request: u64,
}

pub struct CarDamageFxPlugin;

impl Plugin for CarDamageFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GlassFxFired>().init_resource::<CarDamageFx>().add_systems(Update, car_damage_fx
                    .before(crate::car_damage_render::CarDamageRenderSet)
                    .before(crate::stateless_particles::particle_messages)
                    // physics' collision debris (PhysicsManager 0x001A3A30) is spawned before CarDamageFX 0x00079730 runs
                    .after(crate::race_effects::RaceEffectsMessages));
    }
}

fn paint_bytes(spec: &crate::car_render::CarSpec) -> [u8; 4] {
    // car data +2236 = the ShinyCar paint (`byte * 0.003921569` back to the byte)
    let b = |x: f32| (x * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    [b(spec.colour.z), b(spec.colour.y), b(spec.colour.x), 0xFF]
}

/// CarDamage.cpp 0x00076E80's stateless part, each rendered frame: debounce, damage events, glass follow
fn car_damage_fx(
    time: Res<Time>,
    mut fx: ResMut<CarDamageFx>,
    particles: Option<ResMut<Particles>>,
    mut events: MessageReader<CarDamageRenderEvent>,
    mut glass: MessageWriter<GlassFxFired>,
    cars: Query<(&CarRender, &GlobalTransform, Option<&CarDamageSource>, Option<&CarLightDamage>)>,
) {
    let Some(mut particles) = particles else {
        events.clear();
        return;
    };
    let mgr = &mut particles.stateless;
    let find = |r: u64| cars.iter().find(|(c, ..)| c.request == r);
    for ev in events.read() {
        match &ev.kind {
            CarDamageRenderKind::CollisionFx { vehicle, point, impulse } => {
                let Some((c, g, ..)) = find(ev.request) else { continue };
                fx.generate_collision_particles(mgr, *vehicle, g.to_matrix(), paint_bytes(&c.spec), Vec3::from_array(*point), *impulse);
            }
            CarDamageRenderKind::LightBroken(l) => {
                let bits = find(ev.request).and_then(|(.., b)| b).map_or(0, |b| b.0);
                if fx.generate_glass_particles(mgr, ev.request, Some(*l), None, bits) {
                    glass.write(GlassFxFired { request: ev.request });
                }
            }
            // 0x000730C0 crack (not repair); its `byte_6D63A0[0]` gate is taken as set
            CarDamageRenderKind::WindowCrack { window, .. } => {
                if fx.generate_glass_particles(mgr, ev.request, None, Some(*window), 0) {
                    glass.write(GlassFxFired { request: ev.request });
                }
            }
            _ => {}
        }
    }
    if fx.cooldown > 0.0 {
        fx.cooldown = (fx.cooldown - time.delta_secs()).max(0.0);
    }
    if fx.any {
        fx.update_glass(mgr, |r| {
            let (_, g, src, _) = find(r)?;
            let src = src.cloned().unwrap_or_default();
            Some((g.to_matrix(), Vec3::from_array(src.head), Vec3::from_array(src.brake)))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rigid_inverse_matches_inverse() {
        let m = Mat4::from_rotation_translation(Quat::from_rotation_y(0.7) * Quat::from_rotation_x(0.2), Vec3::new(3.0, -2.0, 9.0));
        assert!(rigid_inverse(m).abs_diff_eq(m.inverse(), 1e-5));
    }

    #[test]
    fn rotation_order_is_x_then_y() {
        // d3d Rx(a) * Ry(b): x is rotated first
        let r = rotation_xyz(std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
        assert!(r.transform_vector3(Vec3::Y).abs_diff_eq(Vec3::X, 1e-5));
    }

    #[test]
    fn side_hits_make_no_particles_but_start_the_cooldown() {
        let mut mgr = CStatelessParticleMgr::default();
        let mut fx = CarDamageFx::default();
        fx.generate_collision_particles(&mut mgr, 0, Mat4::IDENTITY, [0; 4], Vec3::new(1.0, 0.0, 0.0), 100.0);
        assert!(fx.cooldown > 0.0);
        assert!(mgr.active.is_empty());
    }
}
