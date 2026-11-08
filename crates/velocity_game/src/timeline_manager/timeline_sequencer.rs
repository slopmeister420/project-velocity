//! src\TimelineManager\TimelineSequencer.cpp — the game-side timeline peer `CTimelineSequencer`
//! (`GameManager+0xC`, `dword_6F5BD4`) of the flag girl start intro: the intro's load (GameDataLoading.cpp
//! 0x0009CB30), its countdown markers, and the peer itself — channels bound to the intro's flag girl models,
//! cameras and particle emitters, placed at the player's grid slot, and the `F3EventCamChange` /
//! `F3EventFlagGirl` events that switch cameras, show the girls and let the cars go. The engine sequencer that
//! plays it is `GameManager+0x1FC`.

use bevy::math::{Mat4, Quat, Vec3};
use visos::file::vs_file_directory::VsFileDirectoryDetails;
use visos::handy::handy_crc::crc32;
use visos::instances::file_instances::FileInstances;
use visos::timeline::file_tlf::{FileItem, FilePayload, FileTimeline, ITEM_KIND_BONE_ANIM, ITEM_KIND_TYPED};
use visos::timeline::timeline_sequencer::{TimelinePeer, TimelineSequencer};
use visos::type_::vs_type::Value;

use crate::camera::vs_base_camera::quat_matrix;

/// the flag girl intro timeline with the times read off it (TimelineSequencer +0xCD8 / +0xCDC).
#[derive(Clone, Debug, Default)]
pub struct FlagGirlTimeline {
    /// file picked from `\Timeline\FlagGirl.vcd`
    pub name: String,
    /// `byte_6F7130` the intro number parsed from the name ("FG_Intro_03.bun" -> 3; 1 for short names)
    pub intro: u8,
    pub timelines: Vec<FileTimeline>,
    /// the bundle's instances (+0xA48): models, cameras and emitters the channels bind to
    pub instances: Option<FileInstances>,
    /// +0xCD8: the "InGame" camera change, less 1e-5 (where pressing a button skips the intro to, and when the
    /// countdown starts)
    pub in_game_time: Option<f32>,
    /// +0xCDC: the flag girl's "ALLOWGO" (the cars may go)
    pub allow_go_time: Option<f32>,
}

/// first field of a typed event's data, as a string (the event structs start with a `string`)
fn first_string(v: &Option<Value>) -> Option<&str> {
    match v.as_ref()? {
        Value::Struct(f) => f.first().and_then(|(_, v)| v.as_str()),
        _ => None,
    }
}

/// TimelineSequencer.cpp 0x001767D0 — scan every typed event: `F3EventCamChange` "InGame" and
/// `F3EventFlagGirl` "ALLOWGO" (case-insensitive); later items overwrite earlier ones.
pub fn countdown_times(tl: &FileTimeline) -> (Option<f32>, Option<f32>) {
    let (mut in_game, mut allow_go) = (None, None);
    for ch in &tl.channels {
        for tr in &ch.tracks {
            for it in &tr.items {
                if it.kind != ITEM_KIND_TYPED {
                    continue;
                }
                let FilePayload::Typed { type_ref: Some(t), value, .. } = &it.payload else { continue };
                let s = first_string(value);
                if t.name.eq_ignore_ascii_case("F3EventCamChange") {
                    if s.is_some_and(|s| s.eq_ignore_ascii_case("InGame")) {
                        in_game = Some(it.start - f32::from_bits(0x3727_C5AC));
                    }
                } else if t.name.eq_ignore_ascii_case("F3EventFlagGirl")
                    && s.is_some_and(|s| s.eq_ignore_ascii_case("ALLOWGO"))
                {
                    allow_go = Some(it.start);
                }
            }
        }
    }
    (in_game, allow_go)
}

/// GameDataLoading.cpp 0x0009CB30 — load entry `index % count` of the pack and read its first timeline.
/// (`byte_6F70F1` selects a variant name; it is never set in F3_D)
pub fn load_flag_girl_timeline(pack: &str, index: u8) -> Option<FlagGirlTimeline> {
    let d = VsFileDirectoryDetails::open(pack).ok()?;
    let names = d.file_names();
    if names.is_empty() {
        return None;
    }
    let name = names[index as usize % names.len()].clone();
    // atoi of the name from 6 characters before its end
    let intro = if name.len() < 6 {
        1
    } else {
        let tail = &name[name.len() - 6..];
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u32>().unwrap_or(0) as u8
    };
    let Ok(b) = d.load_file(&name) else {
        log::warn!("WARNING: Unable to load FG file [{name}] from [{pack}]");
        return None;
    };
    let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
    let bun = match visos::chunk::Bundle::load(&b, &mut env) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("WARNING: Unable to load FG file [{name}] from [{pack}]: {e}");
            return None;
        }
    };
    let mut fg = FlagGirlTimeline { name, intro, timelines: bun.timelines, instances: bun.instances, ..Default::default() };
    // GameDataLoading_GetTimelines(..., 1): the sequencer plays the first one
    if let Some(t) = fg.timelines.first() {
        (fg.in_game_time, fg.allow_go_time) = countdown_times(t);
    }
    Some(fg)
}

/// `MAX_TIMELINE_MODELS`
pub const MAX_MODELS: usize = 5;
/// cameras (+0xA4E count, 116-byte records at +0x2CC)
pub const MAX_CAMERAS: usize = 10;
/// `MAX_TIMELINE_PARTICLES` (the add stops at 9)
pub const MAX_PARTICLES: usize = 10;

/// a flag girl model (+0xA4C names at +0xA4C, 128-byte matrices at +0x44)
#[derive(Clone, Debug, Default)]
pub struct FlagGirlModel {
    /// upper-case instance link name (contains "FLAGGIRL" or "FG_")
    pub name: String,
    /// +0x84 the bound instance's orientation (translation cleared)
    pub base: Mat4,
    /// +0x00 shown
    pub visible: bool,
}

/// a camera of the intro (116 bytes at +0x2CC)
#[derive(Clone, Debug, Default)]
pub struct FlagGirlCamera {
    pub name: String,         // +0x00 upper case
    pub position: Vec3,       // +0x40
    pub target: Vec3,         // +0x50
    pub up: Vec3,             // +0x60
    /// +0x70 vertical fov in degrees from the instance's HorizontalFOV at 4:3
    pub fov: f32,
}

/// a particle emitter of the intro (72 bytes at +0x770)
#[derive(Clone, Debug, Default)]
pub struct FlagGirlParticle {
    pub name: String,
    pub hash: u32,     // +0x40
    pub started: bool, // +0x44
}

/// what a channel drives (`a4[1]`: 3 model, 2 camera, 4 particle)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagGirlRef {
    Model(usize),
    Camera(usize),
    Particle(usize),
}

/// what the intro asks of the cameras, renderer, HUD and race start
#[derive(Clone, Debug, PartialEq)]
pub enum FlagGirlEvent {
    /// TimelineBridge 0x00098C60: a flag girl model's world matrix this update
    ModelMatrix { model: usize, matrix: Mat4 },
    /// "HIDEFLAGGIRL" / "SHOWFLAGGIRL" (and GameManager 0x000BC4D0 / 0x000BC590 for all of them)
    ModelVisible { model: usize, visible: bool },
    /// "InGame": every player's camera cut and back on the follow camera (VsCameraManager 0x0005FFE0 mode 1)
    InGameCameras,
    /// the intro camera now in use (+0x754), `None` = no match ("Unable to find matching camera during cam change")
    Camera(Option<usize>),
    /// +0x758 the next camera frame is a cut
    CameraCut,
    /// OSD3dInterface 0x000EAF60 (the HUD back on)
    ShowHud,
    /// "ALLOWGO": `GameManager+0x15` — the cars may go
    AllowGo,
    /// ParticleManager 0x000FF900 start (once) / 0x000FFA70 stop, emitters of the system placed at `position`
    Particles { system: u32, active: bool, position: Option<Vec3> },
    /// StartLineEffects 0x001472F0: the start line fire at the emitter
    StartLineFire(Vec3),
    /// TimelineSequencer.cpp 0x002B5810: a bone anim item on a model's channel — `.gaf` name, the unwrapped
    /// normalised frame (0x002B5BA0 before the loop / clamp, which needs the anim) and the layer weight (0x002B5D00)
    BoneAnim { model: usize, animation: String, frame: f32, weight: f32 },
}

/// TimelineSequencer.cpp 0x002B5BA0 tail — the unwrapped frame of a `BoneAnim` event to 0..1: wrapped when the anim
/// loops (its +0x9C, BoneAnim 0x002E0F70), else clamped
pub fn bone_anim_frame(frame: f32, looping: bool) -> f32 {
    if looping {
        let v9 = frame - frame.floor();
        if v9 < 0.0 { v9 + 1.0 } else { v9 }
    } else {
        frame.min(1.0).max(0.0)
    }
}

/// TimelineSequencer.cpp 0x002B5D00 — the layer weight: sine ease in over `blend_in`, out over `blend_out`
pub fn bone_anim_weight(item: &FileItem, local_t: f32) -> f32 {
    let FilePayload::BoneAnim { blend_in, blend_out, weight, .. } = item.payload else { return 0.0 };
    let hi = item.length;
    let v6 = local_t.clamp(0.0, hi) / hi;
    let v8 = if v6 >= blend_in {
        if 1.0 - blend_out >= v6 {
            1.0
        } else {
            ((1.0 - v6) / blend_out * 3.1415927f32 / 2.0).sin()
        }
    } else {
        (v6 / blend_in * 3.1415927f32 / 2.0).sin()
    };
    v8 * weight
}

/// where the intro is placed: the player's grid slot (`dword_6F5C0C`, y cleared for cameras and emitters) and
/// the start direction in radians (`dword_6F5C10`)
#[derive(Clone, Copy, Debug, Default)]
pub struct FlagGirlPlacement {
    pub grid: Vec3,
    pub heading: f32,
    /// CarDamage 0x000761B0 player car +0x8CC: bit 0 left camera, bit 1 right camera preferred
    pub car_flags: u32,
}

/// the flag girl peer
#[derive(Clone, Debug, Default)]
pub struct FlagGirlSequencer {
    pub models: Vec<FlagGirlModel>,
    pub cameras: Vec<FlagGirlCamera>,
    pub particles: Vec<FlagGirlParticle>,
    /// per channel of the bound timeline
    pub refs: Vec<Option<FlagGirlRef>>,
    pub current_camera: Option<usize>, // +0x754
    pub camera_cut: bool,              // +0x758
    /// +0xCD4 time the update seeks to before stepping (-1 = none)
    pub seek: f32,
    /// `dword_74AA38` the camera path target last used (a new one is a cut)
    pub last_camera_path: u32,
    /// GameManager+0x20 the girls are shown
    pub shown: bool,
    /// +0xB8C anim names (count +0xCCD, ≤ 5), from the bound timeline's bone anim items (0x00176640)
    pub anims: Vec<String>,
    /// what 0x00175870 loaded for the models and anims
    pub loads: TimelineModelLoads,
}

/// the files TimelineSequencer.cpp 0x00175870 loads (`\FlagGirl\FG_T.vcd` textures pushed, then FG_M models and
/// FG_A anims); the render crate loads the geometry itself, this is the original's lookup and its log
#[derive(Clone, Debug, Default)]
pub struct TimelineModelLoads {
    /// LoadModelGeometry (GameDataLoading.cpp 0x0009FE10) file per model (+0x08 + 4i); `None` = not found
    pub models: Vec<Option<String>>,
    /// GameDataLoading.cpp 0x000A0080 file per anim (+0x1C + 4j); `None` = not found
    pub anims: Vec<Option<String>>,
}

/// `MAX_TIMELINE_ANIMS`
pub const MAX_ANIMS: usize = 5;

/// the renderer's model extension (`dword_74A434` +4)
const MODEL_EXT: &str = "xmf";

/// LoadModelGeometry's file name: `name.<ext>` unless the name (or its upper-case copy) already contains it
pub fn model_file_name(name: &str) -> String {
    if name.contains(MODEL_EXT) || name.to_ascii_uppercase().contains(MODEL_EXT) {
        name.to_string()
    } else {
        format!("{name}.{MODEL_EXT}")
    }
}

struct Peer<'a> {
    fg: &'a mut FlagGirlSequencer,
    instances: Option<&'a FileInstances>,
    place: FlagGirlPlacement,
    out: &'a mut Vec<FlagGirlEvent>,
}

/// first string field of a typed event (the event structs start with a string)
fn typed_strings(v: Option<&Value>) -> Vec<&str> {
    match v {
        Some(Value::Struct(f)) => f.iter().filter_map(|(_, v)| v.as_str()).collect(),
        _ => Vec::new(),
    }
}

impl FlagGirlSequencer {
    /// TimelineSequencer.cpp 0x00175680 (reset) + 0x00176490 models, 0x00176020 cameras, 0x001762D0 emitters
    pub fn new(instances: Option<&FileInstances>) -> Self {
        let mut s = FlagGirlSequencer { seek: -1.0, ..Default::default() };
        let Some(inst) = instances else { return s };
        for l in &inst.links {
            let n = l.a.clone().unwrap_or_default().to_uppercase();
            if n.contains("FLAGGIRL") || n.contains("FG_") {
                if s.models.len() >= MAX_MODELS {
                    log::error!("assert: too many flag girl models");
                    continue;
                }
                s.models.push(FlagGirlModel { name: n, base: Mat4::IDENTITY, visible: true });
            }
        }
        for i in &inst.instances {
            match i.type_name() {
                Some("VSCamera") => {
                    if s.cameras.len() >= MAX_CAMERAS {
                        log::error!("assert: too many timeline cameras");
                        continue;
                    }
                    let mut c = FlagGirlCamera { name: i.name.clone().unwrap_or_default().to_uppercase(), ..Default::default() };
                    if let Some(h) = i.value.as_ref().and_then(|v| v.field("HorizontalFOV")).and_then(Value::as_f32) {
                        let a = ((0.5 * h) * 0.017453292f32).tan() as f64 * 3.0 / 4.0;
                        let v = a.atan() as f32;
                        c.fov = (v + v) * 57.295776;
                    }
                    s.cameras.push(c);
                }
                _ if i.type_ref.as_ref().is_some_and(|t| t.crc == crc32(b"F3ParticleEmitter", 0)) => {
                    if s.particles.len() >= MAX_PARTICLES {
                        log::error!("assert: m_u8NumParticles < MAX_TIMELINE_PARTICLES");
                    }
                    if s.particles.len() < 9 {
                        let name = i.name.clone().unwrap_or_default();
                        let hash = crate::timeline_manager::objects::timeline_object::ccrc(&name);
                        s.particles.push(FlagGirlParticle { name, hash, started: false });
                    }
                }
                _ => {}
            }
        }
        s
    }

    /// TimelineSequencer.cpp 0x00176640 — the anim of every bone anim item (kind 1) of the timeline, in channel /
    /// track / item order, duplicates included (the count asserts below `MAX_ANIMS`)
    pub fn bind_anims(&mut self, tl: &FileTimeline) {
        for ch in &tl.channels {
            for tr in &ch.tracks {
                for it in &tr.items {
                    if it.kind != ITEM_KIND_BONE_ANIM {
                        continue;
                    }
                    if let FilePayload::BoneAnim { animation, .. } = &it.payload {
                        self.anims.push(animation.clone().unwrap_or_default());
                        if self.anims.len() >= MAX_ANIMS {
                            log::error!("assert: m_u8NumAnims < MAX_TIMELINE_ANIMS");
                        }
                    }
                }
            }
        }
    }

    /// TimelineSequencer.cpp 0x00175870 — open FG_T / FG_M / FG_A, push FG_T as the texture directory, load each
    /// model's geometry from FG_M and each anim from FG_A; a model that did not load logs "Missing timeline model"
    /// (its bone palette +0x75C and 28-byte draw record +0x30 are the render crate's)
    pub fn load_models(&mut self) {
        let m = VsFileDirectoryDetails::open("\\FlagGirl\\FG_M.vcd").ok();
        let a = VsFileDirectoryDetails::open("\\FlagGirl\\FG_A.vcd").ok();
        let found = |d: &Option<VsFileDirectoryDetails>, f: String| d.as_ref().and_then(|d| d.load_file(&f).ok()).map(|_| f);
        let mut loads = TimelineModelLoads::default();
        for md in &self.models {
            let f = found(&m, model_file_name(&md.name));
            if f.is_none() {
                log::warn!("Failed to load model \"{}\"", model_file_name(&md.name));
            }
            loads.models.push(f);
        }
        for an in &self.anims {
            // 0x000A0080: the name plus ".gaf"
            let f = found(&a, format!("{an}.gaf"));
            if f.is_none() {
                log::warn!("Failed to load anim \"{an}.gaf\"");
            }
            loads.anims.push(f);
        }
        for (i, f) in loads.models.iter().enumerate() {
            if f.is_none() {
                log::warn!("Missing timeline model {i}");
            }
        }
        self.loads = loads;
    }

    /// TimelineSequencer.cpp 0x00174930 — the loaded anim named `name` (case-insensitive), else "No matching animation"
    pub fn find_anim(&self, name: &str) -> Option<&str> {
        for (i, a) in self.anims.iter().enumerate() {
            if a.eq_ignore_ascii_case(name) {
                return self.loads.anims.get(i).and_then(|f| f.as_deref());
            }
        }
        log::info!("No matching animation");
        None
    }

    /// TimelineSequencer.cpp 0x00175E40 — camera by upper-case name ("NONE" never matches)
    pub fn find_camera(&self, name: &str) -> Option<usize> {
        let n = name.to_uppercase();
        if n == "NONE" {
            return None;
        }
        self.cameras.iter().position(|c| c.name == n)
    }

    /// TimelineSequencer.cpp 0x00175810 — the emitters may start again (race restart)
    pub fn reset_particles(&mut self) {
        for p in &mut self.particles {
            p.started = false;
        }
    }

    /// GameManager.cpp 0x000BC4D0 / 0x000BC590 — every model shown / hidden
    pub fn show(&mut self, on: bool, out: &mut Vec<FlagGirlEvent>) {
        for (k, m) in self.models.iter_mut().enumerate() {
            m.visible = on;
            out.push(FlagGirlEvent::ModelVisible { model: k, visible: on });
        }
        self.shown = on;
    }

    fn rotation(place: &FlagGirlPlacement) -> Mat4 {
        // TimelineSequencer 0x002B7750 axis-angle about +y, AC::Renderer to a matrix
        let q = Quat::from_xyzw(0.0, (0.5 * (0.0 - place.heading)).sin(), 0.0, (0.5 * (0.0 - place.heading)).cos());
        Mat4::from_mat3(quat_matrix(q))
    }
}

/// the flag girl intro: engine sequencer (+0x1FC) and the peer (+0xC)
#[derive(Clone, Debug, Default)]
pub struct FlagGirlIntro {
    pub data: FlagGirlTimeline,
    pub seq: TimelineSequencer,
    pub peer: FlagGirlSequencer,
    /// VsStringAnsi 0x002B47E0: a timeline is bound
    pub bound: bool,
    /// the players' cameras are on the cine camera showing the intro's camera (mode 3 until "InGame")
    pub cine: bool,
}

impl FlagGirlIntro {
    /// GameManager.cpp (FlagGirlTimeline step): the peer, then the sequencer bound to the first timeline
    pub fn new(data: FlagGirlTimeline, place: FlagGirlPlacement, out: &mut Vec<FlagGirlEvent>) -> Self {
        let mut peer = FlagGirlSequencer::new(data.instances.as_ref());
        // GameDataLoading 0x0009CB30 binds the anims at load, GameManager 0x000BA920 loads the models before the bind
        if let Some(t) = data.timelines.first() {
            peer.bind_anims(t);
        }
        peer.load_models();
        let mut seq = TimelineSequencer::default();
        let bound = match data.timelines.first() {
            Some(t) => {
                let mut p = Peer { fg: &mut peer, instances: data.instances.as_ref(), place, out };
                seq.bind(t, &mut p)
            }
            None => false,
        };
        if !bound {
            log::warn!("flag-girl sequencer has no timeline set");
        }
        FlagGirlIntro { data, seq, peer, bound, cine: false }
    }

    /// VsStringAnsi 0x002B4890 — the sequencer time
    pub fn time(&self) -> f32 {
        self.seq.time
    }

    /// VsStringAnsi 0x002B4DA0 Rewind then 0x002B4CF0 Play (GameManager MonitorSessions state 2)
    pub fn start(&mut self, place: FlagGirlPlacement, out: &mut Vec<FlagGirlEvent>) {
        let Some(t) = self.data.timelines.first() else { return };
        let mut p = Peer { fg: &mut self.peer, instances: self.data.instances.as_ref(), place, out };
        self.seq.set_time(0.0, t, &mut p);
        self.seq.play(t, &mut p);
    }

    /// GameManager.cpp InGameUpdate (0x000B77F0 tail): seek if a skip asked for it, then step
    pub fn update(&mut self, dt: f32, place: FlagGirlPlacement, out: &mut Vec<FlagGirlEvent>) {
        let Some(t) = self.data.timelines.first() else { return };
        let mut p = Peer { fg: &mut self.peer, instances: self.data.instances.as_ref(), place, out };
        if p.fg.seek >= 0.0 {
            let s = p.fg.seek;
            self.seq.set_time(s, t, &mut p);
            p.fg.seek = -1.0;
        }
        self.seq.update(dt, t, &mut p);
    }
}

impl TimelinePeer for Peer<'_> {
    /// TimelineSequencer.cpp 0x00174480 — model (first 6 characters of a model name), camera, then emitter
    fn bind(&mut self, c: usize, tl: &FileTimeline) -> bool {
        if self.fg.refs.len() < tl.channels.len() {
            self.fg.refs.resize(tl.channels.len(), None);
        }
        let ch = &tl.channels[c];
        let name = ch.name.clone().unwrap_or_default().to_uppercase();
        for k in 0..self.fg.models.len() {
            let m = &self.fg.models[k].name;
            if name.bytes().take(6).eq(m.bytes().take(6)) {
                match self.instances.and_then(|i| i.instances.iter().find(|i| i.name_crc == ch.crc)) {
                    Some(i) => {
                        let mut base = Mat4::from_cols_array_2d(&i.world);
                        base.w_axis = bevy::math::Vec4::W;
                        self.fg.models[k].base = base;
                    }
                    None => log::warn!("WARNING: Unable to find flag girl instance in bundle"),
                }
                self.fg.refs[c] = Some(FlagGirlRef::Model(k));
                return true;
            }
        }
        if let Some(k) = self.fg.find_camera(&name) {
            self.fg.refs[c] = Some(FlagGirlRef::Camera(k));
            return true;
        }
        let h = crate::timeline_manager::objects::timeline_object::ccrc(ch.name.as_deref().unwrap_or(""));
        if let Some(k) = self.fg.particles.iter().position(|p| p.hash == h) {
            self.fg.refs[c] = Some(FlagGirlRef::Particle(k));
            return true;
        }
        log::warn!("WARNING: Unknown reference in timeline");
        false
    }

    /// TimelineSequencer.cpp 0x00174A70 — path results placed at the grid slot
    fn set_target_matrix(&mut self, c: usize, item: &FileItem, m: Mat4) {
        let rot = FlagGirlSequencer::rotation(&self.place);
        let grid0 = Vec3::new(self.place.grid.x, 0.0, self.place.grid.z);
        let FilePayload::Path { target_crc, flags, .. } = item.payload else { return };
        match self.fg.refs.get(c).copied().flatten() {
            Some(FlagGirlRef::Camera(k)) => {
                // translation(path position) * rotation, then the slot (y cleared)
                let p = rot.transform_point3(m.w_axis.truncate()) + grid0;
                let cam = &mut self.fg.cameras[k];
                if flags & 1 != 0 {
                    cam.position = p;
                    if target_crc != self.fg.last_camera_path {
                        self.fg.camera_cut = true;
                        self.out.push(FlagGirlEvent::CameraCut);
                        self.fg.last_camera_path = target_crc;
                    }
                } else if flags & 0x10 != 0 {
                    cam.target = p;
                } else if flags & 8 != 0 {
                    cam.up = p;
                }
            }
            Some(FlagGirlRef::Model(k)) => {
                // base * path * rotation, then the slot (height kept)
                let mut w = rot * m * self.fg.models[k].base;
                w.w_axis += self.place.grid.extend(0.0);
                self.out.push(FlagGirlEvent::ModelMatrix { model: k, matrix: w });
            }
            Some(FlagGirlRef::Particle(k)) => {
                let p = &mut self.fg.particles[k];
                if p.hash == crc32(b"stopparticles", 0) {
                    self.out.push(FlagGirlEvent::Particles { system: crc32(b"startlinefireeffect", 0), active: false, position: None });
                    return;
                }
                p.started = true;
                let pos = rot.transform_point3(m.w_axis.truncate()) + grid0;
                // the emitter is dropped onto the ground found between 5 m above and below the slot (owner's ray)
                self.out.push(FlagGirlEvent::Particles { system: p.hash, active: true, position: Some(pos) });
                if p.hash == crc32(b"startlinefireeffect", 0) {
                    self.out.push(FlagGirlEvent::StartLineFire(pos));
                }
            }
            None => {}
        }
    }

    /// TimelineSequencer.cpp 0x00175270
    fn event_begin(&mut self, c: usize, item: &FileItem, _t: f32, _length: f32) {
        let FilePayload::Typed { type_ref: Some(t), value, .. } = &item.payload else { return };
        let s = typed_strings(value.as_ref());
        let arg = |i: usize| s.get(i).copied().unwrap_or("");
        if t.name.eq_ignore_ascii_case("F3EventCamChange") {
            if arg(0).eq_ignore_ascii_case("InGame") {
                self.fg.camera_cut = true;
                self.out.push(FlagGirlEvent::InGameCameras);
                self.out.push(FlagGirlEvent::ShowHud);
                self.fg.current_camera = None;
            } else {
                let (mid, left, right) = (arg(0), arg(1), arg(2));
                let flags = self.place.car_flags;
                let cam = if left.eq_ignore_ascii_case("None") || flags & 1 == 0 {
                    if !right.eq_ignore_ascii_case("None") && flags & 2 != 0 {
                        self.fg.find_camera(right)
                    } else {
                        self.fg.find_camera(mid).or_else(|| self.fg.find_camera(left)).or_else(|| self.fg.find_camera(right))
                    }
                } else {
                    self.fg.find_camera(left)
                };
                if cam.is_none() {
                    log::warn!("WARNING: Unable to find matching camera during cam change");
                }
                self.fg.current_camera = cam;
            }
            self.out.push(FlagGirlEvent::Camera(self.fg.current_camera));
        } else if t.name.eq_ignore_ascii_case("F3EventFlagGirl") {
            let e = arg(0);
            let model = match self.fg.refs.get(c).copied().flatten() {
                Some(FlagGirlRef::Model(k)) => Some(k),
                _ => None,
            };
            if e.eq_ignore_ascii_case("ALLOWGO") {
                self.out.push(FlagGirlEvent::AllowGo);
            } else if e.eq_ignore_ascii_case("HIDEFLAGGIRL") || e.eq_ignore_ascii_case("SHOWFLAGGIRL") {
                let visible = e.eq_ignore_ascii_case("SHOWFLAGGIRL");
                if let Some(k) = model {
                    self.fg.models[k].visible = visible;
                    self.out.push(FlagGirlEvent::ModelVisible { model: k, visible });
                }
            } else if e.eq_ignore_ascii_case("SHOWHUD") {
                self.out.push(FlagGirlEvent::ShowHud);
            } else {
                log::warn!("Unknown FlagGirl Event [{e}]");
            }
        } else {
            log::warn!("Unsupported timeline event [{}]", t.name);
        }
    }

    /// TimelineSequencer.cpp 0x002B5810 — the model's anim layer (sampled by the renderer)
    fn bone_anim(&mut self, c: usize, item: &FileItem, local_t: f32) {
        let Some(FlagGirlRef::Model(model)) = self.fg.refs.get(c).copied().flatten() else { return };
        let FilePayload::BoneAnim { animation: Some(a), start, end, .. } = &item.payload else { return };
        let hi = item.length;
        let frame = local_t.clamp(0.0, hi) / hi * (end - start) + start;
        self.out.push(FlagGirlEvent::BoneAnim { model, animation: a.clone(), frame, weight: bone_anim_weight(item, local_t) });
    }
}

/// the camera the intro shows (VsCameraManager cine camera of player 0 while the intro runs)
pub fn intro_camera_view(fg: &FlagGirlSequencer) -> Option<crate::game_bridge::camera_bridge::CameraBridgeData> {
    let c = &fg.cameras[fg.current_camera?];
    Some(crate::game_bridge::camera_bridge::CameraBridgeData {
        position: c.position,
        target: c.target,
        up: if c.up == Vec3::ZERO { Vec3::Y } else { c.up },
        fov: c.fov.to_radians(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every shipped intro loads and carries both markers, the camera change before the go.
    #[test]
    fn flag_girl_intros() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        for i in 0..10u8 {
            let fg = load_flag_girl_timeline("\\Timeline\\FlagGirl.vcd", i).expect("flag girl timeline");
            assert_eq!(fg.intro, i + 1, "{}", fg.name);
            let (a, b) = (fg.in_game_time.unwrap(), fg.allow_go_time.unwrap());
            assert!(a > 0.0 && b > a, "{} {a} {b}", fg.name);
        }
    }

    /// 0x00175870 / 0x00176640 on every shipped intro: which model and anim files the original's lookup finds
    #[test]
    fn flag_girl_model_loads() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let (mut models, mut found_m, mut anims, mut found_a) = (0, 0, 0, 0);
        for i in 0..10u8 {
            let fg = load_flag_girl_timeline("\\Timeline\\FlagGirl.vcd", i).expect("flag girl timeline");
            let intro = FlagGirlIntro::new(fg, FlagGirlPlacement::default(), &mut Vec::new());
            let l = &intro.peer.loads;
            assert_eq!(l.models.len(), intro.peer.models.len());
            assert_eq!(l.anims.len(), intro.peer.anims.len());
            eprintln!("intro {i}: models {:?} -> {:?}; anims {:?} -> {:?}", intro.peer.models.iter().map(|m| &m.name).collect::<Vec<_>>(), l.models, intro.peer.anims, l.anims);
            models += l.models.len();
            found_m += l.models.iter().flatten().count();
            anims += l.anims.len();
            found_a += l.anims.iter().flatten().count();
        }
        eprintln!("models {found_m}/{models}, anims {found_a}/{anims}");
        // shipped: 12 / 12 models and 26 / 26 anims found, no "Missing timeline model"
        assert!(models > 0 && anims > 0);
        assert_eq!((found_m, found_a), (models, anims));
    }

    /// every intro binds, plays to its end, switches to the in-game cameras and lets the cars go
    #[test]
    fn flag_girl_intros_play() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        for i in 0..10u8 {
            let fg = load_flag_girl_timeline("\\Timeline\\FlagGirl.vcd", i).expect("flag girl timeline");
            let place = FlagGirlPlacement { grid: Vec3::new(10.0, 2.0, 20.0), heading: 0.5, car_flags: 0 };
            let mut out = Vec::new();
            let name = fg.name.clone();
            let mut intro = FlagGirlIntro::new(fg, place, &mut out);
            assert!(intro.bound, "{name}");
            assert!(!intro.peer.models.is_empty(), "{name}");
            intro.start(place, &mut out);
            for _ in 0..24 * 40 {
                intro.update(1.0 / 24.0, place, &mut out);
            }
            assert!(out.contains(&FlagGirlEvent::AllowGo), "{name}");
            assert!(out.contains(&FlagGirlEvent::InGameCameras), "{name}");
            assert!(out.iter().any(|e| matches!(e, FlagGirlEvent::Camera(Some(_)))), "{name}");
            assert!(out.iter().any(|e| matches!(e, FlagGirlEvent::ModelMatrix { .. })), "{name}");
            // the girls' bone anims name `.gaf`s of the flag girl pack, on bound models
            let fga = visos::file::Vcd::open("\\FlagGirl\\FG_A.vcd").unwrap();
            let anims: Vec<_> = out.iter().filter_map(|e| match e {
                FlagGirlEvent::BoneAnim { model, animation, frame, weight } => Some((*model, animation.clone(), *frame, *weight)),
                _ => None,
            }).collect();
            assert!(!anims.is_empty(), "{name}");
            for (m, a, f, w) in anims {
                assert!(m < intro.peer.models.len() && fga.get(&format!("{a}.gaf")).is_some(), "{name}: {m} {a}");
                assert!(f.is_finite() && (0.0..=1.0001).contains(&w), "{name}: {a} {f} {w}");
            }
        }
    }
}
