//! src/Audio/VsAudioVehics.cpp — per-car sound: emitters placed around the car, tyre skid/drift/brake loops
//! by surface, suspension clunks, the ai engines, traffic passbys/horns/sirens, police sirens and physics
//! impacts. (the module begins at 0x0004FD50; the assert-based region map starts it at 0x000531D0.)

use bevy::math::Vec3;

use crate::audio::system::{AudioSystem, EmitterId, EventSlot};
use crate::input::{AiCarAudioInput, CarAudioInput, PoliceAudioInput, TrafficAudioInput};
use crate::vs_audio_events::crc;

/// `Audio.*` tweaks of this file (0x00684FBC..0x00685068), defaults from the data section.
#[derive(Clone, Debug)]
pub struct VehicleTweaks {
    pub activate_dist2: f32,   // 0x684FBC
    pub deactivate_dist2: f32, // 0x684FC0
    pub ai_revs_rate: f32,     // 0x684FC8 "ai revs rate of change"
    pub siren_times: [(f32, f32); 4], // 0x684FCC.. "State n min/max time"
    pub susp_wait_frames: i32, // 0x684FEC
    pub front_pressure: f32,   // 0x684FF0
    pub rear_pressure: f32,    // 0x684FF4
    pub passby: f32,           // 0x684FF8
    pub impact_distance: f32,  // 0x684FFC
    pub front_vol: [f32; 4],   // 0x685000 per camera view
    pub wheel_vol: [f32; 4],   // 0x685010
    pub rear_vol: [f32; 4],    // 0x685020
    pub drift_light_on: f32,   // 0x685034
    pub drift_light_peak: f32, // 0x685038
    pub drift_light_end: f32,  // 0x68503C
    pub drift_heavy_on: f32,   // 0x685040
    pub drift_heavy_full: f32, // 0x685044
    pub brake_on: f32,         // 0x685048
    pub brake_full: f32,       // 0x68504C
    pub brake_max_speed: f32,  // 0x685054
    pub pitch_speed_mid: f32,  // 0x685058
    pub pitch_speed_max: f32,  // 0x68505C
    pub pitch_low: f32,        // 0x685060
}

impl Default for VehicleTweaks {
    fn default() -> Self {
        Self {
            activate_dist2: 10000.0,
            deactivate_dist2: 14400.0,
            ai_revs_rate: 50.0,
            siren_times: [(2.0, 8.0), (2.5, 7.0), (0.5, 1.0), (4.0, 10.0)],
            susp_wait_frames: 5,
            front_pressure: f32::from_bits(0x3D4C_CCCD),
            rear_pressure: f32::from_bits(0x3D4C_CCCD),
            passby: f32::from_bits(0x3F4C_CCCD),
            impact_distance: 10.0,
            front_vol: [0.0, f32::from_bits(0xBE99_999A), f32::from_bits(0x3E99_999A), f32::from_bits(0x3E4C_CCCD)],
            wheel_vol: [0.0, f32::from_bits(0xBE99_999A), f32::from_bits(0x3E99_999A), f32::from_bits(0x3E4C_CCCD)],
            rear_vol: [0.0, f32::from_bits(0x3E33_3333), f32::from_bits(0xBE99_999A), f32::from_bits(0xBE99_999A)],
            drift_light_on: 1.0,
            drift_light_peak: 20.0,
            drift_light_end: 30.0,
            drift_heavy_on: 5.0,
            drift_heavy_full: 50.0,
            brake_on: 10.0,
            brake_full: 80.0,
            brake_max_speed: 33.0,
            pitch_speed_mid: 8.0,
            pitch_speed_max: 40.0,
            pitch_low: f32::from_bits(0x3F33_3333),
        }
    }
}

pub const IMP_SUSPENSION_SML: u32 = 0xD9B7_43DD; // -642300963
pub const IMP_SUSPENSION_MED: u32 = 0x090D_CF9D; // 151900061
pub const IMP_SUSPENSION_LRG: u32 = 0x9445_7086; // -1807388538
pub const IMPACT_CAR_PHYSICS: u32 = 0x9843_D147; // -1740385977
pub const PASSBY_DRY: u32 = 0x3C40_012F; // rn_tarmac_passby_dry_light
pub const PASSBY_WET: u32 = 0xBEA4_2669; // -1096538519, not in this build's data
pub const PASSBY_EMERGENCY: u32 = 1314; // numbered id, not in this build's data
pub const HORN_PRESS: u32 = 979; // numbered id, not in this build's data
pub const FIRE_SIREN: u32 = 0x4CBD_6277; // petrolstation_fireenginesiren
pub const HORNS: [u32; 3] = [0x4D97_6D81, 0xD49E_3C3B, 0xA399_0CAD]; // horn_car_1..3
pub const SIREN_POLICE_SILENCE: u32 = 0xE172_2B32; // -512611534
pub const SIREN_POLICE_01: u32 = 0x0ECE_2B40;
pub const SIREN_POLICE_02: u32 = 0x97C7_7AFA;
pub const SIREN_POLICE_03: u32 = 0xE0C0_4A6C; // -524268948

/// `VsVehicleAudio` (172 bytes): one car's emitters, event names and running events.
pub struct VsVehicleAudio {
    /// +0 engine, +4, +8 body, +12/+16 drift light L/R, +20/+24 drift heavy L/R, +28/+32 brake L/R, +36 rear
    pub emitters: [Option<EmitterId>; 10],
    /// port: `VsAudio::frame_scale`, set before the per-frame updates
    pub frame_scale: f32,
    pub surface: [u32; 2],    // +40/+44
    pub road: [u32; 2],       // +48 road noise
    pub brake: [u32; 2],      // +56
    pub drift_light: [u32; 2], // +64
    pub drift_heavy: [u32; 2], // +72
    pub wheelspin: [u32; 2],  // +80
    pub engine: EventSlot,    // +88
    pub passby: EventSlot,    // +92
    pub siren: EventSlot,     // +96
    pub brake_ev: [EventSlot; 2], // +108
    pub drift_light_ev: [EventSlot; 2], // +116
    pub drift_heavy_ev: [EventSlot; 2], // +124
    pub rpm: f32,             // +140
    pub last_rpm: f32,        // +144
    pub drift_acc: [f32; 2],  // +148
    pub brake_acc: [f32; 2],  // +156
    pub horn: u32,            // +164 (-1 = none)
    pub siren_state: i8,      // +168
    pub active: bool,         // +169
    pub horn_played: bool,    // +170
    pub horn_pressed: bool,   // +171
}

impl VsVehicleAudio {
    /// VsAudioVehics.cpp 0x00052670
    pub fn new(audio: &mut AudioSystem) -> Self {
        Self {
            emitters: [None; 10],
            frame_scale: 1.0,
            surface: [0; 2],
            road: [0; 2],
            brake: [0; 2],
            drift_light: [0; 2],
            drift_heavy: [0; 2],
            wheelspin: [0; 2],
            engine: audio.new_slot(),
            passby: audio.new_slot(),
            siren: audio.new_slot(),
            brake_ev: [audio.new_slot(), audio.new_slot()],
            drift_light_ev: [audio.new_slot(), audio.new_slot()],
            drift_heavy_ev: [audio.new_slot(), audio.new_slot()],
            rpm: 0.0,
            last_rpm: 0.0,
            drift_acc: [0.0; 2],
            brake_acc: [0.0; 2],
            horn: 0,
            siren_state: 0,
            active: false,
            horn_played: false,
            horn_pressed: false,
        }
    }

    fn slots(&self) -> [EventSlot; 9] {
        [
            self.engine,
            self.passby,
            self.siren,
            self.brake_ev[0],
            self.brake_ev[1],
            self.drift_light_ev[0],
            self.drift_light_ev[1],
            self.drift_heavy_ev[0],
            self.drift_heavy_ev[1],
        ]
    }

    /// VsAudioVehics.cpp 0x000527F0: stop every event and destroy the emitters.
    pub fn release(&mut self, audio: &mut AudioSystem) {
        for s in self.slots() {
            audio.stop_slot(s);
        }
        for e in self.emitters.iter_mut() {
            if let Some(id) = e.take() {
                audio.destroy_emitter(id);
            }
        }
        self.active = false;
    }

    /// VsAudioVehics.cpp 0x00050770: place the emitters around the car and set their view-dependent volumes.
    pub fn update_emitters(
        &mut self,
        audio: &mut AudioSystem,
        car: &CarAudioInput,
        view: i8,
        engine_vol: f32,
        tyres_vol: f32,
        tw: &VehicleTweaks,
    ) {
        let p = car.position;
        let side = car.forward.cross(car.up);
        let e = self.emitters;
        if let Some(x) = e[0] {
            audio.set_emitter_position(x, p);
            audio.set_emitter_velocity(x, car.velocity);
            audio.set_emitter_volume(x, view_volume(engine_vol, &tw.front_vol, view));
        }
        if let Some(x) = e[2] {
            audio.set_emitter_position(x, p);
            audio.set_emitter_volume(x, view_volume(tyres_vol, &tw.wheel_vol, view));
        }
        if let Some(x) = e[1] {
            audio.set_emitter_position(x, p);
            audio.set_emitter_volume(x, view_volume(engine_vol, &tw.front_vol, view));
        }
        let left = p + side * 1.0;
        if let Some(x) = e[3] {
            audio.set_emitter_position(x, left);
            audio.set_emitter_velocity(x, car.velocity);
        }
        if let Some(x) = e[5] {
            audio.set_emitter_position(x, left);
            audio.set_emitter_volume(x, view_volume(tyres_vol, &tw.wheel_vol, view));
        }
        if let Some(x) = e[7] {
            audio.set_emitter_position(x, left);
        }
        let right = p - side * 1.0;
        if let Some(x) = e[4] {
            audio.set_emitter_position(x, right);
        }
        if let Some(x) = e[6] {
            audio.set_emitter_position(x, right);
            audio.set_emitter_volume(x, view_volume(tyres_vol, &tw.wheel_vol, view));
        }
        if let Some(x) = e[8] {
            audio.set_emitter_position(x, right);
        }
        let rear = p - car.forward * 0.75;
        if let Some(x) = e[9] {
            audio.set_emitter_position(x, rear);
            audio.set_emitter_volume(x, view_volume(engine_vol, &tw.rear_vol, view));
        }
    }

    /// VsAudioVehics.cpp 0x00050DD0: the per-side skid/drift/road-noise event names for the surface underneath.
    pub fn build_surface_events(&mut self, audio: &mut AudioSystem, side: usize, wet: bool) {
        let side_name = if side != 0 { "RIGHT" } else { "LEFT" };
        let (mut a, mut b) = surface_prefix(self.surface[side], wet, false);
        self.brake[side] = crc(&format!("{a}BRAKE_{side_name}"));
        if audio.find_event(self.brake[side]) == -1 {
            (a, b) = surface_prefix(self.surface[side], wet, true);
            self.brake[side] = crc(&format!("{a}BRAKE_{side_name}"));
        }
        self.drift_light[side] = crc(&format!("{a}DRIFT_LIGHT_{side_name}"));
        self.drift_heavy[side] = crc(&format!("{a}DRIFT_HEAVY_{side_name}"));
        self.wheelspin[side] = crc(&format!("{a}WHEELSPIN_{side_name}"));
        self.road[side] = crc(&format!("{b}{side_name}"));
    }

    /// VsAudioVehics.cpp 0x00051B00: brake/lock-up squeal from the longitudinal slip of one wheel.
    #[allow(clippy::too_many_arguments)]
    pub fn update_brakes(
        &mut self,
        audio: &mut AudioSystem,
        car: &CarAudioInput,
        wheel: usize,
        side: usize,
        tyres_vol: f32,
        tw: &VehicleTweaks,
    ) {
        let mut vel = car.velocity;
        vel.y = 0.0;
        let speed = vel.length();
        let mut fwd = car.forward;
        fwd.y = 0.0;
        fwd = fwd.normalize_or_zero();
        let s = car.wheels[wheel].slip;
        let mut slip = Vec3::new(s.x, 0.0, s.z);
        let slip_len = slip.length();
        slip = slip.normalize_or_zero();
        if car.reversing {
            fwd = -fwd;
        }
        let mut v = -slip.dot(fwd) * slip_len;
        if !car.wheels[wheel].contact {
            v = 0.0;
        }
        if v < 0.0 {
            v = 0.0;
        }
        if speed > tw.brake_max_speed {
            v = 0.0;
        }
        // (acc + v) * 0.7 per reference frame
        self.brake_acc[side] = crate::vs_audio::leaky(self.brake_acc[side], v, f32::from_bits(0x3F33_3333), self.frame_scale);
        let em = self.emitters[7 + side];
        let slot = self.brake_ev[side];
        if self.brake_acc[side] > tw.brake_on && audio.slot(slot).is_none() {
            if let Some(e) = em {
                audio.play_event(e, self.brake[side], None, 0, Some(slot));
            }
        }
        if audio.slot(slot).is_some() {
            let t = ((self.brake_acc[side] - tw.brake_on) / (tw.brake_full - tw.brake_on)).clamp(0.0, 1.0);
            if let Some(e) = em {
                audio.set_emitter_volume(e, t * tyres_vol);
            }
            if t == 0.0 {
                audio.stop_slot(slot);
            } else if audio.slot_sample(slot).is_some() {
                let mut k = if speed <= tw.pitch_speed_mid {
                    (tw.pitch_speed_mid - speed) * tw.pitch_low / tw.pitch_speed_mid
                } else {
                    (speed - tw.pitch_speed_mid) / (tw.pitch_speed_max - tw.pitch_speed_mid)
                };
                k = k.clamp(0.0, 1.0);
                set_pitch_in_range(audio, slot, k);
            }
        }
    }

    /// VsAudioVehics.cpp 0x00051F10: light/heavy drift loops from the slip angle of the car.
    #[allow(clippy::too_many_arguments)]
    pub fn update_drift(&mut self, audio: &mut AudioSystem, car: &CarAudioInput, wheel: usize, side: usize, tw: &VehicleTweaks) {
        let mut vel = car.velocity;
        vel.y = 0.0;
        let speed = vel.length();
        let vel = vel.normalize_or_zero();
        let mut fwd = car.forward;
        fwd.y = 0.0;
        fwd = fwd.normalize_or_zero();
        if car.reversing {
            fwd = -fwd;
        }
        let d = vel.dot(fwd);
        let cross = vel.cross(fwd);
        let mut v = (1.0 - d) * speed;
        if cross.y <= 0.0 {
            if side == 1 {
                v *= f32::from_bits(0x3E99_999A); // 0.3
            }
        } else if side == 0 {
            v *= f32::from_bits(0x3E99_999A);
        }
        if !car.wheels[wheel].contact {
            v = 0.0;
        }
        // (acc + v) * 0.8 per reference frame
        self.drift_acc[side] = crate::vs_audio::leaky(self.drift_acc[side], v, f32::from_bits(0x3F4C_CCCD), self.frame_scale);
        let acc = self.drift_acc[side];
        let (light, heavy) = (self.drift_light_ev[side], self.drift_heavy_ev[side]);
        if acc > tw.drift_light_on && audio.slot(light).is_none() {
            if let Some(e) = self.emitters[3 + side] {
                audio.play_event(e, self.drift_light[side], None, 0, Some(light));
            }
        }
        if acc > tw.drift_heavy_on && audio.slot(heavy).is_none() {
            if let Some(e) = self.emitters[5 + side] {
                audio.play_event(e, self.drift_heavy[side], None, 0, Some(heavy));
            }
        }
        if audio.slot(heavy).is_some() {
            let t = ((acc - tw.drift_heavy_on) / (tw.drift_heavy_full - tw.drift_heavy_on)).clamp(0.0, 1.0);
            if let Some(e) = self.emitters[5 + side] {
                audio.set_emitter_volume(e, t);
            }
            if t == 0.0 {
                audio.stop_slot(heavy);
            } else if audio.slot_sample(heavy).is_some() {
                set_pitch_in_range(audio, heavy, t);
            }
        }
        if audio.slot(light).is_some() {
            let t = if tw.drift_light_peak < acc {
                1.0 - (acc - tw.drift_light_peak) / (tw.drift_light_end - tw.drift_light_peak)
            } else {
                (acc - tw.drift_light_on) / (tw.drift_light_peak - tw.drift_light_on)
            }
            .clamp(0.0, 1.0);
            if let Some(e) = self.emitters[3 + side] {
                audio.set_emitter_volume(e, t);
            }
            if t == 0.0 {
                audio.stop_slot(light);
            } else if audio.slot_sample(light).is_some() {
                set_pitch_in_range(audio, light, t);
            }
        }
    }
}

/// VsAudioVehics.cpp 0x000505F0 / 0x00050670 / 0x000506F0 FrontVolume, WheelVolume, RearVolume: an option volume plus the camera view's offset.
pub fn view_volume(v: f32, table: &[f32; 4], view: i8) -> f32 {
    if v == 0.0 {
        return 0.0;
    }
    // view -1 (before the first camera query) reads one entry before the table in the original
    let off = if (0..4).contains(&view) { table[view as usize] } else { 0.0 };
    (v + off).clamp(0.0, 1.0)
}

/// VsAudioVehics.cpp 0x00050B40: ("SKID_<SURF>_[DRY_|WET_]", "RN_<SURF>_[DRY_|WET_]")
pub fn surface_prefix(surface: u32, wet: bool, force_dry: bool) -> (String, String) {
    let mut a = String::from("SKID_");
    let mut b = String::from("RN_");
    let (sa, sb) = match surface {
        1 => {
            a.push_str("TARMAC_WET_");
            b.push_str("TARMAC_WET_");
            return (a, b);
        }
        9 | 10 => ("DIRT_", "DIRT_"),
        11 => ("GRASS_", "GRASS_"),
        12 => ("GRAVEL_", "GRAVEL_DEEP_"),
        13 => ("GRAVEL_", "GRAVEL_LOOSE_"),
        _ => ("TARMAC_", "TARMAC_"),
    };
    a.push_str(sa);
    b.push_str(sb);
    if !wet || force_dry {
        a.push_str("DRY_");
        b.push_str("DRY_");
    } else {
        a.push_str("WET_");
        b.push_str("WET_");
    }
    (a, b)
}

/// `CRim__452C0(min, max, t)` on the event sound's pitch range (`ev +24` sound, +32/+36).
fn set_pitch_in_range(audio: &mut AudioSystem, slot: EventSlot, t: f32) {
    let Some(ev) = audio.slot(slot) else { return };
    let Some(a) = audio.events.get(ev) else { return };
    let (r, si) = (a.event, a.sound);
    let Some(sd) = audio.event(r).and_then(|e| e.sounds.get(si)).map(|s| (s.def.pitch_min, s.def.pitch_max)) else {
        return;
    };
    let p = sd.0 + (sd.1 - sd.0) * t;
    if let Some(ps) = audio.slot_sample(slot) {
        audio.set_sample_pitch(ps, p);
    }
}

/// the suspension state machine's statics (`dword_6D52D8..6D52F8`).
#[derive(Default)]
pub struct SuspensionState {
    pub init: u32,           // 6D52F8
    pub last: [f32; 4],      // 6D52F4 (wheel 3), F0 (0), EC (2), E8 (1)
    pub all_counter: f32,    // 6D52D8 (frames; reference frames here)
    pub all_latched: bool,   // 6D52DC
    pub two_counter: f32,    // 6D52E0
    pub two_latched: bool,   // 6D52E4
}

/// VsAudioVehics.cpp 0x00051570: clunks when a wheel's compression jumps and landings after airtime.
pub fn update_suspension(
    audio: &mut AudioSystem,
    st: &mut SuspensionState,
    emitter: Option<EmitterId>,
    car: &CarAudioInput,
    tw: &VehicleTweaks,
    scale: f32,
) {
    // the airtime counters count rendered frames: `scale` = reference frames this frame
    let wait = tw.susp_wait_frames as f32;
    let order = [3usize, 0, 2, 1];
    let contact: Vec<bool> = order.iter().map(|&w| car.wheels[w].contact).collect();
    let comp: Vec<f32> = order.iter().map(|&w| car.wheels[w].compression).collect();
    for (k, bit) in [1u32, 2, 4, 8].into_iter().enumerate() {
        if st.init & bit == 0 {
            st.init |= bit;
            st.last[k] = comp[k];
        }
    }
    let play = |audio: &mut AudioSystem, h: u32| {
        if let Some(e) = emitter {
            audio.play_event(e, h, None, 0, None);
        }
    };
    let (pair, limit) = if car.front_suspension { ([0usize, 1], tw.front_pressure) } else { ([2usize, 3], tw.rear_pressure) };
    for k in pair {
        let d = comp[k] - st.last[k];
        if d > limit || 0.0 - limit > d {
            play(audio, IMP_SUSPENSION_SML);
        }
        st.last[k] = comp[k];
    }
    let airborne = contact.iter().filter(|c| !**c).count() as i32;
    if !st.two_latched {
        if airborne < 2 {
            st.two_counter = 0.0;
        } else {
            st.two_counter += scale;
            st.two_latched = st.two_counter > wait;
        }
    }
    if !st.all_latched {
        if airborne == 4 {
            st.all_counter += scale;
            st.all_latched = st.all_counter > wait;
        } else {
            st.all_counter = 0.0;
        }
    }
    let mut fire = None;
    if st.all_latched && airborne < 4 {
        st.all_latched = false;
        if airborne < 2 {
            st.two_latched = false;
        }
        fire = Some(IMP_SUSPENSION_LRG);
    }
    if st.two_latched && airborne < 2 {
        st.two_latched = false;
        fire = Some(IMP_SUSPENSION_MED);
    }
    if let Some(h) = fire {
        play(audio, h);
    }
}

/// VsAudioVehics.cpp 0x000531D0
pub fn load_ai_engine(audio: &mut AudioSystem, path: &str) -> i32 {
    match audio.load_group(path, None) {
        Ok(_) => 0,
        Err(e) => {
            log::warn!("Failed to load audio group 'AIEngine'");
            e
        }
    }
}

/// the ai engine event of an engine group path: crc of its upper-cased base name (PhysicsManager 0x0019E080).
pub fn ai_engine_event(path: &str) -> u32 {
    let base = &path[path.rfind(['\\', '/']).map(|i| i + 1).unwrap_or(0)..];
    crc(&base.to_ascii_uppercase())
}

/// VsAudio::UpdateAiCars (0x00053290) for one ai race vehicle (`m_pAIRaceVehicles[i - players]`, 0x00017310).
pub fn update_ai_car(
    audio: &mut AudioSystem,
    va: &mut VsVehicleAudio,
    ai: &AiCarAudioInput,
    ai_max_rpm: i32,
    sfx_vol: f32,
    tw: &VehicleTweaks,
    view: i8,
    engine_vol: f32,
    tyres_vol: f32,
) {
    va.update_emitters(audio, &ai.car, view, engine_vol, tyres_vol, tw);
    let Some(e0) = va.emitters[0] else { return };
    if audio.slot(va.engine).is_none() && ai.present {
        audio.set_emitter_flags(e0, 0x4000_0000);
        audio.play_event(e0, ai.engine_event, None, 0, Some(va.engine));
    }
    if audio.slot(va.engine).is_some() && ai.present {
        // the event's own pitch range is read and then replaced by fixed values
        let _ = audio.event_pitch_min(ai.engine_event);
        let _ = audio.event_pitch_max(ai.engine_event);
        let (pmin, pmax, rmin) = (-1000.0f32, 50.0f32, 1000.0f32);
        let rpm = match ai.ai_rpm {
            Some(r) => r,
            None => ai.car.engine_rpm as f32 + (audio.rand() % 100) as f32,
        };
        let t = (rpm - rmin) / (ai_max_rpm as f32 - rmin);
        let pitch = (pmax - pmin) * t + pmin;
        if let Some(ps) = audio.slot_sample(va.engine) {
            audio.set_sample_pitch(ps, pitch);
            let mut vol = f32::from_bits(0x3EB3_3334) * t + f32::from_bits(0x3E4C_CCCD);
            if va.last_rpm - rpm <= tw.ai_revs_rate {
                vol *= f32::from_bits(0x3EE6_6666); // 0.45
            } else {
                vol *= 0.25;
            }
            audio.set_sample_volume(ps, vol);
        }
        audio.set_emitter_volume(e0, sfx_vol);
        va.last_rpm = rpm;
    }
}

/// VsAudio::UpdateTraffic (0x00053660) for one traffic car. `muted` is `byte_6D5054`.
#[allow(clippy::too_many_arguments)]
pub fn update_traffic_car(
    audio: &mut AudioSystem,
    va: &mut VsVehicleAudio,
    t: &TrafficAudioInput,
    player_pos: Vec3,
    wet: bool,
    muted: bool,
    sfx_vol: f32,
    tw: &VehicleTweaks,
    view: i8,
    engine_vol: f32,
    tyres_vol: f32,
) {
    let mut release = false;
    if !t.car.active || muted {
        release = true;
    } else {
        let d2 = (player_pos - t.car.position).length_squared();
        if tw.activate_dist2 <= d2 || va.active {
            if d2 > tw.deactivate_dist2 && va.active {
                va.active = false;
                release = true;
            }
        } else {
            va.active = true;
            if va.emitters[0].is_none() {
                va.emitters[0] = audio.create_emitter(0x4000_0006).ok();
                if let Some(e) = va.emitters[0] {
                    audio.set_emitter_volume(e, sfx_vol);
                }
                va.emitters[3] = audio.create_emitter(0x4000_0006).ok();
                if let Some(e) = va.emitters[3] {
                    audio.set_emitter_distances(e, 10.0, 50.0);
                }
            }
            audio.stop_slot(va.engine);
            if let Some(e) = va.emitters[0] {
                if let Some(em) = audio.emitters.get_mut(e) {
                    em.distance_gain = 0.0;
                }
                let hash = if t.emergency {
                    audio.set_emitter_distances(e, 6.0, 40.0);
                    PASSBY_EMERGENCY
                } else {
                    audio.set_emitter_distances(e, 4.0, 30.0);
                    if wet { PASSBY_WET } else { PASSBY_DRY }
                };
                audio.play_event(e, hash, None, 256, Some(va.passby));
                if let Some(head) = audio.emitters.get(e).and_then(|x| x.samples) {
                    let v = audio.samples.get(head).map(|p| p.volume).unwrap_or(0.0) * tw.passby;
                    audio.set_sample_volume(head, v);
                }
            }
        }
        if va.active {
            va.update_emitters(audio, &t.car, view, engine_vol, tyres_vol, tw);
            if let Some(e) = va.emitters[0] {
                audio.set_emitter_volume(e, sfx_vol);
            }
            let e12 = va.emitters[3];
            if t.siren {
                if audio.slot(va.siren).is_none() {
                    if let Some(e) = e12 {
                        audio.play_event(e, FIRE_SIREN, None, 256, Some(va.siren));
                    }
                }
            } else if audio.slot(va.siren).is_some() {
                // the original stops the event but leaves its handle for the engine to clear
                if let Some(ev) = audio.slot(va.siren) {
                    audio.stop_event(Some(ev));
                }
            }
            if d2 >= 36.0 || t.ai_speed >= 5.0 || t.ai_speed <= 2.0 {
                if va.horn_pressed {
                    va.horn_pressed = false;
                    va.horn_played = false;
                }
            } else if !va.horn_pressed {
                if let Some(e) = e12 {
                    audio.play_event(e, HORN_PRESS, None, 0, None);
                }
                va.horn_pressed = true;
                if !va.horn_played {
                    if va.horn != u32::MAX {
                        if let Some(e) = e12 {
                            audio.play_event(e, va.horn, None, 0, None);
                        }
                    }
                    va.horn_played = true;
                }
            }
            if d2 < 36.0 && t.horn {
                if !va.horn_played {
                    if va.horn != u32::MAX {
                        if let Some(e) = e12 {
                            audio.play_event(e, va.horn, None, 0, None);
                        }
                    }
                    va.horn_played = true;
                }
            } else if va.horn_played {
                va.horn_played = false;
            }
        }
    }
    if release {
        va.release(audio);
    }
}

/// VsAudio::UpdatePolice (0x00054060) for one unit.
#[allow(clippy::too_many_arguments)]
pub fn update_police_car(
    audio: &mut AudioSystem,
    va: &mut VsVehicleAudio,
    p: &PoliceAudioInput,
    player_pos: Vec3,
    wet: bool,
    muted: bool,
    sfx_vol: f32,
    tw: &VehicleTweaks,
    view: i8,
    engine_vol: f32,
    tyres_vol: f32,
) {
    let mut release = false;
    if !p.car.active || muted {
        release = true;
    } else {
        let d2 = (player_pos - p.car.position).length_squared();
        if tw.activate_dist2 <= d2 || va.active {
            if d2 > tw.deactivate_dist2 && va.active {
                va.active = false;
                release = true;
            }
        } else {
            va.active = true;
            if va.emitters[0].is_none() {
                va.emitters[0] = audio.create_emitter(0x4000_0006).ok();
                if let Some(e) = va.emitters[0] {
                    if let Some(em) = audio.emitters.get_mut(e) {
                        em.distance_gain = 0.0;
                    }
                }
            }
            if let Some(e) = va.emitters[0] {
                audio.set_emitter_distances(e, 4.0, 30.0);
                audio.play_event(e, if wet { PASSBY_WET } else { PASSBY_DRY }, None, 256, Some(va.passby));
            }
        }
        if va.active {
            va.update_emitters(audio, &p.car, view, engine_vol, tyres_vol, tw);
            let Some(e0) = va.emitters[0] else { return };
            audio.set_emitter_volume(e0, sfx_vol);
            if p.siren {
                if va.siren_state <= 0 {
                    audio.stop_slot(va.siren);
                    let r = audio.rand() % 100;
                    va.siren_state = if r < 0 {
                        1
                    } else if r < 25 {
                        2
                    } else if r < 50 {
                        3
                    } else {
                        4
                    };
                }
                let (hash, times) = match va.siren_state {
                    1 => (SIREN_POLICE_SILENCE, tw.siren_times[0]),
                    2 => (SIREN_POLICE_02, tw.siren_times[1]),
                    3 => (SIREN_POLICE_03, tw.siren_times[2]),
                    _ => (SIREN_POLICE_01, tw.siren_times[3]),
                };
                if audio.slot(va.siren).is_none() {
                    audio.play_event(e0, hash, None, 0, Some(va.siren));
                    let span = ((times.1 - times.0) * 10.0) as i32;
                    let r = audio.rand();
                    let dur = (if span != 0 { r % span } else { 0 }) as f32 * f32::from_bits(0x3DCC_CCCD) + times.0;
                    if let Some(ps) = audio.slot_sample(va.siren) {
                        let v = audio.samples.get(ps).unwrap().volume;
                        audio.fade_sample_volume(ps, v, v, dur);
                    }
                }
                if let Some(ps) = audio.slot_sample(va.siren) {
                    let (v, ft) = audio.samples.get(ps).map(|x| (x.volume, x.fade_time)).unwrap();
                    if ft <= 0.0 {
                        audio.fade_sample_volume(ps, v, 0.0, f32::from_bits(0x3E4C_CCCD));
                        va.siren_state = -va.siren_state;
                    }
                }
            } else if audio.slot(va.siren).is_some() {
                audio.stop_slot(va.siren);
                va.siren_state = -1;
            }
        }
    }
    if release {
        va.release(audio);
    }
}

/// impact emitters round-robin and the recent-impact list (`dword_6D52FC`, VsAudio +508).
#[derive(Default)]
pub struct ImpactState {
    pub next: usize,
    pub history: [u32; 10],
    pub age: f32, // dword_6D4A4C (frames; reference frames here)
}

/// VsAudio.cpp 0x00052E90 PlayPhysicsImpact: an impact at `pos` near the player; `strength` 0..50.
#[allow(clippy::too_many_arguments)]
pub fn play_physics_impact(
    audio: &mut AudioSystem,
    st: &mut ImpactState,
    emitters: &[Option<EmitterId>; 6],
    player_pos: Vec3,
    hash: u32,
    pos: Vec3,
    strength: f32,
    only_car_layer: bool,
    sfx_vol: f32,
) {
    let d2 = (player_pos - pos).length_squared();
    if d2 >= 25.0 {
        return;
    }
    let t = (strength / 50.0).clamp(0.0, 1.0);
    let mut fired = false;
    // var_25: cleared by the object branch (0x00053042), so the car-layer replay below only runs for
    // `only_car_layer` calls; the history list is still filled
    let mut allowed = true;
    let e = emitters[st.next];
    if !only_car_layer {
        allowed = false;
        if let Some(e) = e {
            audio.stop_emitter_samples(e);
            audio.set_emitter_position(e, pos);
            audio.set_emitter_volume(e, sfx_vol * t);
            if audio.play_event(e, hash, None, 0, None) != 0 {
                log::debug!("Error Playing physics object audio event for impact sound");
            }
            if audio.play_event(e, IMPACT_CAR_PHYSICS, None, 0, None) != 0 {
                log::debug!("Error adding car audio event for impact sound");
            }
        }
        fired = true;
    }
    if d2 < 6.0 {
        let mut k = 0;
        while k < 10 && st.history[k] != 0 {
            if st.history[k] == hash {
                allowed = false;
                break;
            }
            k += 1;
        }
        if k == 10 {
            allowed = false;
        } else if st.history[k] == 0 {
            st.history[k] = hash;
        }
        if allowed {
            if let Some(e) = e {
                audio.stop_emitter_samples(e);
                audio.set_control_param(9, t);
                if audio.play_event(e, IMPACT_CAR_PHYSICS, None, 0, None) != 0 {
                    log::debug!("Error playing car audio event for impact sound");
                }
            }
            fired = true;
        }
    }
    if fired {
        st.next += 1;
        if st.next == 6 {
            st.next = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the per-frame steps give the same curve at 120 Hz (two half frames) as at 60 Hz, and the original at 60.
    #[test]
    fn per_frame_steps_are_rate_independent() {
        use crate::vs_audio::{leaky, per_frame};
        let k = f32::from_bits(0x3F33_3333);
        assert_eq!(leaky(2.0, 3.0, k, 1.0), (2.0 + 3.0) * k);
        let (mut a60, mut a120) = (0.0f32, 0.0f32);
        for _ in 0..30 {
            a60 = leaky(a60, 3.0, k, 1.0);
            a120 = leaky(leaky(a120, 3.0, k, 0.5), 3.0, k, 0.5);
        }
        assert!((a60 - a120).abs() < 0.02 * a60, "{a60} {a120}");
        let a = per_frame(0.4, 0.5);
        assert!(((1.0 - a) * (1.0 - a) - 0.6).abs() < 1e-6);
        assert_eq!(per_frame(0.4, 1.0), 0.4);
    }

    /// every vehicle database ai engine group (`VehicleAudio::audio`, as PhysicsManager passes LoadAiEngine) loads,
    /// and UpdateAiCars then starts and pitches its base-name event
    #[test]
    fn every_ai_engine_loads_and_plays() {
        let db = velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase::load().unwrap();
        let mut paths: Vec<String> = db.vehicles.iter().filter_map(|v| v.audio.audio.clone()).collect();
        paths.sort();
        paths.dedup();
        assert!(!paths.is_empty());
        let tw = VehicleTweaks::default();
        let (mut failed, mut no_event) = (Vec::new(), Vec::new());
        for path in &paths {
            let (mut audio, _mixer) = AudioSystem::with_default_mixer();
            if load_ai_engine(&mut audio, path) != 0 {
                failed.push(path.clone());
                continue;
            }
            let mut va = VsVehicleAudio::new(&mut audio);
            va.emitters[0] = Some(audio.create_emitter(4).unwrap());
            let ai = AiCarAudioInput {
                car: CarAudioInput { engine_rpm: 3000, active: true, ..Default::default() },
                engine_event: ai_engine_event(path),
                ai_rpm: None,
                present: true,
            };
            for _ in 0..3 {
                update_ai_car(&mut audio, &mut va, &ai, 6000, 1.0, &tw, 0, 0.45, 1.0);
                audio.update(1.0 / 60.0);
            }
            if audio.find_event(ai.engine_event) < 0 {
                let b = visos::file::platform::read(&format!("{path}.XVG")).unwrap();
                let (xg, _) = crate::audio::xvg::XvgGroup::from_bytes(&b).unwrap();
                let names: Vec<&str> = xg.events.iter().map(|e| e.name.as_str()).collect();
                no_event.push(format!("{path} {names:?}"));
                continue;
            }
            assert!(audio.slot(va.engine).is_some(), "{path}: engine loop not playing");
            eprintln!("{path}: event {:08X}", ai.engine_event);
        }
        eprintln!("no base-name event: {no_event:#?}");
        assert!(failed.is_empty(), "ai engine groups failing to load: {failed:?}");
        // data: v240sai0 holds event v200sai0, vc2vtai0 holds vsaxoai0 (those opponents are silent, original)
        assert_eq!(no_event.len(), 2, "{no_event:?}");
    }

    #[test]
    fn compiled_crcs() {
        assert_eq!(crc("IMP_SUSPENSION_SML"), IMP_SUSPENSION_SML);
        assert_eq!(crc("IMP_SUSPENSION_MED"), IMP_SUSPENSION_MED);
        assert_eq!(crc("IMP_SUSPENSION_LRG"), IMP_SUSPENSION_LRG);
        assert_eq!(crc("IMPACT_CAR_PHYSICS"), IMPACT_CAR_PHYSICS);
        assert_eq!(crc("SIREN_POLICE_SILENCE"), SIREN_POLICE_SILENCE);
        assert_eq!(crc("SIREN_POLICE_01"), SIREN_POLICE_01);
        assert_eq!(crc("SIREN_POLICE_02"), SIREN_POLICE_02);
        assert_eq!(crc("SIREN_POLICE_03"), SIREN_POLICE_03);
        assert_eq!(crc("RN_TARMAC_PASSBY_DRY_LIGHT"), PASSBY_DRY);
        assert_eq!(crc("HORN_CAR_2"), HORNS[1]);
    }

    #[test]
    fn surface_names_exist_in_the_surface_banks() {
        let (mut audio, _) = AudioSystem::with_default_mixer();
        for g in [r"\Audio\surface\tarmac\sftarm", r"\Audio\surface\tarmac\sftarmw", r"\Audio\surface\dirt\sfdirt",
                  r"\Audio\surface\gravel\sfgravl"] {
            audio.load_group(g, None).unwrap();
        }
        let mut va = VsVehicleAudio::new(&mut audio);
        for (surf, wet) in [(0, false), (0, true), (9, false), (13, false)] {
            va.surface = [surf, surf];
            va.build_surface_events(&mut audio, 0, wet);
            assert!(audio.find_event(va.brake[0]) >= 0, "brake surf {surf} wet {wet}");
            assert!(audio.find_event(va.drift_light[0]) >= 0, "drift surf {surf} wet {wet}");
            assert!(audio.find_event(va.drift_heavy[0]) >= 0);
        }
    }
}
