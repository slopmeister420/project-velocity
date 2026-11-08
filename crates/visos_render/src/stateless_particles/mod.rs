//! StatelessParticles — emitter types (`.slp`, PartFX.bin), the emitter pool and the CPU particle evaluation,
//! plus the bevy side: loading, updating and drawing (with src\ParticleManager's placed emitters).
//!
//! API: write `LoadParticles { area, sub }` on race / foyer load (`"\\Tracks\\LADown\\"`, `Some("Circuit\\Race01\\")`):
//! PartFX.bin, the generic and the area's PARTDEF.VCD, then the area's and the sub folder's `particle.pld`.
//! `SpawnParticle { name, matrix }` fires a one-shot emitter (skid smoke, collision debris, timeline effects);
//! `ParticleGroup { group, start }` starts / stops placed groups. Time stops while `EffectClock::paused`.

pub mod stateless_particle_emitter;
pub mod stateless_particle_mgr;
pub mod stateless_particle_render_inst;
pub mod stateless_particle_type;

use bevy::camera::primitives::Frustum;
use bevy::prelude::*;

use crate::effect_draw::{EffectClock, EffectDraw,EffectShader, EffectTexture, EffectVertex};
use crate::particle_manager::particle_emitter::{parse_pld, partdef_files};
use crate::particle_manager::particle_manager::CParticleManager;
use stateless_particle_mgr::CStatelessParticleMgr;
use stateless_particle_render_inst::{axis_billboard, frame_table, push_quad};

#[derive(Resource, Default)]
pub struct Particles {
    pub stateless: CStatelessParticleMgr,
    pub placed: CParticleManager,
    /// texture name -> load index (the texture id in the draw sort key)
    texture_ids: std::collections::HashMap<String, u32>,
}

#[derive(Message, Clone, Debug)]
pub struct LoadParticles {
    /// game path of the area (`\Tracks\<t>\`, `\Foyers\<f>\`)
    pub area: String,
    /// track-specific folder below it (the race), with its own `particle.pld`
    pub sub: Option<String>,
}

#[derive(Message, Clone, Debug)]
pub struct UnloadParticles;

#[derive(Message, Clone, Debug)]
pub struct SpawnParticle {
    pub name: String,
    /// d3d row-vector matrix as glam (rows 0-2 = axes, translation in `w_axis`), game space
    pub matrix: Mat4,
}

/// PhysicsObjectManager 0x001E1F80 -> ParticleManager 0x000FFC30: debris of a hit physics object
#[derive(Message, Clone, Copy, Debug)]
pub struct ObjectCollisionParticles {
    /// object type +52 particle kind (`OBJECT_DEBRIS` index)
    pub kind: u32,
    /// the object's matrix (d3d row-vector as glam)
    pub object: Mat4,
    pub point: Vec3,
    pub impulse: f32,
}

/// PhysicsObjectParticleTracker 0x001E4C30 -> ParticleManager 0x001000E0: event 24 propane, 26 wheel fire
#[derive(Message, Clone, Copy, Debug)]
pub struct ParticleTrail {
    pub from: Vec3,
    pub to: Vec3,
    pub event: u32,
}

#[derive(Message, Clone, Debug)]
pub struct ParticleGroup {
    pub group: u32,
    pub start: bool,
}

impl Particles {
    /// GameManager::InGameInit 0x000B63FA (PartFX.bin, resetting the pool) + ParticleEmitter.cpp 0x000FE320
    /// (generic defs, area defs, area instances, track instances)
    pub fn load(&mut self, area: &str, sub: Option<&str>) {
        let area = if area.ends_with('\\') { area.to_string() } else { format!("{area}\\") };
        match visos::file::platform::read("\\Particle\\PartFX.bin") {
            Ok(d) => {
                self.stateless.load_emitter_types(&d, true);
            }
            Err(_) => {
                log::warn!("PartFX.bin missing");
                self.stateless.reset(true);
            }
        }
        self.placed = CParticleManager::default();
        for d in partdef_files("\\Particle\\PARTDEF.VCD") {
            self.stateless.load_emitter_types(&d, false);
        }
        for d in partdef_files(&format!("{area}PARTDEF.vcd")) {
            self.stateless.load_emitter_types(&d, false);
        }
        let mut inst = Vec::new();
        match visos::file::platform::read(&format!("{area}particle.pld")).ok().and_then(|d| parse_pld(&d)) {
            Some(v) => inst.extend(v),
            None => log::warn!("ERROR: No particle data (particle.pld) present for this area!!! ({area})"),
        }
        if let Some(sub) = sub {
            match visos::file::platform::read(&format!("{area}{sub}particle.pld")).ok().and_then(|d| parse_pld(&d)) {
                Some(v) => inst.extend(v),
                None => log::warn!("ERROR: No track specific particle data (particle.pld) present for this track!!! ({area}{sub})"),
            }
        }
        self.texture_ids.clear();
        for t in &self.stateless.types {
            let n = self.texture_ids.len() as u32;
            self.texture_ids.entry(t.texture.to_ascii_lowercase()).or_insert(n);
        }
        self.placed.process(&mut self.stateless, inst);
    }
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::effect_draw::EffectDrawPlugin>() {
            app.add_plugins(crate::effect_draw::EffectDrawPlugin);
        }
        app.init_resource::<Particles>()
            .add_message::<LoadParticles>()
            .add_message::<UnloadParticles>()
            .add_message::<SpawnParticle>()
            .add_message::<ParticleGroup>()
            .add_message::<ObjectCollisionParticles>()
            .add_message::<ParticleTrail>()
            .add_systems(Update, (particle_messages, update_particles).chain())
            .add_systems(PostUpdate, draw_particles.in_set(crate::effect_draw::EffectOrder::Particles));
    }
}

pub(crate) fn particle_messages(
    mut p: ResMut<Particles>,
    mut load: MessageReader<LoadParticles>,
    mut unload: MessageReader<UnloadParticles>,
    mut spawn: MessageReader<SpawnParticle>,
    mut group: MessageReader<ParticleGroup>,
    mut objects: MessageReader<ObjectCollisionParticles>,
    mut trails: MessageReader<ParticleTrail>,
) {
    let p = &mut *p;
    for _ in unload.read() {
        p.placed.stop_all(&mut p.stateless);
        p.placed = CParticleManager::default();
        p.stateless.reset(true);
    }
    for l in load.read() {
        p.load(&l.area, l.sub.as_deref());
    }
    for s in spawn.read() {
        p.stateless.spawn_named(&s.name, s.matrix);
    }
    for o in objects.read() {
        p.placed.object_collision_particles(&mut p.stateless, o.kind, o.object, o.point, o.impulse);
    }
    for t in trails.read() {
        crate::particle_manager::particle_manager::fire_off_particle_trail(&mut p.stateless, t.from, t.to, t.event);
    }
    for g in group.read() {
        if g.start {
            p.placed.start_group(&mut p.stateless, g.group);
        } else {
            p.placed.stop_group(&mut p.stateless, g.group);
        }
    }
}

/// GameManager 0x000B9EC9: `D3::StatelessParticleMgr(dt)` once per rendered frame
pub(crate) fn update_particles(time: Res<Time>, clock: Res<EffectClock>, mut p: ResMut<Particles>) {
    if !clock.paused {
        p.stateless.update(time.delta_secs() * clock.scale);
    }
}

/// GameManager 0x000B9010 per view: `D3::StatelessParticleMgr(dt)` then 0x002D3460 with the view's camera — the
/// first view's tick is `update_particles`, every further view ticks again by the whole frame delta (the original
/// runs the update inside the view loop: twice a frame for two players)
fn draw_particles(
    time: Res<Time>,
    clock: Res<EffectClock>,
    mut p: ResMut<Particles>,
    views: crate::view_render::ViewCams,
    mut draw: ResMut<EffectDraw>,
    mut danger: Option<ResMut<crate::danger_zone_render::DangerParticles>>,
) {
    for (i, v) in views.list().iter().enumerate() {
        if i > 0 {
            // DangerParticles 0x001235F0 (GameManager 0x000B9E24) places the emitters on each view's camera before
            // that view's tick (view 0's: `danger_particles_system`)
            if let Some(dp) = danger.as_deref_mut() {
                dp.place(&v.transform, &mut p.stateless);
            }
            if !clock.paused {
                p.stateless.update(time.delta_secs() * clock.scale);
            }
        }
        draw.set_view(v.tag);
        draw_particles_view(&p, &v.transform, v.frustum.as_ref(), &mut draw);
    }
    draw.set_view(None);
}

/// StatelessParticleMgr.cpp 0x002D3460 — every visible emitter's copies as camera-facing quads
fn draw_particles_view(p: &Particles, cam: &GlobalTransform, frustum: Option<&Frustum>, draw: &mut EffectDraw) {
    let (right, up, fwd) = (cam.right().as_vec3(), cam.up().as_vec3(), cam.forward().as_vec3());
    let list = p.stateless.draw_list(frustum, |t| p.texture_ids.get(&t.texture.to_ascii_lowercase()).copied().unwrap_or(0));
    let mut tris = Vec::new();
    for (idx, copies) in list {
        let e = &p.stateless.emitters[idx];
        let ty = &p.stateless.types[e.type_index as usize];
        let frames = frame_table(ty.frames_u(), ty.frames_v());
        let tex = if ty.texture.is_empty() { EffectTexture::White } else { EffectTexture::Particle(ty.texture.to_ascii_lowercase()) };
        // StatelessParticleRenderInst_XBOX.cpp 0x00321DB0: z test, no z write
        let sh = EffectShader::new(tex, ty.blend_mode).z(true, false);
        for c in copies {
            let (r, u) = if c.axis_billboard { axis_billboard(c.matrix.y_axis.truncate().normalize_or_zero(), fwd) } else { (right, up) };
            tris.clear();
            for (pos, argb, size, angle, frame) in c.particles() {
                let uv = frames.get(frame as usize).unwrap_or(&frames[0]);
                push_quad(&mut tris, pos, argb, size, angle, uv, r, u);
            }
            draw.triangles(&sh, tris.iter().map(|&(p, c, uv)| EffectVertex::new(p, c, uv)));
        }
    }
}
