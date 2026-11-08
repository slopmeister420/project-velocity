//! src/Physics/Vehicle/VehicleEntity.cpp — the arcade car on top of the FastCar rigid-body model.

pub mod cx;
#[cfg(feature = "tr")]
pub mod tr;
#[cfg(feature = "tr")]
pub mod tr_ext;
#[cfg(feature = "tr")]
pub mod sim;
#[cfg(feature = "tr")]
pub mod sim_ai;
#[cfg(feature = "tr")]
pub mod ov;

use crate::c_config::{self, CLocalConfig};
use crate::vehicle::vs_config_file_read::{fmt_g, Defines};

/// Car names known to the config files (`off_688AA0`, 49 entries).
pub const CONFIG_CAR_NAMES: [&str; 49] = [
    "240SX", "MR2", "MINI", "SILVIA", "MX5", "C2", "CELICA", "ECLIPSE", "206GTI", "300ZX", "CIVIC", "CUPRA", "RX8",
    "MORANO", "S2000", "350Z", "RSX", "EVO8", "IMPREZA", "M3", "RX7", "SUPRA", "JAGXKR", "NSX", "VIPER", "MGSRV",
    "SKYLINE", "LA_FIRE01", "LON_BUS01", "LON_FIRE01", "SCHOOL_BUS", "4BY4", "CAR01", "CAR02", "COUPE01", "LON_4X4",
    "LON_CAR01", "LON_CAR02", "LON_CAR03", "LON_TAXI01", "LON_VAN01", "TAXI", "VAN", "VOLKSWAGON", "STATION01",
    "LON_EXEC01", "LON_TRUCK01", "COPLO", "COPLA",
];

/// Vehicles loaded with the `BUS` local config branch.
pub const BUS_NAMES: [&str; 5] = ["LA_FIRE01", "LON_BUS01", "LON_FIRE01", "SCHOOL_BUS", "LON_TRUCK01"];

/// Inputs of `VehicleEntity::LoadConfig` that the original takes from the entity.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalConfigInputs {
    /// vehicle database name (compared by name hash against [`CONFIG_CAR_NAMES`])
    pub name: String,
    /// `+0xFE8` bPLAYER
    pub is_player: bool,
    /// `byte_6F70E9`
    pub professional_race: bool,
    /// `+0xFEC..+0xFFC` wheel spans from the model locators / traffic setup
    pub span_front_back: f32,
    pub span_front: f32,
    pub span_back: f32,
    pub distance_to_front_wheels: f32,
    pub distance_to_rear_wheels: f32,
    /// `+0x70..+0x80` performance modifiers 0..1
    pub acceleration_pm: f32,
    pub top_speed_pm: f32,
    pub brake_pm: f32,
    pub tire_softness_pm: f32,
    pub body_roll_pm: f32,
    /// `+0x1000` traffic wheel radius override (0 = none)
    pub wheel_radius_override: f32,
    /// `+0x64` NOS modifier (SVehicleSetup::GetNosModifier)
    pub nos_pm: f32,
    /// `+0xFEA` a traffic setup made the car (traffic handling and collision rules)
    pub traffic: bool,
    /// the setup's automatic gearbox (`VehicleSetup` +0x18C; 0x001CABF0 calls 0x001DA8E0(VE, !automatic, 0):
    /// VE+0xDC9 = car+3289 blocks the automatic up-shift, the pad's gear up/down shift instead)
    pub automatic: bool,
}

impl LocalConfigInputs {
    /// VehicleEntity.cpp 0x001CABF0 — span defaults when neither model locators nor traffic data exist
    pub fn default_spans(name: &str) -> Self {
        let sfb = f32::from_bits(1076145160);
        let sf = f32::from_bits(1068708659);
        Self {
            name: name.into(),
            is_player: false,
            professional_race: false,
            span_front_back: sfb,
            span_front: sf,
            span_back: sf,
            distance_to_front_wheels: sfb * 0.5,
            distance_to_rear_wheels: sfb * 0.5,
            acceleration_pm: 0.0,
            top_speed_pm: 0.0,
            brake_pm: 0.0,
            tire_softness_pm: 0.0,
            body_roll_pm: 0.0,
            wheel_radius_override: 0.0,
            nos_pm: 0.0,
            traffic: false,
            automatic: true,
        }
    }

    /// VehicleEntity.cpp 0x001CABF0, the traffic branch (`a6[1]`): config by the traffic setup's name hash,
    /// spans from its half spans (doubled; the axle distances are half the front-back span), its wheel
    /// radius, no performance modifiers
    pub fn from_traffic(db: &crate::vehicle_database::vehicle_database::CVehicleDatabase, idx: usize) -> Self {
        let t = &db.traffic[idx];
        let mut s = Self::default_spans(t.name.as_deref().unwrap_or_default());
        s.span_front_back = t.half_wheel_span_front_back * 2.0;
        s.span_front = t.half_track * 2.0;
        s.span_back = t.half_track * 2.0;
        s.wheel_radius_override = t.wheel_radius;
        s.distance_to_front_wheels = s.span_front_back * 0.5;
        s.distance_to_rear_wheels = s.span_front_back * 0.5;
        s.traffic = true;
        s
    }

    /// VehicleEntity.cpp 0x001CABF0, the setup branch (`a6[0]`): config by the vehicle entry's name hash and the
    /// setup's performance modifiers; spans from the chassis model's wheel locators (`spans_from_vehicle_model`)
    pub fn from_setup(db: &crate::vehicle_database::vehicle_database::CVehicleDatabase,
        setup: &crate::vehicle_database::vehicle_database::VehicleSetup) -> Self {
        let name = setup.vehicle.and_then(|v| db.vehicles.get(v)).and_then(|v| v.name.clone()).unwrap_or_default();
        let mut s = Self::default_spans(&name);
        let pm = setup.performance_modifiers(db);
        s.acceleration_pm = pm.acceleration;
        s.top_speed_pm = pm.top_speed;
        s.nos_pm = pm.nos;
        s.tire_softness_pm = pm.tire_softness;
        s.body_roll_pm = pm.body_roll;
        s.brake_pm = pm.brake;
        s.automatic = setup.unk_18c != 0;
        s.spans_from_vehicle_model(db, setup);
        s
    }

    /// VehicleEntity.cpp 0x001CABF0 — spans derived from the f_l_wheel / r_l_wheel locators
    /// (`x` lateral, `z` forward in model space, as read through `Armco__12F50`/`12FF0`).
    pub fn spans_from_locators(&mut self, front_left: [f32; 3], rear_left: [f32; 3]) {
        let fz = 0.0 - front_left[2];
        let rz = 0.0 - rear_left[2];
        self.span_front_back = fz - rz;
        self.span_front = front_left[0] + front_left[0];
        self.span_back = rear_left[0] + rear_left[0];
        self.distance_to_front_wheels = fz;
        self.distance_to_rear_wheels = 0.0 - rz;
    }

    /// VehicleEntity.cpp 0x001CABF0 — the locator branch for a database car: the LOD 0 chassis `<name>_base` of
    /// `<path>\vehicleM.vcd` (0x001C0D60 -> race vehicle +1092) when it has both `f_l_wheel` and
    /// `r_l_wheel` (row 3 of each matrix, 0x0005C820). A model without them (shipped: COPTO, HIN01) takes the
    /// "ill-chosen defaults" branch (vehicle entry +0x38 is always set): span 1.0, tracks 2.0, axles 0.5.
    /// Returns true when the locators were used; spans stay unchanged when the pack cannot be read.
    pub fn spans_from_vehicle_model(&mut self, db: &crate::vehicle_database::vehicle_database::CVehicleDatabase,
        setup: &crate::vehicle_database::vehicle_database::VehicleSetup) -> bool {
        let Some(v) = setup.vehicle.and_then(|v| db.vehicles.get(v)) else { return false };
        let Some(name) = v.name.as_deref() else { return false };
        let Ok(vcd) = visos::file::Vcd::open(&format!("{}\\vehicleM.vcd", v.path)) else {
            return false;
        };
        let xmf = vcd.get(&format!("{name}_base.xmf"));
        match xmf.map(|x| (xmf_locator(x, "f_l_wheel"), xmf_locator(x, "r_l_wheel"))) {
            Some((Some(f), Some(r))) => {
                self.spans_from_locators([f[12], f[13], f[14]], [r[12], r[13], r[14]]);
                true
            }
            _ => {
                log::warn!("WARNING: Vehicle with no wheel locators.  Using ill-chosen defaults");
                self.span_front_back = 1.0;
                self.span_front = 2.0;
                self.span_back = 2.0;
                self.distance_to_front_wheels = 0.5;
                self.distance_to_rear_wheels = 0.5;
                false
            }
        }
    }
}

/// GameDataLoading.cpp 0x000A3CB0 — a model's locator matrix by name (header +12 flags bit 0 = has locators,
/// +52 -> {count, matrices, name offsets[]}, all offsets from the file start)
pub fn xmf_locator(b: &[u8], name: &str) -> Option<[f32; 16]> {
    let rd = |o: usize| b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize);
    if rd(12)? & 1 == 0 {
        return None;
    }
    let t = rd(52)?;
    let (n, mats) = (rd(t)?, rd(t + 4)?);
    for i in 0..n {
        let at = rd(t + 8 + 4 * i)?;
        let s = b.get(at..)?;
        let len = s.iter().position(|&c| c == 0)?;
        if &s[..len] == name.as_bytes() {
            let mut m = [0f32; 16];
            for (k, x) in m.iter_mut().enumerate() {
                *x = f32::from_bits(rd(mats + 64 * i + 4 * k)? as u32);
            }
            return Some(m);
        }
    }
    None
}

/// Local-config load result together with the facts the loader reports back to the entity.
#[derive(Clone, Debug)]
pub struct LoadedLocalConfig {
    pub lc: CLocalConfig,
    /// `+0xFE9` vehicle uses the BUS branch
    pub is_bus: bool,
    /// matched entry of [`CONFIG_CAR_NAMES`] (`+0x20`)
    pub config_name: Option<&'static str>,
    /// defines left after loading (`DRIVING_WHEELS` etc. are printed from it)
    pub defines: Defines,
}

fn name_hash(s: &str) -> u32 {
    crate::vehicle_database::string_hash(s)
}

/// VehicleEntity.cpp 0x001C9070 — builds the define set for this car and loads `Main.cfg`
/// (numbered variant `n`) into a fresh `CLocalConfig`.
pub fn load_local_config(inp: &LocalConfigInputs, main: &str, n: i32) -> (bool, LoadedLocalConfig) {
    load_local_config_into(inp, main, n, CLocalConfig::default())
}

/// VehicleEntity.cpp 0x001C9070 — the same load on top of an existing config (the original's VE+3552 object
/// persists between the `Main.cfg` and `Mainsafe.cfg` attempts)
fn load_local_config_into(inp: &LocalConfigInputs, main: &str, n: i32, lc: CLocalConfig) -> (bool, LoadedLocalConfig) {
    let mut lc = lc;
    let mut d = Defines::new();
    c_config::local_common_defines(&mut d, inp.professional_race);
    d.define("DEFAULT", "");
    d.define("NOT_BUS", "");
    for nm in CONFIG_CAR_NAMES {
        d.define(nm, "//");
    }
    d.define("BUS", "//");
    let g = |v: f32| fmt_g(v as f64, 6);
    d.define("SPAN_FRONT_BACK_FROM_XML", &g(inp.span_front_back));
    d.define("SPAN_FRONT_FROM_XML", &g(inp.span_front));
    d.define("SPAN_BACK_FROM_XML", &g(inp.span_back));
    d.define("DISTANCE_TO_FRONT_WHEELS_FROM_XML", &g(inp.distance_to_front_wheels));
    d.define("DISTANCE_TO_REAR_WHEELS_FROM_XML", &g(inp.distance_to_rear_wheels));
    d.define("m_fAcceleration_PM", &g(inp.acceleration_pm));
    d.define("m_fTop_speed_PM", &g(inp.top_speed_pm));
    d.define("m_Brake_PM", &g(inp.brake_pm));
    d.define("m_fTire_Softness_PM", &g(inp.tire_softness_pm));
    d.define("m_fBody_roll_PM", &g(inp.body_roll_pm));
    let h = name_hash(&inp.name);
    let mut config_name = None;
    for nm in CONFIG_CAR_NAMES {
        if name_hash(nm) == h {
            d.define(nm, "");
            log::debug!(" Trying to load the config file for the car: {}", nm);
            d.define("DEFAULT", "//");
            config_name = Some(nm);
        }
    }
    let mut is_bus = false;
    if let Some(bus) = BUS_NAMES.iter().find(|b| name_hash(b) == h) {
        d.define("BUS", "");
        d.define("NOT_BUS", "//");
        d.define("DEFAULT", "//");
        log::debug!(" Loading LOCAL config for bus: {}", bus);
        is_bus = true;
    } else if config_name.is_none() {
        log::debug!(" Loading LOCAL config for car: DEFAULT, NOT_BUS");
    }
    d.define("bPLAYER", if inp.is_player { "1" } else { "0" });
    let ok = c_config::load_main(main, n, &mut d, |path, defs| lc.load(path, defs).is_ok());
    if inp.wheel_radius_override != 0.0 {
        lc.front_wheel_radius = inp.wheel_radius_override;
        lc.back_wheel_radius = inp.wheel_radius_override;
    }
    (ok, LoadedLocalConfig { lc, is_bus, config_name, defines: d })
}

/// `Main.cfg` then `Mainsafe.cfg` as in `VehicleEntity::Create` (0x001CABF0).
pub fn load_local_config_with_fallback(inp: &LocalConfigInputs, n: i32) -> LoadedLocalConfig {
    let (ok, l) = load_local_config(inp, "Main.cfg", n);
    if ok {
        return l;
    }
    // 0x001CABF0: Mainsafe.cfg into the same object; it is not shipped, so a partial Main.cfg load (C2: no
    // CarBoxes.cfg entry) keeps what it read
    load_local_config_into(inp, "Mainsafe.cfg", 0, l.lc).1
}
