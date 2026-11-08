//! per-frame game state the audio code reads. the original reaches into the physics vehicle, the race/
//! traffic/police managers, the camera manager and the game options directly; the port receives the same
//! values in these structs (filled from ECS components by `plugin`). each field names the accessor it
//! replaces.

use bevy::math::Vec3;

/// one wheel (`VehicleEntity +240` wheel set)
#[derive(Clone, Debug, Default)]
pub struct WheelAudioInput {
    /// on the ground (`AUDIO_Resample__364BB0`, the wheel contact query)
    pub contact: bool,
    /// suspension compression (`AUDIO_Resample__3649D0`)
    pub compression: f32,
    /// contact patch slip velocity, world space (`AUDIO_Resample__36CCB0`)
    pub slip: Vec3,
}

/// a car's physical state.
#[derive(Clone, Debug, Default)]
pub struct CarAudioInput {
    /// `PhysicsCarInput::GetPosition` (0x001BA270)
    pub position: Vec3,
    /// `VehicleEntity::GetVelocity` (0x001C6480)
    pub velocity: Vec3,
    /// `VehicleEntity` up axis (0x001C6630)
    pub up: Vec3,
    /// `VehicleEntity` forward axis (0x001C66C0)
    pub forward: Vec3,
    /// speed in m/s (`VsAIRaceVehicle__24220`)
    pub speed: f32,
    /// engine rpm (`VsConfigFileRead__1B6F10`)
    pub engine_rpm: i32,
    /// gear (`VsConfigFileRead__1B74A0`); > 100 marks the off-load/neutral state the engine code tests
    pub gear: i32,
    /// throttle 0..1 (gearbox +36)
    pub throttle: f32,
    /// driving backwards (`VehicleEntity +3216 < 0`)
    pub reversing: bool,
    /// gearbox +44: suspension clunks listen to the front pair instead of the rear
    pub front_suspension: bool,
    pub wheels: [WheelAudioInput; 4],
    /// surface id under the left/right wheels (`VsVehicleAudio +40/+44`); 1 wet tarmac, 9/10 dirt, 11 grass,
    /// 12 deep gravel, 13 loose gravel, others tarmac
    pub surface: [u32; 2],
    /// the vehicle takes part in the simulation (`+28`)
    pub active: bool,
}

/// an ai racer (`VsAIDriver`)
#[derive(Clone, Debug, Default)]
pub struct AiCarAudioInput {
    pub car: CarAudioInput,
    /// hash of the ai engine group's base name (`ai +356`)
    pub engine_event: u32,
    /// the driver's own rpm when it is ai-driven (`VsAIPoliceVehicle__189E0`, gated by `+260`)
    pub ai_rpm: Option<f32>,
    /// the ai race vehicle pointer is set (always, for an opponent in `ai`)
    pub present: bool,
}

/// a traffic car
#[derive(Clone, Debug, Default)]
pub struct TrafficAudioInput {
    pub car: CarAudioInput,
    /// fire engine siren (`VsAITrafficVehicle__2B5D0`)
    pub siren: bool,
    /// lights flag `+4073`: emergency vehicle passby distances and event
    pub emergency: bool,
    /// `VsAIDriver__15C60` (speed used for the horn window 2..5)
    pub ai_speed: f32,
    /// `+320` horn wanted
    pub horn: bool,
}

/// a police unit
#[derive(Clone, Debug, Default)]
pub struct PoliceAudioInput {
    pub car: CarAudioInput,
    /// siren on (`+12`)
    pub siren: bool,
}

/// the active camera (`VsCameraManager`).
#[derive(Clone, Debug, Default)]
pub struct CameraAudioInput {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    /// the camera follows the player car (`VsCameraManager__5FFB0(0) == 1`): listener velocity = car's
    pub follows_player: bool,
    /// view index for the position volume offsets (0 normal, 1 far, 2 bumper, 3 bonnet)
    pub view: i8,
}

/// `g_GameOptions` audio settings (`dword_6F9164..6F917C`)
#[derive(Clone, Debug)]
pub struct AudioSettings {
    pub master: f32,
    pub music: f32,
    pub engine: f32,
    pub tyres: f32,
    pub speech: f32,
    pub sfx: f32,
    /// 0 mono, 1 stereo, 2 surround
    pub output_mode: u32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        // the defaults the globals carry before the options are applied (0x00684EE0..)
        Self { master: 1.0, music: 0.75, engine: 0.45, tyres: 1.0, speech: 1.0, sfx: 1.0, output_mode: 1 }
    }
}

/// everything `VsAudio::Update` reads from the rest of the game this frame. a bevy resource: gameplay
/// systems overwrite it before `AudioSystems::Update` runs.
#[derive(Clone, Debug, Default, bevy::prelude::Resource)]
pub struct GameAudioInput {
    pub camera: Option<CameraAudioInput>,
    pub player: Option<CarAudioInput>,
    pub ai: Vec<AiCarAudioInput>,
    pub traffic: Vec<TrafficAudioInput>,
    pub police: Vec<PoliceAudioInput>,
    /// the track is wet (`VsAudio::IsTrackWet`)
    pub track_wet: bool,
    /// a tft sequence is running (`VsAIPlayerTFTManager__172D0`)
    pub tft_active: bool,
}
