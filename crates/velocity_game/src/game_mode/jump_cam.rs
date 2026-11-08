//! the jump cam glue: physics' VehicleEntity 0x001CCAD0 / 0x001CF0D0 call `g_ReplayCameraManager` (0x001665E0,
//! 0x001665A0) inside the step; core answers through the physics jump cam hook with the replay manager's fixed
//! camera (moved into shared state for the step), mirrors `dword_749110` / `dword_749118` around the step and
//! runs InGameUpdate's countdown (0x000B8329).

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use bevy::math::Vec3;
use bevy::prelude::*;
use velocity_physics::plugin::PhysicsWorldRes;
use velocity_physics::vehicle::vehicle_entity::cx::{JumpCamCall, JumpCamFn, JumpCamWorld};

use crate::camera::game_cameras::GameCameras;
use crate::replays::cameras::replay_camera_fixed::CReplayCameraFixed;
use crate::replays::cameras::ReplayWorld;

/// what the hook touches of `g_ReplayCameraManager` (0x00749128) during a step
#[derive(Default)]
pub struct JumpCamState {
    /// manager +260 (moved in before the step, back after it)
    pub fixed: CReplayCameraFixed,
    /// manager +0x1BE (0x00166490)
    pub review: bool,
    /// manager +17 (0x001665A0 sets it)
    pub unknown_11: bool,
    /// `byte_75B278` as handed to physics for this step (log only: 0x001CCAD0 tests it after CanSee)
    pub option: bool,
}

/// `dword_68580C` "System Update Rate" (FontHelper 0x000B38AC tweak, 1..60): written by VsCommonMain 0x0016E17C
/// (24), FrontEndInit 0x000B5627 (`dword_625578`), InGameInit 0x000B6099 / InGameUpdate 0x000B836B (24) and
/// physics' VehicleEntity 0x001C5800 (jump cam ease 0.005 -> 18); **no game code reads it** (the frame timer's rate
/// is set by VsBounds 0x00057450 with immediates), so the jump cam's "slow motion" changes nothing in F3_D. Kept
/// as the value physics sees between steps.
pub const SYSTEM_UPDATE_RATE: f32 = 24.0;

#[derive(Resource)]
pub struct SystemUpdateRate(pub f32);

impl Default for SystemUpdateRate {
    fn default() -> Self {
        Self(SYSTEM_UPDATE_RATE)
    }
}

/// InGameInit 0x000B6091: `dword_68580C` = 24.0
pub fn reset_system_update_rate(mut r: ResMut<SystemUpdateRate>) {
    r.0 = SYSTEM_UPDATE_RATE;
}

/// the hook and its shared state
#[derive(Resource)]
pub struct JumpCam {
    pub state: Arc<Mutex<JumpCamState>>,
    hook: Arc<Mutex<JumpCamFn>>,
}

impl Default for JumpCam {
    fn default() -> Self {
        let state = Arc::new(Mutex::new(JumpCamState::default()));
        let s = state.clone();
        let hook: Arc<Mutex<JumpCamFn>> =
            Arc::new(Mutex::new(move |c: JumpCamCall, w: &mut dyn JumpCamWorld| answer(&mut s.lock().unwrap(), c, w)));
        Self { state, hook }
    }
}

/// the step's world as the replay cameras see it (0x001649A0 only reads car positions and the camera ray)
struct HookWorld<'a, 'b>(RefCell<&'a mut (dyn JumpCamWorld + 'b)>);

impl ReplayWorld for HookWorld<'_, '_> {
    fn car_position(&self, v: u32) -> Vec3 {
        Vec3::from(self.0.borrow_mut().car_position(v))
    }
    fn wheel_position(&self, _v: u32, _w: u32) -> Vec3 {
        Vec3::ZERO
    }
    fn car_axes(&self, _v: u32) -> (Vec3, Vec3, Vec3) {
        (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO)
    }
    fn blocked(&self, from: Vec3, to: Vec3, v: u32) -> bool {
        self.0.borrow_mut().blocked(from.into(), to.into(), v)
    }
}

/// ReplayCamera_Manager 0x001665E0 (CanSee) / 0x001665A0 (Start)
fn answer(s: &mut JumpCamState, c: JumpCamCall, w: &mut dyn JumpCamWorld) -> bool {
    match c {
        JumpCamCall::CanSee { choose } => {
            if choose {
                // ReplayCamera_Manager.cpp 0x00166B70: fixed +13 = 1
                s.fixed.rechoose = true;
            }
            let review = s.review;
            let seen = s.fixed.choose(review, &HookWorld(RefCell::new(w)));
            log::info!(
                "jump cam: can see (choose {choose}) -> {seen} (option {}, fixed cameras {}, vehicle {}, car at {:?}, camera {:?})",
                s.option,
                s.fixed.count(),
                s.fixed.vehicle,
                s.fixed.target,
                s.fixed.current
            );
            seen
        }
        JumpCamCall::Start => {
            log::info!("jump cam: start");
            s.unknown_11 = true;
            s.fixed.rechoose = true;
            true
        }
    }
}

/// before the step: the hook, the option `byte_75B278`, `dword_749110`, `dword_749118` and the fixed camera
pub fn jump_cam_pre(jc: Res<JumpCam>, rate: Res<SystemUpdateRate>, cams: Option<ResMut<GameCameras>>, physics: Option<Res<PhysicsWorldRes>>) {
    let (Some(p), Some(mut cams)) = (physics.as_deref(), cams) else { return };
    let mut g = p.lock();
    let Some(w) = g.as_mut() else { return };
    w.set_jump_cam_hook(Some(jc.hook.clone()));
    w.set_jump_cam_rate(rate.0);
    let r = &mut cams.bypass_change_detection().replay;
    w.set_jump_cam_option(r.jump_cam_option);
    w.set_review_cameras(r.review_cameras as i32);
    w.set_review_timer(r.review_timer);
    let mut s = jc.state.lock().unwrap();
    s.fixed = std::mem::take(&mut r.fixed);
    s.review = r.review;
    s.unknown_11 = r.unknown_11;
    s.option = r.jump_cam_option;
}

/// after the step: everything back (2 = the jump cam started, with its flight time countdown)
pub fn jump_cam_post(
    jc: Res<JumpCam>,
    mut rate: ResMut<SystemUpdateRate>,
    cams: Option<ResMut<GameCameras>>,
    physics: Option<Res<PhysicsWorldRes>>,
    mut last_flight: Local<f32>,
) {
    let Some(mut cams) = cams else { return };
    let r = &mut cams.bypass_change_detection().replay;
    {
        let mut s = jc.state.lock().unwrap();
        r.fixed = std::mem::take(&mut s.fixed);
        r.unknown_11 = s.unknown_11;
    }
    let Some(p) = physics.as_deref() else { return };
    let g = p.lock();
    let Some(w) = g.as_ref() else { return };
    // the landing predictor's flight time of player 0 (VE+4592, 0x001CCAD0 needs > 0.65 `flt_6889F4`)
    if w.vehicle_ids().contains(&0) {
        let ve = &w.slot(0).ve.m;
        let (flight, cooldown) = (ve.f(4592), ve.i(4876));
        if flight != *last_flight {
            log::info!("jump cam: player 0 predicted flight {flight:.3} s (cooldown {cooldown})");
            *last_flight = flight;
        }
    }
    let new_rate = w.jump_cam_rate();
    if new_rate != rate.0 {
        rate.0 = new_rate;
    }
    let v = w.review_cameras() as u32;
    let (t, start) = w.review_timer();
    if v != r.review_cameras {
        log::info!("jump cam: review cameras {v}, timer {t}");
    }
    r.review_cameras = v;
    r.review_timer = t;
    r.review_timer_start = start;
}

/// InGameUpdate 0x000B8329 — count the jump cam down; at the end `dword_68580C` = 24, review cameras off, VsRender
/// 0x0005C8A0, render cut 0x00060380(1)
pub fn review_timer_countdown(cams: Option<ResMut<GameCameras>>, mut rate: ResMut<SystemUpdateRate>) {
    let Some(mut cams) = cams else { return };
    if cams.replay.review_timer > 0.0 {
        let c = &mut *cams;
        c.replay.review_timer -= 0.04166;
        if c.replay.review_timer <= 0.0 {
            rate.0 = SYSTEM_UPDATE_RATE;
            c.replay.review_cameras = 0;
            c.manager.render_cut = true;
        }
    }
}
