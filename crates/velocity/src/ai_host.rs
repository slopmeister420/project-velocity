//! glue: the ai's `AiHost` calls core handles (velocity_ai cannot name velocity_game's camera objects' systems).
//! Camera calls made inside an ai update are recorded into `AiCameraCalls` and applied by `game_cameras` right
//! after the Ai set; the ai's cinematic bars join the game OSD's `OsdCinematicBars` stream in the same tick.
//! Add further host calls here (one `AiHostResource` per app).

use bevy::prelude::*;
use velocity_ai::ai::ai_context::AiHost;
use std::sync::{Arc, Mutex};
use velocity_ai::plugin::{AiCinematicBars, AiHostResource, AiState};
use velocity_game::EGameMode;
use visos_render::car_render::CarRender;
use visos_render::flare_render::{CarFlareMessage, CarFlareState};
use velocity_game::camera::game_cameras::{AiCameraCall, AiCameraCalls};
use velocity_game::game_mode::race_session::OsdCinematicBars;
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};

/// FlareRender calls made inside an ai update (vehicle id = physics id = `CarRender.request`), written as
/// `CarFlareMessage`s in the same frame
#[derive(Resource, Clone, Default)]
struct FlareCalls(Arc<Mutex<Vec<CarFlareMessage>>>);

struct CoreHost(AiCameraCalls, FlareCalls);

impl CoreHost {
    fn flare(&self, m: CarFlareMessage) {
        if let Ok(mut v) = self.1 .0.lock() {
            v.push(m);
        }
    }
    fn push(&self, c: AiCameraCall) {
        if let Ok(mut v) = self.0 .0.lock() {
            v.push(c);
        }
    }
}

impl AiHost for CoreHost {
    fn camera_cut(&mut self, id: u32) {
        self.push(AiCameraCall::Cut { id });
    }
    fn crash_camera_reset(&mut self, physics: usize) {
        self.push(AiCameraCall::CrashCameraReset { physics });
    }
    fn camera_point_of_interest_clear(&mut self, priority: f32) {
        self.push(AiCameraCall::PointOfInterestClear { priority });
    }
    /// VsAITrafficVehicle 0x0002B4B0 -> DangerZoneRender 0x00123930 (headlight flash timer 4.0 / 0)
    fn traffic_hazard(&mut self, vehicle_id: u32, on: bool) {
        self.flare(CarFlareMessage::HeadlightFlash { request: vehicle_id as u64, on });
    }
    /// VsAIPoliceVehicle::SetSiren 0x000179B0 -> 0x00123970 (top light on/off, colour a, r, g, b; intensity unused)
    #[allow(clippy::too_many_arguments)]
    fn police_light(&mut self, vehicle_id: u32, light: u32, on: bool, _intensity: f32, a: u8, r: u8, g: u8, b: u8) {
        self.flare(CarFlareMessage::TopLight { request: vehicle_id as u64, light: light as usize, on, a, r, g, b });
    }
}

/// the ai update's FlareRender calls, in call order
fn flare_calls(calls: Res<FlareCalls>, mut out: MessageWriter<CarFlareMessage>) {
    let Ok(mut v) = calls.0.lock() else { return };
    for m in v.drain(..) {
        log::info!("flare call: {m:?}");
        out.write(m);
    }
}

/// FlareRender 0x00125190's test per car (each rendered frame): vehicle +5080 (the traffic manager index, set only for
/// traffic cars) >= 0 and VsAITrafficVehicle 0x0002B5D0 `is_triggered_type` -> the emergency lights of that index
fn traffic_flare_lights(ai: Option<Res<AiState>>, cars: Query<(&CarRender, &CarFlareState)>, mut out: MessageWriter<CarFlareMessage>) {
    let Some(t) = ai.as_ref().and_then(|a| a.world.as_ref()).and_then(|w| w.traffic.as_ref()) else { return };
    for (i, v) in t.vehicles.iter().enumerate() {
        let Some(id) = v.base.physics else { continue };
        let index = v.is_triggered_type().then_some(i as i32);
        if let Some((r, st)) = cars.iter().find(|(r, _)| r.request == id as u64) {
            if st.traffic != index {
                out.write(CarFlareMessage::TrafficLights { request: r.request, index });
            }
        }
    }
}

/// VsAIRaceVehicle 0x00018BD7's OSDManager 0x000ED6E0(false), in the tick the Ai set wrote it
fn ai_bars(mut r: MessageReader<AiCinematicBars>, mut out: MessageWriter<OsdCinematicBars>) {
    for AiCinematicBars(on) in r.read() {
        out.write(OsdCinematicBars(*on));
    }
}

/// VsCrashCamera 0x00062D90 -> VsAIRaceVehicle 0x000208D0: the crash camera's respawn, handed to the ai before its
/// Ai set (a respawn from the ai's own crash camera reset call, written after the Ai set, is applied next update)
fn crash_respawn(
    mut r: MessageReader<velocity_game::camera::game_cameras::CrashCameraRespawn>,
    mut out: MessageWriter<velocity_ai::plugin::AiRespawnOnRaceline>,
) {
    for m in r.read() {
        out.write(velocity_ai::plugin::AiRespawnOnRaceline { vehicle: m.vehicle, speed: m.speed });
    }
}

/// the shared "Game.ScoreManager" globals (core tweaks) and the front end's "Game.OSD" Split Time Display Duration
/// (`dword_6860EC`) into every vehicle's score manager (the original reads the globals directly)
fn score_tweaks(
    gm: Res<velocity_game::GameManager>,
    mut res: ResMut<velocity_game::game_mode::core_tweaks::ScoreTweaksRes>,
    ai: Option<ResMut<AiState>>,
) {
    let tm = &gm.tweaks;
    let split = tm
        .find_system("Game.OSD")
        .and_then(|s| tm.find_tweak(s, "Split Time Display Duration"))
        .map(|id| tm.tweak(id).cell.f32())
        .unwrap_or(2.0);
    if res.0.split_time_duration != split {
        res.0.split_time_duration = split;
    }
    let Some(mut ai) = ai else { return };
    let Some(w) = ai.world.as_mut() else { return };
    for v in w.race.players.iter_mut().chain(w.race.ai.iter_mut()) {
        if let Some(s) = v.score.as_mut() {
            if s.tw != res.0 {
                s.tw = res.0.clone();
            }
        }
    }
}

pub fn build(app: &mut App) {
    app.add_systems(GameTick, score_tweaks.after(InGameUpdateSet::Input).before(InGameUpdateSet::Ai));
    let calls = app.world_mut().get_resource_or_insert_with(AiCameraCalls::default).clone();
    let flares = FlareCalls::default();
    app.insert_resource(AiHostResource(Box::new(CoreHost(calls, flares.clone()))))
        .insert_resource(flares)
        .add_systems(GameTick, ai_bars.after(InGameUpdateSet::Ai).before(InGameUpdateSet::Physics))
        .add_systems(GameTick, crash_respawn.after(InGameUpdateSet::CrashCam).before(InGameUpdateSet::Ai));
    // FlareRender reads its messages in PostUpdate; without it (no renderer) the calls are dropped
    if app.world().contains_resource::<Messages<CarFlareMessage>>() {
        app.add_systems(Update, (flare_calls, traffic_flare_lights.run_if(in_state(EGameMode::InGame))));
    }
    app.add_systems(OnExit(velocity_game::InRace), |c: Res<FlareCalls>| {
        if let Ok(mut v) = c.0.lock() {
            v.clear();
        }
    });
}
