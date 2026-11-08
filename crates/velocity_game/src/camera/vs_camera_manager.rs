//! src\Camera\VsCameraManager.cpp — `g_CameraMan` (0x006D55C8): per player the follow camera and its mirror,
//! the fly camera (crash camera / intro), the roaming debug camera and the 3d interface camera, and which of them
//! is the player's view. The cameras live here (the original keeps pointers, the fly / roaming / 3d interface
//! cameras belonging to the game manager).
//!
//! `CCinematicCameraSet` / `CCinematicCamera` (0x00060790 / 0x000603C0 / 0x00060CE0, key-framed cameras read from
//! a text file) are compiled in but referenced by nothing in F3_D, so they are not ported.

use bevy::math::Vec3;

use super::vs_base_camera::VsBaseCamera;
use super::vs_fly_camera::{VsFlyCamera, VsRoamingCamera};
use super::vs_vehicle_follow_camera::{CameraTweaks, CameraView, FollowGlobals, VsVehicleFollowCamera};
use crate::game_bridge::camera_bridge::CameraBridgeData;

pub const MAX_PLAYERS: usize = 2;

/// near plane of views not mounted on the car (`dword_68509C`)
pub const DEFAULT_NEAR: f32 = 0.5;
/// .data far planes (`dword_6850A0`, mirror `dword_6850A8`), halved in split screen: defaults of [`ClipTweaks`]
pub const FAR: f32 = 600.0;
pub const MIRROR_FAR: f32 = 200.0;
/// .data near plane of the other cameras (`dword_6850A4`; view changes overwrite it with the view's)
pub const OTHER_NEAR: f32 = 1.0;

/// the "Graphics.Camera" globals (GameManager 0x000B4170): read when a camera is set up (0x0005ECC0, 0x000B73D0,
/// 0x000B75D0, 0x000B7730) and by the tweak-changed listener 0x000C0A90 ([`VsCameraManager::clip_tweak_changed`])
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipTweaks {
    pub far: f32,        // dword_6850A0 "FarClipDistance"
    pub near: f32,       // dword_6850A4 "NearClipDistance" (0x0005F140 writes the view's [17])
    pub mirror: bool,    // byte_685808 "Enable Mirror" (read by 0x000B88B0)
    pub mirror_far: f32, // dword_6850A8 "Mirror FarClipDistance"
}

impl Default for ClipTweaks {
    fn default() -> Self {
        ClipTweaks { far: FAR, near: OTHER_NEAR, mirror: true, mirror_far: MIRROR_FAR }
    }
}
/// 3d interface camera field of view (0x3F490FDB, 45 degrees)
pub const INTERFACE_FOV: f32 = f32::from_bits(0x3F49_0FDB);

/// `m_PlayerCams[n].m_Mode` (+0x20)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ECameraMode {
    #[default]
    None = 0,
    Follow = 1,
    Mirror = 2,
    /// the fly camera ("cine")
    Cine = 3,
    Roaming = 4,
    Interface3d = 5,
}

/// one player's cameras (44 bytes)
#[derive(Clone, Debug, Default)]
pub struct PlayerCams {
    /// +0x04 index of the car view in use
    pub view: u32,
    pub follow: Option<VsVehicleFollowCamera>, // +0x0C
    /// +0x10 the rear-view mirror camera
    pub mirror: Option<VsVehicleFollowCamera>,
    pub cine: Option<VsFlyCamera>,             // +0x14
    pub roaming: Option<VsRoamingCamera>,      // +0x18
    pub interface3d: Option<VsBaseCamera>,     // +0x1C
    pub mode: ECameraMode,                     // +0x20
    /// +0x28 view changed this update (camera cut; 0x0002C620 also sends message 3)
    pub cut: bool,
    /// +0x29 in the zone (ZoneRender_PC_XBOX 0x00163740, written by the zone FX each rendered frame)
    pub in_zone: bool,
    /// the car's view table (`VehicleData` list a, 80-byte records)
    pub views: Vec<[f32; 20]>,
    /// near / far of the follow and mirror cameras
    pub follow_near: f32,
    pub follow_far: f32,
    pub mirror_near: f32,
    pub mirror_far: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VsCameraManager {
    pub players: Vec<PlayerCams>,
    /// +0x58 per player views that may be cycled to (bit per view; bit 3 = the in-car view is allowed)
    pub view_masks: [u16; MAX_PLAYERS],
    /// +0x60
    pub unknown_60: u32,
    /// `byte_6F915D` 16:9
    pub widescreen: bool,
    /// the "Camera" tweak globals (`dword_685204..685214`, VsFlyCamera 0x000668C0 / 0x00066830)
    pub camera_tweaks: CameraTweaks,
    /// the follow cameras' tweakable globals ("Camera.Globals.*", "Camera.TurnRates", ZRotation)
    pub globals: FollowGlobals,
    /// the "Graphics.Camera" clip globals
    pub clip: ClipTweaks,
    /// `byte_6F7157` two players (the far planes are halved), as at init
    pub split_screen: bool,
    /// port: 0x00060380(1) was called (GameBridge +0x14); the glue moves it to the bridge
    pub render_cut: bool,
}

impl VsCameraManager {
    /// VsCameraManager.cpp 0x0005ECC0 — each player gets a follow camera on view 0 of its car and a mirror
    /// camera (`views[p]` = the car's table, single player list or split screen list per 0x0005F0F0)
    pub fn init(&mut self, views: Vec<Vec<[f32; 20]>>, split_screen: bool) {
        let n = views.len();
        if n > MAX_PLAYERS {
            log::error!("assert: m_nPlayers <= MAX_PLAYERS");
        }
        self.unknown_60 = 0;
        self.split_screen = split_screen;
        self.players.clear();
        for (p, table) in views.into_iter().enumerate().take(MAX_PLAYERS) {
            let mut pc = PlayerCams { mode: ECameraMode::None, cut: true, ..Default::default() };
            self.view_masks[p] = 0xFFFF;
            let v0 = table.first().map(CameraView::from_record).unwrap_or_default();
            let follow = VsVehicleFollowCamera { view: v0, player: p as u32, ..Default::default() };
            // the mirror takes the same view with its 7th parameter cleared, flagged rear-facing
            let mirror = VsVehicleFollowCamera {
                view: CameraView { unknown_528: 0.0, chase: 0.0, ..v0 },
                player: p as u32,
                mirror: true,
                rear: true,
                ..Default::default()
            };
            if p == 0 {
                self.camera_tweaks = CameraTweaks::from_view(&follow.view);
            }
            let s = if split_screen { 0.5 } else { 1.0 };
            pc.follow_near = if v0.chase == 0.0 { DEFAULT_NEAR } else { v0.near };
            pc.follow_far = self.clip.far * s;
            pc.mirror_near = table.get(2).map(|r| r[17]).unwrap_or(DEFAULT_NEAR);
            pc.mirror_far = self.clip.mirror_far * s;
            pc.follow = Some(follow);
            pc.mirror = Some(mirror);
            pc.views = table;
            pc.view = 0;
            self.players.push(pc);
            self.set_mode(ECameraMode::Follow, p);
        }
    }

    /// VsCameraManager.cpp 0x0005FFB0
    pub fn mode(&self, p: usize) -> ECameraMode {
        self.players.get(p).map(|c| c.mode).unwrap_or_default()
    }

    /// VsCameraManager.cpp 0x0005FFE0 — switch a player's view (only on a change)
    pub fn set_mode(&mut self, mode: ECameraMode, p: usize) {
        let Some(pc) = self.players.get_mut(p) else { return };
        if pc.mode == mode {
            return;
        }
        pc.mode = mode;
        match mode {
            ECameraMode::Cine => {
                if let Some(c) = pc.cine.as_mut() {
                    c.base.update_extents();
                    c.base.copy_to_render();
                }
            }
            ECameraMode::Roaming => {
                if let Some(c) = pc.roaming.as_mut() {
                    c.base.update_extents();
                    c.base.copy_to_render();
                }
            }
            ECameraMode::Interface3d => {
                if let Some(c) = pc.interface3d.as_mut() {
                    c.update_extents();
                    c.copy_to_render();
                }
            }
            _ => {}
        }
    }

    /// VsCameraManager.cpp 0x00060200 — allow / forbid the in-car view (bit 3)
    pub fn allow_in_car_view(&mut self, p: usize, on: bool) {
        if p < self.players.len() {
            if on {
                self.view_masks[p] |= 8;
            } else {
                self.view_masks[p] &= !8;
            }
        }
    }

    /// VsCameraManager.cpp 0x00060260
    pub fn is_in_car_view(&self, p: usize) -> bool {
        self.players.get(p).is_some_and(|c| c.view == 3)
    }

    /// VsCameraManager.cpp 0x0005F140 — next car view (`index` < 0) or a given one; not while the fly camera
    /// is the view. Returns whether the view changed.
    pub fn change_view(&mut self, p: usize, index: i8) -> bool {
        let mask = self.view_masks.get(p).copied().unwrap_or(0);
        let Some(pc) = self.players.get_mut(p) else { return false };
        if pc.follow.is_none() || pc.mode == ECameraMode::Cine {
            return false;
        }
        let count = pc.views.len() as u32;
        if index >= 0 {
            pc.view = index as u32;
        } else {
            // the original spins until an allowed view comes round
            for _ in 0..64 {
                pc.view += 1;
                if pc.view == count {
                    pc.view = 0;
                }
                if pc.view > 3 {
                    pc.view = 0;
                }
                if (1u16 << pc.view) & mask != 0 {
                    break;
                }
            }
        }
        let Some(rec) = pc.views.get(pc.view as usize) else { return false };
        let mut v = CameraView::from_record(rec);
        let f = pc.follow.as_mut().unwrap();
        pc.follow_near = if v.chase == 0.0 { DEFAULT_NEAR } else { v.near };
        if v.chase == 0.0 && v.unknown_528 == 0.0 && v.height < 0.75 {
            v.height = 0.75;
        }
        // `dword_6850A4` (the "NearClipDistance" tweak) = the view's [17]
        self.clip.near = v.near;
        // 0x00069650: +524 changing between zero and non-zero cuts the next update
        if (v.chase == 0.0) != (f.view.chase == 0.0) {
            f.cut_pending = true;
        }
        f.view = v;
        f.view_index = pc.view;
        if p == 0 && pc.mode == ECameraMode::Follow {
            self.camera_tweaks = CameraTweaks::from_view(&f.view);
        }
        // 0x0005FA80 (extents, the VE's camera pointers) has nothing to do here; then 0x00060380(1)
        self.render_cut = true;
        pc.cut = true;
        true
    }

    /// VsCameraManager.cpp 0x0005F8F0 — per update: aspect of every active camera, then the fly / roaming
    /// cameras run (the follow cameras are updated with their cars)
    pub fn update(
        &mut self,
        timeline: &mut [Option<super::vs_fly_camera::TimelineCameraOutput>],
        ground_y: &dyn Fn(Vec3) -> Option<f32>,
        roam_input: &dyn Fn(crate::controller::user_input::EAction) -> f32,
    ) -> Vec<Option<bool>> {
        let aspect = if self.widescreen { 1.777_777_8 } else { 1.333_333_4 };
        let mut frozen = Vec::new();
        for (p, pc) in self.players.iter_mut().enumerate() {
            match pc.mode {
                ECameraMode::Cine => {
                    if let Some(c) = pc.cine.as_mut() {
                        c.base.set_aspect(aspect);
                        let tc = timeline.get_mut(p).and_then(|t| t.as_mut());
                        let _bridge = c.update(tc, ground_y);
                        c.base.copy_to_render();
                        // VsFlyCamera 0x00064090: a timeline camera cut (+1880) → 0x00060380(1)
                        if c.cut {
                            c.cut = false;
                            self.render_cut = true;
                        }
                    }
                    frozen.push(None);
                }
                ECameraMode::Roaming => {
                    if let Some(c) = pc.roaming.as_mut() {
                        c.base.set_aspect(aspect);
                        let (_, fz) = c.update(roam_input);
                        c.base.copy_to_render();
                        frozen.push(Some(fz));
                    } else {
                        frozen.push(None);
                    }
                }
                ECameraMode::Follow => {
                    if p == 0 {
                        Self::write_back_view(pc);
                    }
                    frozen.push(None);
                }
                _ => frozen.push(None),
            }
        }
        frozen
    }

    /// VsCameraManager.cpp 0x0005F800 — player 0's live (tweaked) chase view goes back into the car's table entry
    /// of its current view: [2] height, [0] interest height, [4] / [5] distances, [11] fov, [17] near
    pub fn write_back_view(pc: &mut PlayerCams) {
        let Some(f) = pc.follow.as_ref() else { return };
        let Some(rec) = pc.views.get_mut(pc.view as usize) else { return };
        rec[2] = f.view.height;
        rec[0] = f.view.look_height;
        rec[4] = f.view.dist_a;
        rec[5] = f.view.dist_b;
        rec[11] = f.view.fov;
        rec[17] = pc.follow_near;
    }

    /// the view a player sees now, as a camera hand-over record
    pub fn view(&self, p: usize) -> Option<CameraBridgeData> {
        let pc = self.players.get(p)?;
        let from_base = |b: &VsBaseCamera| CameraBridgeData {
            position: b.pos_render,
            target: b.pos_render + b.dir_render,
            up: b.up_render,
            fov: b.fov_render,
        };
        let from_follow = |f: &VsVehicleFollowCamera| {
            f.bridge.unwrap_or(CameraBridgeData { position: f.eye, target: f.eye + f.look, up: f.up, fov: f.fov_rad })
        };
        match pc.mode {
            ECameraMode::Follow => pc.follow.as_ref().map(from_follow),
            ECameraMode::Mirror => pc.mirror.as_ref().map(from_follow),
            ECameraMode::Cine => pc.cine.as_ref().map(|c| from_base(&c.base)),
            ECameraMode::Roaming => pc.roaming.as_ref().map(|c| from_base(&c.base)),
            ECameraMode::Interface3d => pc.interface3d.as_ref().map(from_base),
            ECameraMode::None => None,
        }
    }

    /// GameManager.cpp 0x000B7730 — "Free Cameras": a roaming camera per player (mode 4 then back by the
    /// caller); 0x000B75D0 — "3d Interface": the 3d interface camera looking at the origin from z = 30
    pub fn create_debug_cameras(&mut self) {
        let clip = self.clip;
        for pc in &mut self.players {
            let mut r = VsRoamingCamera::default();
            r.base.set_near(clip.near);
            r.base.set_far(clip.far);
            pc.roaming = Some(r);
            let mut i = VsBaseCamera::default();
            i.set_near(clip.near);
            i.set_far(clip.far);
            i.set_fov(INTERFACE_FOV);
            i.look_at(Vec3::new(0.0, 0.0, 30.0), Vec3::ZERO, Vec3::Y);
            pc.interface3d = Some(i);
        }
    }

    /// GameManager.cpp 0x000C0A90 — the listener GameManager 0x000B4170 adds on MessageManager channel 1 (every
    /// tweak change, not just "Graphics.Camera"): player 0's current camera (+0x24) takes the far (halved in split
    /// screen) and near globals, its mirror the mirror far. The "Display Debug Graph" copy at the end
    /// (`byte_7B21E1` → `unk_7AB208` +0x6FD8) has no port (no debug graph).
    pub fn clip_tweak_changed(&mut self) {
        let s = if self.split_screen { 0.5 } else { 1.0 };
        let (far, near, mirror_far) = (self.clip.far * s, self.clip.near, self.clip.mirror_far * s);
        let Some(pc) = self.players.first_mut() else { return };
        match pc.mode {
            ECameraMode::Follow => {
                pc.follow_far = far;
                pc.follow_near = near;
            }
            ECameraMode::Mirror => {
                pc.mirror_far = far;
                pc.mirror_near = near;
            }
            ECameraMode::Cine => {
                if let Some(c) = pc.cine.as_mut() {
                    c.base.set_far(far);
                    c.base.set_near(near);
                }
            }
            ECameraMode::Roaming => {
                if let Some(c) = pc.roaming.as_mut() {
                    c.base.set_far(far);
                    c.base.set_near(near);
                }
            }
            ECameraMode::Interface3d => {
                if let Some(c) = pc.interface3d.as_mut() {
                    c.set_far(far);
                    c.set_near(near);
                }
            }
            // the original dereferences the null camera here
            ECameraMode::None => {}
        }
        pc.mirror_far = mirror_far;
    }

    /// GameManager.cpp 0x000B73D0 — "Fly Cameras": each player's fly camera follows the intro timeline from
    /// behind the grid slot, and becomes the view
    pub fn start_intro(&mut self, grid: &[Vec3], split_screen: bool) {
        let n = self.players.len();
        for p in 0..n {
            let g = grid.get(p).copied().unwrap_or(Vec3::ZERO);
            let mut f = VsFlyCamera { player: p as u32, mode: 1, target: g, ..Default::default() };
            f.base.pos = g + Vec3::new(0.0, 1.5, -10.0);
            f.base.set_near(self.clip.near);
            f.base.set_far(self.clip.far * if split_screen { 0.5 } else { 1.0 });
            self.players[p].cine = Some(f);
            self.set_mode(ECameraMode::Cine, p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Vec<[f32; 20]> {
        let mut v = vec![[0.0f32; 20]; 4];
        for (i, r) in v.iter_mut().enumerate() {
            r[2] = 1.0 + i as f32;
            r[11] = 50.0;
            r[17] = 0.1;
        }
        v[3][14] = 1.0;
        v
    }

    #[test]
    fn views_cycle_through_the_mask() {
        let mut m = VsCameraManager::default();
        m.init(vec![table()], false);
        assert_eq!(m.mode(0), ECameraMode::Follow);
        m.view_masks[0] = 0b0101;
        assert!(m.change_view(0, -1));
        assert_eq!(m.players[0].view, 2);
        assert!(m.change_view(0, -1));
        assert_eq!(m.players[0].view, 0);
        // view 0's height 1.0 is kept (>= 0.75)
        assert_eq!(m.players[0].follow.as_ref().unwrap().view.height, 1.0);
        m.create_debug_cameras();
        m.start_intro(&[Vec3::new(10.0, 0.0, 0.0)], false);
        assert_eq!(m.mode(0), ECameraMode::Cine);
        assert!(m.view(0).is_some());
    }
}
