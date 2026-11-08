//! elParticle — the stateful particle framework (`.\elParticle\*.cpp`): parameter sets (`elParticleType`) built in
//! code or loaded from SFP files (none ship: the PARTDEF packs hold only `.slp`), emitter instances
//! (`elParticleSystem`) whose particles carry real state (position, velocity, trail nodes), the manager that ticks
//! and draws them once per rendered frame (GameManager 0x000B9010: tyre spray update 0x001527F0, tick 0x002C5B70
//! with the frame time, render 0x002C6660), and the Xbox quad renderer.
//!
//! API: `ElParticleSystemManager` (resource): `add_type(ElParticleType::new(max, segments, mode, name))`, the
//! type setters through `type_mut`, `add_system(name, type, keep)`, `start` / `stop` / `remove_system`; set a
//! system's `shape`, `velocity`, `transform` through `system_mut`. Users: tyre spray (`tyrespray`), puddle mist
//! and CarDamage smoke (not ported yet).

pub mod el_particle_control_curve;
pub mod el_particle_renderer;
pub mod el_particle_renderer_xbox;
pub mod el_particle_system;
pub mod el_particle_system_manager;
pub mod vs_pool_allocator;

use bevy::prelude::*;

use crate::effect_draw::{EffectClock, EffectDraw};
use el_particle_renderer::CameraBasis;
pub use el_particle_system_manager::{ElParticleSystemManager, SystemId, TypeId};

/// ticks and draws after the producers (tyre spray) updated their systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElParticleSet;

pub struct ElParticlePlugin;

impl Plugin for ElParticlePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::effect_draw::EffectDrawPlugin>() {
            app.add_plugins(crate::effect_draw::EffectDrawPlugin);
        }
        app.init_resource::<ElParticleSystemManager>()
            .configure_sets(PostUpdate, ElParticleSet.in_set(crate::effect_draw::EffectOrder::ElParticle))
            .add_systems(PostUpdate, el_particle_frame.in_set(ElParticleSet));
    }
}

/// GameManager 0x000B9F6A / 0x000B9F8A: `tick(frame ms · 0.001)` then `render(view)` — the real frame delta (0 while
/// paused). Both run inside the view loop: once per view (twice a frame for two players), each tick by the whole
/// frame delta, each render with that view's camera.
fn el_particle_frame(
    time: Res<Time>,
    clock: Res<EffectClock>,
    mut mgr: ResMut<ElParticleSystemManager>,
    views: crate::view_render::ViewCams,
    mut draw: ResMut<EffectDraw>,
    mut rain: Option<ResMut<crate::camera_rain::CameraRain>>,
    rain_in: Option<Res<crate::camera_rain::CameraRainInput>>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
) {
    let dt = if clock.paused { 0.0 } else { time.delta_secs() * clock.scale };
    let list = views.list();
    if list.is_empty() {
        mgr.tick(dt);
        return;
    }
    for v in &list {
        draw.set_view(v.tag);
        // camera rain 0x001341A0 / 0x00135690 sit before the tick in the same view pass
        if let (Some(r), Some(i)) = (rain.as_deref_mut(), rain_in.as_deref()) {
            crate::camera_rain::camera_rain_view(dt, clock.paused, r, i, &mut mgr, &v.transform, v.projection.as_ref(), physics.as_deref(), &mut draw);
        }
        mgr.tick(dt);
        let cam = &v.transform;
        let basis = CameraBasis { right: cam.right().as_vec3(), up: cam.up().as_vec3(), fwd: cam.forward().as_vec3() };
        mgr.render(&basis, &mut draw);
    }
    draw.set_view(None);
}
