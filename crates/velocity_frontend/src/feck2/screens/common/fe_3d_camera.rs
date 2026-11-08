//! src\Feck2\Screens\Common\FE3DCamera.cpp — the frontend's 3D foyer camera (`gpFE3DCamera`, `dword_7691B8`):
//! which foyer is shown (`SetFoyer` 0x00272070, e.g. "hangout" from the main menu) and where the camera is,
//! from the position / look-at / up tables (static init 0x0042FA60 / 0x0042FBF0, 16-byte vectors), which the
//! constructor overwrites from `\FE\Cameras.txt` (0x0026F8B0).
//! The scene itself (foyer models, player car) is built on the bevy side (`foyer.rs`) from this state.
//!
//! Camera moves (`MoveTo` 0x00271D20 with move = 1) take "Move Duration" (`dword_6C1530` = 0.65 s);
//! `SetPosition` (0x00271D70) snaps. The ease is linear here (the original's spline is not ported).

/// `dword_7691E0` camera positions
pub const CAMERA_POS: [[f32; 3]; 11] = [
    [0.247, 2.941, -6.609],
    [0.247, 2.941, 6.609],
    [0.247, 2.941, 8.609],
    [0.247, 2.941, 10.609],
    [0.247, 2.941, 12.609],
    [3.037, 2.056, 3.225],
    [3.077, 2.138, 4.677],
    [3.037, 2.056, 6.225],
    [3.077, 2.138, 7.677],
    [-1.247, 2.941, 8.609],
    [0.247, 2.941, 12.609],
];
/// `dword_7692D0` look-at points
pub const CAMERA_LOOK_AT: [[f32; 3]; 11] = [
    [-2.268, 1.248, -16.99],
    [-2.268, 1.248, -4.99],
    [-2.268, 1.248, -2.99],
    [-2.268, 1.248, 0.99],
    [-2.268, 1.248, 2.99],
    [6.489, 2.056, 3.129],
    [6.529, 2.138, 4.58],
    [6.489, 2.056, 6.129],
    [6.529, 2.138, 7.58],
    [-2.268, 1.248, 2.99],
    [-2.268, 1.248, 2.99],
];
/// `dword_769390` up vectors (all +Y before `Cameras.txt`)
pub const CAMERA_UP: [f32; 3] = [0.0, 1.0, 0.0];
/// the +228 duration stand-in (`0x00272110` is not ported; = the "Move Duration" tweak's initial value)
pub const MOVE_DURATION: f32 = 0.65;
/// `byte_7691BE` "Frontend.Camera.Enable Duration Change" (bss 0)
pub static ENABLE_DURATION_CHANGE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `dword_6C1530` "Frontend.Camera.Move Duration" (0.65), f32 bits
pub static MOVE_DURATION_TWEAK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F26_6666);

/// FE3DCamera.cpp 0x00272020 — a starting move's duration (+224): +228, or "Move Duration" while "Enable Duration
/// Change" is on
pub fn move_duration(base: f32) -> f32 {
    use std::sync::atomic::Ordering::Relaxed;
    if ENABLE_DURATION_CHANGE.load(Relaxed) {
        f32::from_bits(MOVE_DURATION_TWEAK.load(Relaxed))
    } else {
        base
    }
}

/// the 3D backgrounds the camera element registers at construction (FE3DCamera.cpp 0x0026F3D0 -> 0x00273000):
/// model (`.xmf` stem) and texture pack per index; `byte_769380` (offset of 1..3) is (0,0,0) (static init 0x0042FA40)
pub const BACKGROUNDS: [(&str, &str); 8] = [
    (r"\Foyers\hangout\Model\hangout", r"\Foyers\hangout\Model\hangoutT.vcd"),
    (r"\Foyers\hangout\Model\LA", r"\Foyers\hangout\Model\hangoutT.vcd"),
    (r"\Foyers\hangout\Model\London", r"\Foyers\hangout\Model\hangoutT.vcd"),
    (r"\Foyers\hangout\Model\Tokyo", r"\Foyers\hangout\Model\hangoutT.vcd"),
    (r"\Foyers\modArea\Model\modArea", r"\Foyers\modArea\Model\modAreaT.vcd"),
    (r"\Foyers\carSel\Model\carSel", r"\Foyers\carSel\Model\carSelT.vcd"),
    (r"\Foyers\hangCE\Model\hangCE", r"\Foyers\hangCE\Model\hangCET.vcd"),
    (r"\Foyers\empty\Model\empty", r"\Foyers\empty\Model\emptyT.vcd"),
];
/// orbit: pitch limits `dword_6C1538` / `dword_6C153C`, stick rates `dword_6C1540` / `dword_6C1544`
pub const PITCH_MIN: f32 = f32::from_bits(0xBF4CCCCD);
pub const PITCH_MAX: f32 = f32::from_bits(0xBDCCCCCD);
pub const YAW_RATE: f32 = f32::from_bits(0xBDCCCCCD);
pub const PITCH_RATE: f32 = f32::from_bits(0xBCA3D70A);

#[derive(Clone, Debug)]
pub struct FoyerState {
    /// `SetFoyer` name (directory under `\Foyers`), None = no 3D scene
    pub name: Option<String>,
    /// area model shown with the foyer (1 LA, 2 London, 3 Tokyo)
    pub area: u32,
    pub from: usize,
    pub to: usize,
    pub t: f32, // 0..1 along the move
    /// camera element fov in degrees (vertical): 66 from `CElementCamera::Load` (0x00299770), changed by screens
    pub fov: f32,
    /// position / look-at set directly (`SetPosition` 0x00271D70 / `SetLookAt` 0x00271E50 with a vector, as the
    /// character cameras do) instead of a table entry
    pub custom: Option<([f32; 3], [f32; 3])>,
    /// background elements shown (`SetBackgroundVisible` 0x00273620)
    pub backgrounds: [bool; 8],
    /// +240 re-read the orbit from the current position before the next orbit step
    pub orbit_reinit: bool,
    /// +345 stick orbit: 0 off, 1 yaw only, 2 pitch only, 3 both
    pub orbit_lock: u8,
    pub orbit_dist: f32,  // +236
    pub orbit_yaw: f32,   // +256
    pub orbit_pitch: f32, // +260
    /// a move between given points (`SetPosition` +128 / `SetLookAt` +192 start, `MoveTo` +144 / +208 target),
    /// None when the camera sits on its target
    pub vmove: Option<VectorMove>,
    /// +228 move duration (`0x00272110`; `0x00272020` copies it to +224 when a move starts)
    pub move_time: f32,
    /// +244 move mode: 0 eased (sine), 1 linear
    pub move_mode: u32,
    /// +347 a move ends once less than a frame is left
    pub end_early: bool,
    /// the camera tables `dword_7691E0` / `dword_7692D0` / `dword_769390` (position, look-at, up; 11 entries),
    /// static init values until `load_cameras` reads `\FE\Cameras.txt`
    pub table: [[[f32; 3]; 3]; 11],
    /// +64 start / +80 target up of the vector moves (`0x00271DD0` / `0x00271DA0`); the camera's up (+48) is
    /// their normalised mix
    pub up_start: [f32; 3],
    pub up_target: [f32; 3],
    /// +264 / +268 the orbit's pitch clamp (`dword_6C1538` / `dword_6C153C` until `set_pitch_limits`)
    pub pitch_min: f32,
    pub pitch_max: f32,
    /// textures swapped onto background materials (FECK2Element3D 0x00298690), drawn by `foyer::foyer_material_textures`
    pub material_textures: Vec<MaterialTexture>,
    /// camera shake (0x00275E50): +388 total / +392 left / +396 x amplitude / +400 y amplitude / +404 follows the move
    pub shake_total: f32,
    pub shake_left: f32,
    pub shake_amp: [f32; 2],
    pub shake_follow: bool,
    /// the shake's current camera-space offset applied by `eye` (world vector)
    pub shake_off: [f32; 3],
    pub rand: visos::handy::crt_rand::CrtRand,
    /// MTV overlay calls (`MtvEffect::start` / `stop`) for `foyer::run_foyer` to apply
    pub mtv: Vec<MtvCall>,
}

/// a call on graphics' `MtvEffect` (LightStreak.cpp 0x00130310 / 0x00130340)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MtvCall {
    Start { seconds: f32, no_glitch: bool },
    Stop { immediate: bool },
}

/// one material texture swap of a background element: the model's shader `material` draws `texture` (loaded from
/// `vcd` when the texture manager does not hold it yet)
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialTexture {
    pub background: usize,
    pub material: String,
    pub texture: String,
    pub vcd: String,
}

/// one FE3DCamera move between two vectors (+128..+144 position, +192..+208 look-at, +224 time left)
#[derive(Clone, Debug)]
pub struct VectorMove {
    pub from: ([f32; 3], [f32; 3]),
    pub to: ([f32; 3], [f32; 3]),
    pub left: f32,
    /// the up mix of the move (0x00273C80..: `+48 = +64 + (+80 - +64) * s`, normalised)
    pub up: [f32; 3],
}

/// `CElementCamera::Load` (0x00299770 -> 0x00299860): fov 66, aspect 4:3, near 1, far 300
pub const FOV_DEFAULT: f32 = f32::from_bits(1115947008);
pub const NEAR: f32 = 1.0;
pub const FAR: f32 = f32::from_bits(1133903872);

impl Default for FoyerState {
    fn default() -> Self {
        Self {
            name: None,
            area: 0,
            from: 0,
            to: 0,
            t: 0.0,
            fov: FOV_DEFAULT,
            custom: None,
            backgrounds: [false; 8],
            orbit_reinit: false,
            orbit_lock: 0,
            orbit_dist: 0.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
            vmove: None,
            move_time: MOVE_DURATION,
            move_mode: 0,
            end_early: false,
            table: std::array::from_fn(|i| [CAMERA_POS[i], CAMERA_LOOK_AT[i], CAMERA_UP]),
            // 0x0026F3D0: VsVector4::Set(0, 1, 0, 1)
            up_start: CAMERA_UP,
            up_target: CAMERA_UP,
            pitch_min: PITCH_MIN,
            pitch_max: PITCH_MAX,
            material_textures: Vec::new(),
            shake_total: 0.0,
            shake_left: 0.0,
            shake_amp: [0.0; 2],
            shake_follow: false,
            shake_off: [0.0; 3],
            rand: Default::default(),
            mtv: Vec::new(),
        }
    }
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        v
    }
}

impl FoyerState {
    /// FE3DCamera.cpp 0x00272140 — `SetFOV(degrees)` on the camera element.
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    /// FE3DCamera.cpp 0x00271E80 — current = target (end any move).
    pub fn snap_to_target(&mut self) {
        self.from = self.to;
        self.t = 1.0;
    }

    /// FE3DCamera.cpp 0x00272070 — `SetFoyer`
    pub fn set_foyer(&mut self, name: &str) {
        if self.name.as_deref() != Some(name) {
            self.name = Some(name.to_string());
        }
        if self.area == 0 {
            self.area = 1;
        }
    }

    /// FE3DCamera.cpp 0x00271D70 — snap to a table entry.
    pub fn snap_to(&mut self, eye: [f32; 3], at: [f32; 3]) {
        self.custom = Some((eye, at));
        self.vmove = None;
    }

    /// table entry `index` as position, look-at and up (`0x00271D70` / `0x00271E50` / `0x00271DD0` and the
    /// target setters, as the screens' camera calls do, e.g. 0x00224EC0)
    pub fn snap(&mut self, index: usize) {
        self.custom = None;
        self.vmove = None;
        let i = index.min(self.table.len() - 1);
        self.from = i;
        self.to = i;
        self.t = 1.0;
        self.up_start = self.table[i][2];
        self.up_target = self.table[i][2];
    }

    /// FE3DCamera.cpp 0x00271D20 (move = 1) — move from the current place to a table entry.
    pub fn move_to(&mut self, index: usize) {
        self.custom = None;
        self.vmove = None;
        let i = index.min(self.table.len() - 1);
        if i == self.to && self.t >= 1.0 {
            return;
        }
        // restart from where the previous move has reached (approximated by its nearer end)
        self.from = if self.t >= 0.5 { self.to } else { self.from };
        self.to = i;
        self.t = 0.0;
        self.up_start = self.table[self.from][2];
        self.up_target = self.table[i][2];
    }

    /// FE3DCamera.cpp 0x0026F8B0 / 0x0026F3D0 — (from the constructor, which also registers `BACKGROUNDS`) `\FE\Cameras.txt`: 11 entries of CamPos,
    /// CamLookAt, CamAxis; the static tables stay when the file does not load
    pub fn load_cameras(&mut self) {
        use visos::handy::vs_text_file::VsTextFile;
        let Ok(dir) = visos::file::vs_file_directory::VsFileDirectoryDetails::open("\\FE") else { return };
        let mut t = VsTextFile::default();
        if !t.load(Some(&dir), "Cameras.txt") {
            return;
        }
        let f3 = |t: &mut VsTextFile| t.read_f32n::<3>().map_or([0.0; 3], |(_, v)| v);
        for i in 0..11 {
            let pos = f3(&mut t);
            let look = f3(&mut t);
            let up = f3(&mut t);
            self.table[i] = [pos, look, up];
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.t < 1.0 {
            self.t = (self.t + dt / move_duration(MOVE_DURATION)).min(1.0);
        }
        self.update_vector_move(dt);
        self.update_shake(dt);
    }

    /// FE3DCamera.cpp 0x00275E50 — shake the camera for `time` (negative: for the move time, until the move
    /// ends), random offset up to `amp_x` / `amp_y` (negative `amp_y`: = `amp_x`) fading out, with the MTV overlay
    pub fn shake(&mut self, time: f32, amp_x: f32, amp_y: f32) {
        let time = if time >= 0.0 {
            self.shake_follow = false;
            time
        } else {
            self.shake_follow = true;
            self.move_time
        };
        self.shake_total = time;
        self.shake_left = time;
        self.shake_amp[0] = amp_x;
        self.shake_amp[1] = if amp_y < 0.0 { amp_x } else { amp_y };
        self.mtv.push(MtvCall::Start { seconds: time, no_glitch: false });
    }

    /// FE3DCamera.cpp 0x00275F10 — per frame (FE3DCharacters::Update): the offset, camera-space x / y, is
    /// `-amp + rand(0, 2 * amp)` scaled by left / total; the move's end stops a following shake
    fn update_shake(&mut self, dt: f32) {
        if self.shake_left <= 0.0 {
            self.shake_follow = false;
            self.shake_off = [0.0; 3];
            return;
        }
        let moving = self.vmove.is_some() || self.t < 1.0; // +356 clear
        if !moving && self.shake_follow {
            self.shake_left = 0.0;
            self.shake_off = [0.0; 3];
            self.mtv.push(MtvCall::Stop { immediate: false });
            return;
        }
        let s = self.shake_left / self.shake_total;
        let mut r = |amp: f32| {
            let n = -amp;
            n + (self.rand.rand_unit() * (2.0 * amp as f64) ) as f32
        };
        let y = r(self.shake_amp[1]);
        let x = r(self.shake_amp[0]);
        let (eye, at, up) = self.eye_base();
        let fwd = normalise([at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]]);
        let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        let _ = up;
        let right = normalise(cross([0.0, 1.0, 0.0], fwd));
        let upv = normalise(cross(fwd, right));
        let (x, y) = (x * s, y * s);
        self.shake_off = [right[0] * x + upv[0] * y, right[1] * x + upv[1] * y, right[2] * x + upv[2] * y];
        self.shake_left -= dt;
    }

    /// FE3DCamera.cpp 0x00272020 + 0x00271D70 / 0x00271E50 / 0x00271D20 / 0x00271E00 — start a move from one
    /// position / look-at to another taking `move_time` (frame delta driven)
    pub fn start_vector_move(&mut self, from: ([f32; 3], [f32; 3]), to: ([f32; 3], [f32; 3])) {
        self.custom = Some(from);
        self.t = 1.0;
        self.vmove = Some(VectorMove { from, to, left: move_duration(self.move_time), up: normalise(self.up_start) });
    }

    /// FE3DCamera.cpp 0x00271DD0 — start up (+64)
    pub fn set_up(&mut self, up: [f32; 3]) {
        self.up_start = up;
    }

    /// FE3DCamera.cpp 0x00271DA0 — target up (+80)
    pub fn set_target_up(&mut self, up: [f32; 3]) {
        self.up_target = up;
    }

    /// +224 time left of the vector move (0 when none)
    pub fn move_left(&self) -> f32 {
        self.vmove.as_ref().map_or(0.0, |m| m.left)
    }

    /// FE3DCamera.cpp 0x00273C80.. (the camera element's Update, the +224 part): time left goes down by the frame
    /// delta; while moving the camera is `start + (target - start) * s`, s linear (mode 1) or sin(s * 90°) (mode 0),
    /// at the end it snaps to the target
    fn update_vector_move(&mut self, dt: f32) {
        let ups = (self.up_start, self.up_target);
        let Some(m) = self.vmove.as_mut() else { return };
        m.left -= dt;
        let moving = if self.end_early { m.left > dt } else { m.left > 0.0 };
        if moving {
            let mut s = 1.0 - (m.left * 1.0) / (self.move_time * 1.0);
            if self.move_mode == 0 || self.move_mode == 2 {
                s = (s * 90.0).to_radians().sin();
            }
            let l = |a: [f32; 3], b: [f32; 3]| [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s];
            self.custom = Some((l(m.from.0, m.to.0), l(m.from.1, m.to.1)));
            m.up = normalise(l(ups.0, ups.1));
        } else {
            self.custom = Some(m.to);
            self.vmove = None;
        }
    }

    /// FE3DCamera.cpp 0x00273620 — `SetBackgroundVisible(show, exclusive, index)`: exclusive hides the others
    /// FE3DCamera.cpp 0x0026F7A0 / FECK2Element3D.cpp 0x00298690 — background `index`'s element (asserts `index <=
    /// MAX_3D_BACKGROUNDS - 1`), then its model's shader `material` (FindShader) gets `texture` as stage 0 texture; a
    /// texture missing from the manager is loaded from `vcd` first. The swap lasts as long as the element.
    pub fn set_material_texture(&mut self, index: usize, material: &str, texture: &str, vcd: &str) {
        debug_assert!(index <= 7, "index <=MAX_3D_BACKGROUNDS - 1");
        let m = MaterialTexture { background: index, material: material.to_string(), texture: texture.to_string(), vcd: vcd.to_string() };
        match self.material_textures.iter_mut().find(|t| t.background == index && t.material == material) {
            Some(t) => *t = m,
            None => self.material_textures.push(m),
        }
    }

    pub fn show_background(&mut self, show: bool, exclusive: bool, index: usize) {
        if show && exclusive {
            for (i, b) in self.backgrounds.iter_mut().enumerate() {
                if i != index {
                    *b = false;
                }
            }
        }
        if let Some(b) = self.backgrounds.get_mut(index) {
            *b = show;
        }
    }

    /// FE3DCamera.cpp 0x00271CC0 / 0x00271CF0 then 0x00271E80 — position and look-at, current = target
    pub fn set_position_look_at(&mut self, eye: [f32; 3], at: [f32; 3]) {
        self.custom = Some((eye, at));
        self.t = 1.0;
    }

    /// FE3DCamera.cpp 0x00272260 — distance, yaw and pitch of the eye around the look-at point. The pitch is
    /// the y of the unit direction used as an angle, as in the original.
    pub fn orbit_init(&mut self) {
        let (eye, at, _) = self.eye_base();
        let d = [eye[0] - at[0], eye[1] - at[1], eye[2] - at[2]];
        let len2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        self.orbit_dist = len2.sqrt();
        let n = if len2 > 0.0 { [d[0] / self.orbit_dist, d[1] / self.orbit_dist, d[2] / self.orbit_dist] } else { d };
        // acos(+z . n), negative when (+z x n).y = n.x < 0
        self.orbit_yaw = n[2].clamp(-1.0, 1.0).acos();
        if n[0] < 0.0 {
            self.orbit_yaw = -self.orbit_yaw;
        }
        self.orbit_pitch = n[1];
        if self.orbit_pitch > 0.0 {
            self.orbit_pitch = -self.orbit_pitch;
        }
        self.orbit_reinit = false;
    }

    /// FE3DCamera.cpp 0x002727D0 — the orbit from a given eye and look-at: distance, yaw `acos(n.z)` (2π − yaw when
    /// n.x < 0), pitch `−asin(n.y)`
    pub fn orbit_from(&mut self, eye: [f32; 3], at: [f32; 3]) {
        let d = [eye[0] - at[0], eye[1] - at[1], eye[2] - at[2]];
        self.orbit_dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let n = if self.orbit_dist > 0.0 { d.map(|x| x / self.orbit_dist) } else { d };
        self.orbit_pitch = -n[1].clamp(-1.0, 1.0).asin();
        self.orbit_yaw = n[2].clamp(-1.0, 1.0).acos();
        if n[0] < 0.0 {
            self.orbit_yaw = f32::from_bits(0x40C90FDB) - self.orbit_yaw;
        }
    }

    /// FE3DCamera.cpp 0x002729D0 — `SetPitchLimits(lower, upper)`: +264 = −upper, +268 = −lower
    pub fn set_pitch_limits(&mut self, lower: f32, upper: f32) {
        debug_assert!(lower <= upper, "lower <= upper");
        self.pitch_min = 0.0 - upper;
        self.pitch_max = 0.0 - lower;
    }

    /// FE3DCamera.cpp 0x00272450 — orbit step: eye = look-at + (0,0,dist)·RotX(pitch)·RotY(yaw) with the angles
    /// from before this step, then the stick (`CameraRotateHoriz` / `Vert`, or the given amounts) turns them.
    pub fn orbit(&mut self, yaw_in: f32, pitch_in: f32, stick: (f32, f32)) {
        let (sy, cy) = self.orbit_yaw.sin_cos();
        let (sp, cp) = self.orbit_pitch.sin_cos();
        let (mut h, mut v) = stick;
        if self.orbit_lock == 1 {
            v = 0.0;
        } else if self.orbit_lock == 2 {
            h = 0.0;
        }
        if yaw_in != 0.0 {
            h = yaw_in;
        }
        if pitch_in != 0.0 {
            v = pitch_in;
        }
        if h.abs() < 1.0 {
            h = h * h * h;
        }
        if v.abs() < 1.0 {
            v = v * v * v;
        }
        self.orbit_yaw += h * YAW_RATE;
        // VsClamp(+260, +264, +268)
        self.orbit_pitch = (self.orbit_pitch + v * PITCH_RATE).max(self.pitch_min).min(self.pitch_max);
        let (_, at, _) = self.eye_base();
        let d = self.orbit_dist;
        let eye = [at[0] + d * (cp * sy), at[1] - d * sp, at[2] + d * (cp * cy)];
        self.custom = Some((eye, at));
    }

    /// FE3DCamera.cpp 0x00273AD0 / 0x00273B40 / 0x00273BB0 / 0x00273C80 — camera stick held: orbit when enabled
    /// and no move is running
    pub fn camera_held(&mut self, stick: (f32, f32)) {
        if self.orbit_lock == 0 || self.t < 1.0 {
            return;
        }
        if self.orbit_reinit {
            self.orbit_init();
        }
        self.orbit(0.0, 0.0, stick);
    }

    /// current eye, look-at, up (game space)
    pub fn eye(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let (e, a, u) = self.eye_base();
        let o = self.shake_off;
        ([e[0] + o[0], e[1] + o[1], e[2] + o[2]], [a[0] + o[0], a[1] + o[1], a[2] + o[2]], u)
    }

    /// `eye` without the shake
    fn eye_base(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let l = |a: [f32; 3], b: [f32; 3]| [a[0] + (b[0] - a[0]) * self.t, a[1] + (b[1] - a[1]) * self.t, a[2] + (b[2] - a[2]) * self.t];
        if let Some((e, a)) = self.custom {
            // +48: the move's mix, at rest the target up (0x00273C80.. end: `+48 = +80`, normalised)
            let up = self.vmove.as_ref().map_or_else(|| normalise(self.up_target), |m| m.up);
            return (e, a, up);
        }
        let (f, t) = (&self.table[self.from], &self.table[self.to]);
        (l(f[0], t[0]), l(f[1], t[1]), normalise(l(self.up_start, self.up_target)))
    }
}

/// the quick race screens' activate (0x00221C6D / 0x00223B3C / 0x002288A7): the hangout background alone, then
/// the selected track's area model (`dword_6F7144` +0x88, 1 = LA while no track is selected: the car select's logo wall
/// lives in that model; with 0 the hangout alone stayed up and the foyer rendered black)
pub fn show_hangout_and_track_area(fe: &mut crate::feck2::feck2_manager::Feck2) {
    fe.foyer.show_background(true, true, 0);
    let area = fe.quick_race.track.and_then(|t| fe.events.tracks.get(t)).map_or(1, |t| t.area as usize);
    fe.foyer.show_background(true, true, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_follows_the_move_and_stops_with_it() {
        let mut f = FoyerState::default();
        f.snap(0);
        f.move_to(1);
        f.shake(-1.0, 0.1, -1.0);
        assert_eq!(f.mtv, vec![MtvCall::Start { seconds: f.move_time, no_glitch: false }]);
        f.update(0.01);
        assert!(f.shake_off != [0.0; 3] && f.shake_off.iter().all(|v| v.abs() <= 0.1 * 1.01));
        f.t = 1.0; // move done
        f.update(0.01);
        assert_eq!(f.mtv.last(), Some(&MtvCall::Stop { immediate: false }));
        assert_eq!(f.shake_off, [0.0; 3]);
    }

    #[test]
    fn cameras_txt_overrides_the_tables() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut f = FoyerState::default();
        f.load_cameras();
        // entries 0..4 match the static init; 9 = MAINCAMERA_QUICKRACE_CARMODIICATION, axis (0, 1.1, 0)
        assert_eq!(f.table[0][0], CAMERA_POS[0]);
        assert_eq!(f.table[9][0], [3.197, 1.2, 24.404]);
        assert_eq!(f.table[9][2], [0.0, 1.1, 0.0]);
        f.snap(9);
        assert_eq!(f.eye().2, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn vector_move_mixes_the_up() {
        let mut f = FoyerState { move_time: 1.0, move_mode: 1, ..Default::default() };
        f.set_up([0.0, 1.0, 0.0]);
        f.set_target_up([1.0, 0.0, 0.0]);
        f.start_vector_move(([0.0; 3], [0.0, 0.0, 1.0]), ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]));
        f.update(0.5);
        let up = f.eye().2;
        assert!((up[0] - up[1]).abs() < 1e-6 && (up[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        f.update(0.6);
        assert_eq!(f.eye().2, [1.0, 0.0, 0.0]);
    }
}
