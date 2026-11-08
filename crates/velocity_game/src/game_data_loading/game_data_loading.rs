//! src\GameDataLoading\GameDataLoading.cpp — what a race loads, in order, and the race's `eventdef.txt`.
//!
//! paths come from the event: `track_path` = `CEventTrack+0x78` (`Tracks\LACOAST\`), `race_path` = `CEventRace+0x10`
//! (`CIRCUIT\RACE01\`); the race directory is `\` + track_path + race_path.

use visos::file::vs_file_directory::VsFileDirectoryDetails;
use visos::handy::vs_text_file::VsTextFile;

/// one load step of the original init sequences. `label` is the loading-bar / timer string the original passes to
/// `GameManager::LoadingTimer` (0x000B8C10) or `MEM_SetNameSpace`, `owner` the crate that implements it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ELoadStep {
    // GameManager.cpp 0x000B5510 FrontEndInit
    FeLoadStrings,
    FeOpenVcds,
    FeCarPalette,
    // GameManager.cpp 0x000B4F80 CommonInit (front end and in game)
    CommonVisosInit,
    GlobalMaterialInputResolver,
    FullScreenEffects,
    OpenFileDirectories,
    GameShaderProgramManager,
    InitCharacterData,
    AdditionalShaderData,
    CarShaderManager,
    LoadLevelSetup,
    AudioInit,
    FontHelperInit,
    GlowInit,
    FogInit,
    TrackFxInit,
    FlareInit,
    FadeRenderInit,
    ReflectionInit,
    OsdSubtitleInit,
    LicensePlateRenderInit,
    // FrontEndInit, after CommonInit
    GameMenuManagerInit,
    FrontEndInitViews,
    FeFonts,
    FeCarRender,
    FeDofRender,
    MtvEffects,
    FeParticleEffects,
    FeSkidSmoke,
    FrontendData,
    FeShadowRender,
    FeGameBridge,
    FeCloseVcds,
    // GameManager.cpp 0x000B5F80 InGameInit
    InGameStrings,
    CarInitialisation,
    OsdManager,
    InGameFonts,
    MessageManagerInit,
    Tweaks,
    CompartmentPreInit,
    ProjectedLightBeams,
    CatsEyesRenderInit,
    TimelineManagerInit,
    RaceTimelineManagerInit,
    // GameDataLoading.cpp 0x0009CEC0 LoadGameData
    EventDefinition,
    LevelArena,
    LevelTriggers,
    LoadLighting,
    CompartmentsOpenIndex,
    CatsEye,
    LevelSounds,
    CommonCarTextures,
    FlagGirlTimeline,
    MarkersInit,
    // InGameInit continued
    InitViews,
    PreloadParticleTextures,
    StatelessParticleTypes,
    ParticleManagerInit,
    InitGameViews,
    ShadowRenderInit,
    SkidMarkRender,
    MotionBlur,
    SparkRender,
    SlipStreamRender,
    RainRender,
    DofRenderInit,
    CMeshRender,
    SkyRender,
    GameMenuManagerInGame,
    Load3dInterface,
    PauseOsdInit,
    AiWorldInit,
    PhysicsManagerInit,
    PhysicsObjectParticleTracker,
    AiWorldData,
    TftManager,
    OsdMap,
    CameraManager,
    FreeCameras,
    Interface3dCamera,
    CrashCam,
    FlyCameras,
    Barriers,
    NetworkCableScreen,
    CarRender,
    FuryFx,
    SkidSmoke,
    PuddleRender,
    ReplayCameraManager,
    TyreSprayRender,
    LensSprayRender,
    HeatHazeInit,
    SkinnedObjectRenderInit,
    WaveDistortInit,
    LightStreaksInit,
    StartLineEffects,
    TrailblazerEffects,
    ProfilerOsd,
    Bridge,
    CommonTextureCompartment,
    InstanceCompartment,
    WaitForCompartmentSet,
    LoadTimelineData,
    PostRaceInit,
    InGameInitEnded,
}

/// crate responsible for a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EOwner {
    Core,
    Graphics,
    Physics,
    Audio,
    Ai,
    Frontend,
}

impl ELoadStep {
    /// (label, owner)
    pub fn info(self) -> (&'static str, EOwner) {
        use EOwner::*;
        use ELoadStep::*;
        match self {
            FeLoadStrings => ("FEInit Load Strings ", Frontend),
            FeOpenVcds => ("FEInit Open VCD's ", Frontend),
            FeCarPalette => ("FEInit Init File Dirs ", Graphics),
            CommonVisosInit => ("CommonInit:: Common Visos Init", Graphics),
            GlobalMaterialInputResolver => ("Global material input resolver Init", Graphics),
            FullScreenEffects => ("Full Screen Effects Manager Init", Graphics),
            OpenFileDirectories => ("CommonInit::Open File Directories", Core),
            GameShaderProgramManager => ("CommonInit::Game Shader Manager", Graphics),
            InitCharacterData => ("CommonInit::Init Character Data", Graphics),
            AdditionalShaderData => ("CommonInit::Additional Shader Data", Graphics),
            CarShaderManager => ("CommonInit::Car Shader Manager", Graphics),
            LoadLevelSetup => ("CommonInit::Load Level Setup", Graphics),
            AudioInit => ("CommonInit::Audio Init", Audio),
            FontHelperInit => ("CommonInit::FontHelper Init", Frontend),
            GlowInit => ("CommonInit::Glow Init", Graphics),
            FogInit => ("CommonInit::Fog Init", Graphics),
            TrackFxInit => ("CommonInit::Track Fx", Graphics),
            FlareInit => ("CommonInit::Flare Init", Graphics),
            FadeRenderInit => ("CommonInit::Fade Render Init", Graphics),
            ReflectionInit => ("CommonInit::Reflection Init", Graphics),
            OsdSubtitleInit => ("CommonInit::OSDSubtitle Init", Frontend),
            LicensePlateRenderInit => ("CommonInit::License Plate Render Init", Graphics),
            GameMenuManagerInit => ("Game Menu Manger Init", Frontend),
            FrontEndInitViews => ("FrontEndInitViews()", Graphics),
            FeFonts => ("Fonts", Frontend),
            FeCarRender => ("CarRender", Graphics),
            FeDofRender => ("DOFRender", Graphics),
            MtvEffects => ("MTV Effects", Graphics),
            FeParticleEffects => ("Particle Effects", Graphics),
            FeSkidSmoke => (" SKID Smoke", Graphics),
            FrontendData => ("Frontend Data", Frontend),
            FeShadowRender => ("ShadowRenderInit", Graphics),
            FeGameBridge => ("GameBridge", Core),
            FeCloseVcds => ("Close VCDs", Core),
            InGameStrings => ("In Game Strings", Frontend),
            CarInitialisation => ("Car Initialisation", Graphics),
            OsdManager => ("OSD Manager", Frontend),
            InGameFonts => ("Fonts", Frontend),
            MessageManagerInit => ("Message Manager Init", Core),
            Tweaks => ("Tweaks", Core),
            CompartmentPreInit => ("Compartment Manager Pre Init", Graphics),
            ProjectedLightBeams => ("ProjectedLightBeams", Graphics),
            CatsEyesRenderInit => ("Cats Eyes", Graphics),
            TimelineManagerInit => ("New Timeline Manager Init", Core),
            RaceTimelineManagerInit => ("Race Timeline Manager Init", Core),
            EventDefinition => ("Event Definition", Core),
            LevelArena => ("Load Arena and SkyBox", Graphics),
            LevelTriggers => ("Triggers Init", Core),
            LoadLighting => ("Load Lighting", Graphics),
            CompartmentsOpenIndex => ("Compartments openVCDIndex", Graphics),
            CatsEye => ("Load catseyes", Graphics),
            LevelSounds => ("Load Level Sounds", Audio),
            CommonCarTextures => ("Load Common Car Textures", Graphics),
            FlagGirlTimeline => ("Flag Girl Timeline", Core),
            MarkersInit => ("Markers Init", Core),
            InitViews => ("Init Views", Frontend),
            PreloadParticleTextures => ("Preload particle textures", Graphics),
            StatelessParticleTypes => ("Stateless Particle Types", Graphics),
            ParticleManagerInit => ("Particle Init", Graphics),
            InitGameViews => ("Init Game Views", Core),
            ShadowRenderInit => ("ShadowRenderInit", Graphics),
            SkidMarkRender => ("SkidMark Render", Graphics),
            MotionBlur => ("Motion Blur", Graphics),
            SparkRender => ("Spark Render", Graphics),
            SlipStreamRender => ("Slip Stream Render", Graphics),
            RainRender => ("Rain Render", Graphics),
            DofRenderInit => ("DOF Render Init", Graphics),
            CMeshRender => ("CMesh Render", Graphics),
            SkyRender => ("Sky Render", Graphics),
            GameMenuManagerInGame => ("Game Menu Manger Init", Frontend),
            Load3dInterface => ("Load 3D Interface", Frontend),
            PauseOsdInit => ("Pause OSD Init", Frontend),
            AiWorldInit => ("Ai World", Ai),
            PhysicsManagerInit => ("Physics Manager", Physics),
            PhysicsObjectParticleTracker => ("PhysicsObjectParticleTracker Init", Physics),
            AiWorldData => ("AI World Data", Ai),
            TftManager => ("TFT Manager", Ai),
            OsdMap => ("Generating OSD Map", Frontend),
            CameraManager => ("Camera Manager", Core),
            FreeCameras => ("Free Cameras", Core),
            Interface3dCamera => ("3d Interface", Core),
            CrashCam => ("Crash Cam", Core),
            FlyCameras => ("Fly Cameras", Core),
            Barriers => ("Barriers", Graphics),
            NetworkCableScreen => ("Audio System", Frontend),
            CarRender => ("Car Render", Graphics),
            FuryFx => ("Fury FX", Graphics),
            SkidSmoke => ("Skid Smoke", Graphics),
            PuddleRender => ("Puddle Render Init", Graphics),
            ReplayCameraManager => ("Replay Camera Manager", Core),
            TyreSprayRender => ("Tyre Spray Render", Graphics),
            LensSprayRender => ("Lens Spray Render", Graphics),
            HeatHazeInit => ("HeatHaze Init", Graphics),
            SkinnedObjectRenderInit => ("SkinnedObjectRender Init", Graphics),
            WaveDistortInit => ("WaveDistort Init", Graphics),
            LightStreaksInit => ("Light Streaks Init", Graphics),
            StartLineEffects => ("StartLineEffects", Graphics),
            TrailblazerEffects => ("Trailblazer Effects", Graphics),
            ProfilerOsd => ("Profiler OSD", Core),
            Bridge => ("Bridge", Core),
            CommonTextureCompartment => ("Common Texture Compartment", Graphics),
            InstanceCompartment => ("Instance Compartment", Graphics),
            WaitForCompartmentSet => ("Waiting for Compartment Set", Graphics),
            LoadTimelineData => ("Timelines", Core),
            PostRaceInit => ("Post Race Init", Core),
            InGameInitEnded => ("In Game Init Ended", Core),
        }
    }

    pub fn label(self) -> &'static str {
        self.info().0
    }

    pub fn owner(self) -> EOwner {
        self.info().1
    }
}

/// GameManager.cpp 0x000B4F80 — `front_end` is the original's argument (no level setup in the front end).
pub fn common_init_steps(front_end: bool) -> Vec<ELoadStep> {
    use ELoadStep::*;
    let mut v = vec![
        CommonVisosInit,
        GlobalMaterialInputResolver,
        FullScreenEffects,
        OpenFileDirectories,
        GameShaderProgramManager,
        InitCharacterData,
        AdditionalShaderData,
        CarShaderManager,
    ];
    if !front_end {
        v.push(LoadLevelSetup);
    }
    v.extend([
        AudioInit,
        FontHelperInit,
        GlowInit,
        FogInit,
        TrackFxInit,
        FlareInit,
        FadeRenderInit,
        ReflectionInit,
        OsdSubtitleInit,
        LicensePlateRenderInit,
    ]);
    v
}

/// GameManager.cpp 0x000B5510
pub fn front_end_init_steps() -> Vec<ELoadStep> {
    use ELoadStep::*;
    let mut v = vec![FeLoadStrings, FeOpenVcds, FeCarPalette];
    v.extend(common_init_steps(true));
    v.extend([
        GameMenuManagerInit,
        FrontEndInitViews,
        FeFonts,
        FeCarRender,
        FeDofRender,
        MtvEffects,
        FeParticleEffects,
        FeSkidSmoke,
        FrontendData,
        FeShadowRender,
        FeGameBridge,
        FeCloseVcds,
    ]);
    v
}

/// GameDataLoading.cpp 0x0009CEC0 — the flag girl timeline is skipped for cruise races.
pub fn load_game_data_steps(race_path: &str) -> Vec<ELoadStep> {
    use ELoadStep::*;
    let mut v = vec![
        EventDefinition,
        LevelArena,
        LevelTriggers,
        LoadLighting,
        CompartmentsOpenIndex,
        CatsEye,
        LevelSounds,
        CommonCarTextures,
    ];
    if !race_path.to_ascii_lowercase().contains("cruise") {
        v.push(FlagGirlTimeline);
    }
    v.push(MarkersInit);
    v
}

/// GameManager.cpp 0x000B5F80 — the whole in-game init, LoadGameData included.
pub fn in_game_init_steps(race_path: &str, network: bool, has_osd_map: bool) -> Vec<ELoadStep> {
    use ELoadStep::*;
    let mut v = vec![InGameStrings, CarInitialisation];
    v.extend(common_init_steps(false));
    v.extend([
        OsdManager,
        InGameFonts,
        MessageManagerInit,
        Tweaks,
        CompartmentPreInit,
        ProjectedLightBeams,
        CatsEyesRenderInit,
        TimelineManagerInit,
        RaceTimelineManagerInit,
    ]);
    v.extend(load_game_data_steps(race_path));
    v.extend([
        InitViews,
        PreloadParticleTextures,
        StatelessParticleTypes,
        ParticleManagerInit,
        InitGameViews,
        ShadowRenderInit,
        SkidMarkRender,
        MotionBlur,
        SparkRender,
        SlipStreamRender,
        RainRender,
        DofRenderInit,
        CMeshRender,
        SkyRender,
        GameMenuManagerInGame,
        Load3dInterface,
        PauseOsdInit,
        AiWorldInit,
        PhysicsManagerInit,
        PhysicsObjectParticleTracker,
        AiWorldData,
    ]);
    if !network {
        v.push(TftManager);
    }
    if has_osd_map {
        v.push(OsdMap);
    }
    v.extend([
        CameraManager,
        FreeCameras,
        Interface3dCamera,
        CrashCam,
        FlyCameras,
        Barriers,
        NetworkCableScreen,
        CarRender,
        FuryFx,
        SkidSmoke,
        PuddleRender,
        ReplayCameraManager,
        TyreSprayRender,
        LensSprayRender,
        HeatHazeInit,
        SkinnedObjectRenderInit,
        WaveDistortInit,
        LightStreaksInit,
        StartLineEffects,
        TrailblazerEffects,
        ProfilerOsd,
        Bridge,
        CommonTextureCompartment,
        InstanceCompartment,
        WaitForCompartmentSet,
        LoadTimelineData,
        PostRaceInit,
        InGameInitEnded,
    ]);
    v
}

/// the game paths a race reads (from the loaders the steps call), for the owners of each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RacePaths {
    /// `\Tracks\LACOAST\`
    pub track_dir: String,
    /// `\Tracks\LACOAST\CIRCUIT\RACE01\`
    pub race_dir: String,
}

impl RacePaths {
    pub fn new(track_path: &str, race_path: &str) -> Self {
        RacePaths { track_dir: format!("\\{}", track_path), race_dir: format!("\\{}{}", track_path, race_path) }
    }
    /// GameDataLoading.cpp 0x000A0F30
    pub fn arena_t(&self) -> String {
        format!("{}arenaT.vcd", self.track_dir)
    }
    pub fn arena_m(&self) -> String {
        format!("{}arenaM.vcd", self.track_dir)
    }
    /// GameDataLoading.cpp 0x0009CEC0 — compartment index packs
    pub fn cpts(&self) -> String {
        format!("{}cpts.vcd", self.track_dir)
    }
    pub fn race_vcd(&self) -> String {
        format!("{}Race.vcd", self.race_dir)
    }
    /// TimelineTrigger.cpp 0x0017F980 — track-wide and race triggers
    pub fn triggers(&self) -> [String; 2] {
        [format!("{}triggers.trg", self.track_dir), format!("{}triggers.trg", self.race_dir)]
    }
    /// ReplayCamera_Fixed.cpp 0x00165400
    pub fn cameras(&self) -> String {
        format!("{}cameras.cam", self.race_dir)
    }
    /// VibrateEffects.cpp 0x001EDEF0 (built from the track only)
    pub fn markers(&self) -> String {
        format!("{}Markers.mfk", self.track_dir)
    }
    /// RaceTimelineSequencer.cpp 0x00170570
    pub fn timeline(&self) -> String {
        format!("{}timeline.bun", self.race_dir)
    }
    /// StringManager.cpp 0x000C9890 (heat haze, race then track)
    pub fn heat_haze(&self) -> [String; 2] {
        [format!("{}heathaze.bdl", self.race_dir), format!("{}heathaze.bdl", self.track_dir)]
    }
    pub fn skin_inst(&self) -> String {
        format!("{}SkinInst.bdl", self.race_dir)
    }
    /// GameDataLoading.cpp 0x0009E300 (wet weather picks SetupWET.txt)
    pub fn setup(&self, wet: bool) -> String {
        format!("{}{}", self.track_dir, if wet { "SetupWET.txt" } else { "Setup.txt" })
    }
    pub fn event_def(&self) -> String {
        format!("{}eventdef.txt", self.race_dir)
    }
}

/// parsed `eventdef.txt` (GameDataLoading.cpp 0x000A2EF0), in game coordinates (z negated).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventDefinition {
    pub name: String,
    pub event_type: i32,
    pub num_laps: i32,
    /// opponents after the split-screen adjustment
    pub num_opponents: i32,
    pub opponent_vehicles: Vec<String>,
    /// two colours per car (player cars use the first block)
    pub colours: Vec<([f32; 3], [f32; 3])>,
    /// start positions reordered by the player start table, z negated, pulled to 3.5 m behind the start line
    pub start_positions: Vec<[f32; 3]>,
    pub start_direction: f32,
    pub start_line: [[f32; 3]; 2],
    pub finish_line: [[f32; 3]; 2],
    pub unknown_line_point: [f32; 3],
    pub lap_time: f32,
}

impl CEventDefinition {
    /// GameDataLoading.cpp 0x000A2EF0 — `race_dir` like `\Tracks\LACOAST\CIRCUIT\RACE01\`.
    pub fn load(race_dir: &str, num_players: i32) -> Option<Self> {
        let dir = VsFileDirectoryDetails::open(race_dir.trim_end_matches('\\')).ok()?;
        let mut f = VsTextFile::default();
        f.load(Some(&dir), "eventdef.txt").then_some(())?;
        Some(Self::parse(&mut f, num_players))
    }

    pub fn parse(f: &mut VsTextFile, num_players: i32) -> Self {
        let mut d = CEventDefinition { name: f.read_string().unwrap_or_default(), ..Default::default() };
        let s32 = |f: &mut VsTextFile| f.read_s32().map(|v| v.1).unwrap_or(0);
        let v3 = |f: &mut VsTextFile| f.read_f32n::<3>().map(|v| v.1).unwrap_or_default();
        d.event_type = s32(f);
        d.num_laps = s32(f);
        let opponents_in_file = s32(f);
        d.num_opponents = opponents_in_file;
        let extra = num_players - 1;
        if num_players == 2 && d.num_opponents > 0 {
            d.num_opponents -= 1;
        }
        if opponents_in_file != 0 {
            for _ in 0..extra + d.num_opponents {
                d.opponent_vehicles.push(f.read_string().unwrap_or_default());
            }
            for _ in 0..extra + d.num_opponents {
                let a = v3(f);
                let b = v3(f);
                d.colours.push((a, b));
            }
        }
        let n = s32(f).max(0) as usize;
        let raw: Vec<[f32; 3]> = (0..n).map(|_| v3(f)).collect();
        let order: Vec<i32> = (0..n).map(|_| s32(f)).collect();
        d.start_positions = order.iter().map(|&i| raw.get(i as usize).copied().unwrap_or_default()).collect();
        d.start_direction = f.read_f32().map(|v| v.1).unwrap_or(0.0);
        d.start_line = [v3(f), v3(f)];
        d.finish_line = [v3(f), v3(f)];
        d.unknown_line_point = v3(f);
        d.lap_time = f.read_f32().map(|v| v.1).unwrap_or(0.0);
        for p in d.start_line.iter_mut().chain(d.finish_line.iter_mut()).chain(d.start_positions.iter_mut()) {
            p[2] = -p[2];
        }
        if let Some(&first) = d.start_positions.first() {
            // VsAITools::ProjectOntoLine, then shift every grid slot so the first is 3.5 m short of the line
            let p = project_onto_line(d.start_line[0], d.start_line[1], first);
            let mut dv = [p[0] - first[0], p[1] - first[1], p[2] - first[2]];
            let len = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
            let l = len - 3.5;
            if len > 0.0 {
                for c in dv.iter_mut() {
                    *c = *c / len * l;
                }
            }
            for s in d.start_positions.iter_mut() {
                for k in 0..3 {
                    s[k] += dv[k];
                }
            }
        }
        d
    }
}

/// closest point to `p` on the infinite line through a and b.
fn project_onto_line(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let l2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    if l2 == 0.0 {
        return a;
    }
    let t = (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / l2;
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("eventdef.txt")) {
                out.push(p);
            }
        }
    }

    #[test]
    fn every_eventdef() {
        let root = visos::file::game_dir();
        if !root.exists() {
            return;
        }
        let mut files = Vec::new();
        walk(&root.join("Tracks"), &mut files);
        assert_eq!(files.len(), 84);
        let mut with_grid = 0;
        for p in &files {
            let mut f = VsTextFile::from_bytes("eventdef.txt", std::fs::read(p).unwrap());
            let d = CEventDefinition::parse(&mut f, 1);
            assert_eq!(d.opponent_vehicles.len(), d.num_opponents.max(0) as usize, "{}", p.display());
            if !d.start_positions.is_empty() {
                with_grid += 1;
                assert!(d.start_positions.iter().all(|s| s.iter().all(|c| c.is_finite())), "{}", p.display());
            }
        }
        assert!(with_grid > 40, "{with_grid}");
        let d = CEventDefinition::load("\\Tracks\\ExecTest\\Sprint\\Race01\\", 1).unwrap();
        assert_eq!(d.name, "Exec_Demo_Race");
        assert_eq!(d.opponent_vehicles, ["SKYLINE", "S2000", "350Z", "CELICA", "RX8"]);
        assert_eq!(d.start_positions.len(), 6);
    }

    #[test]
    fn step_order() {
        let s = in_game_init_steps("CIRCUIT\\RACE01\\", false, true);
        let i = |x| s.iter().position(|&y| y == x).unwrap();
        assert!(i(ELoadStep::EventDefinition) < i(ELoadStep::AiWorldInit));
        assert!(i(ELoadStep::FlagGirlTimeline) < i(ELoadStep::MarkersInit));
        assert!(!in_game_init_steps("CRUISE\\RACE01\\", false, true).contains(&ELoadStep::FlagGirlTimeline));
        assert_eq!(*s.last().unwrap(), ELoadStep::InGameInitEnded);
    }
}
