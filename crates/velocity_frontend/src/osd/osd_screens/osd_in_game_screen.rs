//! src\Osd\OSDScreens\OSDInGameScreen.cpp — the race overlay: position, lap time, lap counter, points, wrong
//! way, split times and status messages as OSD items. `Create` is 0x000F82F0 (in the Speedo.cpp region of
//! objregions), `Update` 0x000F99E0.
//!
//! Update is fed from `OsdRaceStatus` (the game side's race manager values) instead of reading the AI world.
//! Not ported: split time / failure / trailblazer / status messages' triggers, the countdown text pulse,
//! the countdown graphic items (0x000EE2D0 x3).

use crate::feck2::feck2_manager::{visos_colour, COLOUR_GREY, COLOUR_WHITE};
use crate::font_helper::font_helper::CFontHelper;
use crate::osd::osd_manager::{format_time, COSDManager, ItemId};
use crate::render2d::{Colour, TextureStore};
use crate::string_manager::string_manager::{string_crc, CStringManager};

/// `unk_6860F0` — single player item positions (x, y) in the 1333x1000 OSD space (y up).
pub const LAYOUT_1P: [[f32; 2]; 27] = [
    [1175.0, 925.0], [1175.0, 895.0], [697.5, 850.0], [890.0, 850.0], [1029.0, 850.0], [1125.0, 850.0],
    [100.0, 400.0], [1173.0, 785.0], [1075.0, 785.0], [1075.0, 810.0], [1173.0, 810.0], [685.0, 655.0],
    [685.0, 655.0], [685.0, 655.0], [685.0, 655.0], [200.0, 666.0], [685.0, 655.0], [685.0, 655.0],
    [666.0, 400.0], [1100.0, 780.0], [795.0, 785.0], [666.0, 937.5], [666.0, 62.5], [667.0, 600.0],
    [667.0, 600.0], [0.0, 0.0], [0.0, 0.0],
];

/// the split screen layouts (`byte_6F7157`), picked by Create 0x000F82F0 from `byte_6F9180` (side by side) and the
/// screen's player +20: `unk_6861C8` top (player 0), `unk_6862A0` bottom (1), `unk_686378` left (0), `unk_686450`
/// right (1). Entries 11..27 equal the single player table except 16 / 17 (split times) of the second player.
pub const LAYOUT_SPLIT: [[[f32; 2]; 11]; 4] = [
    [[235.0, 925.0], [235.0, 895.0], [215.0, 850.0], [410.0, 850.0], [550.0, 850.0], [640.0, 850.0], [1000.0, 800.0], [210.0, 575.0], [210.0, 575.0], [210.0, 600.0], [290.0, 600.0]],
    [[235.0, 475.0], [235.0, 445.0], [215.0, 400.0], [410.0, 400.0], [550.0, 400.0], [640.0, 400.0], [1000.0, 380.0], [210.0, 125.0], [210.0, 125.0], [210.0, 150.0], [290.0, 150.0]],
    [[235.0, 925.0], [235.0, 895.0], [215.0, 850.0], [410.0, 850.0], [510.0, 850.0], [600.0, 850.0], [1000.0, 800.0], [210.0, 575.0], [210.0, 575.0], [210.0, 600.0], [290.0, 600.0]],
    [[845.0, 925.0], [845.0, 895.0], [820.0, 850.0], [1010.0, 850.0], [1100.0, 850.0], [1185.0, 850.0], [1000.0, 800.0], [816.0, 575.0], [816.0, 575.0], [816.0, 600.0], [896.0, 600.0]],
];

/// OSDInGameScreen.cpp 0x000F82F0 — the item table of a screen: `split` = None single player, Some(side by side)
pub fn layout(split: Option<bool>, player: u32) -> [[f32; 2]; 27] {
    let mut p = LAYOUT_1P;
    let Some(side) = split else { return p };
    let t = 2 * side as usize + (player != 0) as usize;
    p[..11].copy_from_slice(&LAYOUT_SPLIT[t]);
    // split times: the second player's rows move with its half
    let split_time = [[685.0, 655.0], [685.0, 155.0], [685.0, 655.0], [1090.0, 655.0]][t];
    p[16] = split_time;
    p[17] = split_time;
    p
}

/// `dword_6FD768` = VISOS_Colour(128, 0, 102, 128) (static init 0x004238C1)
pub fn value_colour() -> Colour {
    visos_colour(128, 0, 102, 128)
}
/// `dword_77FDA8` red / `dword_77FD8C` green (static inits 0x00431AE1 / 0x00431B01)
pub fn red() -> Colour {
    visos_colour(128, 128, 0, 0)
}
pub fn green() -> Colour {
    visos_colour(128, 0, 128, 0)
}

const POSITIONS: [&str; 10] = ["OSD_1ST", "OSD_2ND", "OSD_3RD", "OSD_4TH", "OSD_5TH", "OSD_6TH", "OSD_7TH", "OSD_8TH", "OSD_9TH", "OSD_10TH"];

/// what `Update` reads from the race manager / race vehicle each frame.
#[derive(Clone, Debug, Default)]
pub struct OsdRaceStatus {
    pub racing: bool,           // driver +244: crossed the start line (never in a cruise, 0x0003B349)
    pub race_length: u32,       // VsAIWorld::GetRaceLength
    pub position: u32,          // GetPlayerVehicle position, 1-based; 0 / >10 = no position
    pub lap: u32,               // VsAIPoliceVehicle 0x00017E00
    pub laps: u32,              // event +20
    pub drag: bool,             // event type 5
    pub drag_round: u32,        // dword_6F5D10
    pub drag_rounds: u32,       // dword_6F5D0C
    pub lap_time: f32,          // 0x000FAEC0
    pub points: i32,            // race vehicle +84
    pub combo: u32,             // driver +152
    pub multiplier: u32,        // driver +156
    pub wrong_way: bool,        // driver +168 > 1.0
    pub no_hundredths: bool,    // byte_6F9180
    pub trail_blazer_message: u32, // GameManager +0x108C (0x000FAEA0), string crc, 0 = none
    pub attract: bool,          // g_AttractMode +0 playing (FontHelper 0x000B36B0)
    pub attract_quitting: bool, // g_AttractMode +0x48 (FontHelper 0x000B36F0)
    pub session_state: Option<u8>, // dword_6F5BEC (GameManager +0x24 session phase); None = no game flow
}

/// the attract mode's "Press Start" fade (process statics, reset by Create 0x000F9252..0x000F927A)
#[derive(Clone, Copy, Debug)]
pub struct AttractFade {
    pub dir: i32,      // dword_6FD758: -1 fade out, 1 fade in, 0 hold
    pub phase: i32,    // dword_6FD75C: 0 item 20 shown, 1 swapped to item 21
    pub step: u32,     // dword_6860E4
    pub alpha: u32,    // dword_6860E8 (unsigned: the quit fade wraps below 0, original)
    pub lap_secs: u32, // dword_6FD760: whole seconds of the lap time (0x000FAEC0) when racing
}

/// `COSDInGameScreen` item table (`m_ppOSDScreenItemA`) and its text buffers (`m_pszOSDScreenItemTextA`).
pub struct COSDInGameScreen {
    pub items: Vec<Option<ItemId>>,
    pub texts: Vec<String>,
    pub player: u32, // +20
    pub fade: AttractFade,
}

/// OSDInGameScreen.cpp 0x000F82F0 — `Create` for `player` (+20); `split` = None single player, Some(side by side)
/// two players (`byte_6F7157`, `byte_6F9180`). Player 1's screen has no attract / logo / bar / network items.
#[allow(clippy::too_many_arguments)]
pub fn create(
    osd: &mut COSDManager,
    fonts: &CFontHelper,
    strings: &CStringManager,
    tex: &TextureStore,
    drag: bool,
    player: u32,
    split: Option<bool>,
) -> COSDInGameScreen {
    let fade = AttractFade { dir: 0, phase: 0, step: 2, alpha: 0x80, lap_secs: 0 };
    let mut s = COSDInGameScreen { items: vec![None; 27], texts: vec![String::new(); 9], player, fade };
    let p = layout(split, player);
    // (slot, string id or "" for a text buffer item, font, flags, mask, colour, visible)
    let lap_label = if drag { "OSD_DRAG_ROUND" } else { "OSD_LAP" };
    let orange = visos_colour(128, 124, 75, 3);
    let black = visos_colour(0, 0, 0, 0);
    let table: [(usize, &str, usize, u32, u32, Option<Colour>, bool); 23] = [
        (0, "OSD_1ST", 10, 0x2014, 3, Some(orange), true),
        (1, "", 9, 0x2012, 3, None, true),
        (2, "OSD_LAP_TIME", 8, 0x2012, 3, None, true),
        (3, "", 8, 0x2012, 3, Some(value_colour()), true),
        (4, lap_label, 8, 0x2012, 3, None, true),
        (5, "", 8, 0x2012, 3, Some(value_colour()), true),
        (7, "OSD_POINTS", 7, 0x2014, 3, Some(COLOUR_GREY), true),
        (8, "", 7, 0x2014, 3, None, true),
        (9, "", 7, 0x2014, 3, Some(value_colour()), true),
        (10, "", 7, 0x2014, 3, Some(COLOUR_WHITE), true),
        (15, "OSD_WRONG_WAY", 9, 0x2012, 3, Some(COLOUR_WHITE), false),
        (11, "", 9, 0x2048, 0x2000, Some(COLOUR_WHITE), false),
        (12, "OSD_INGAME_RACE_STATUS_INFO", 9, 0x2048, 0x2000, Some(COLOUR_WHITE), false),
        (13, "OSD_INGAME_TRAILBLAZER_INFO", 9, 0x2048, 3, Some(COLOUR_WHITE), false),
        (14, "OSD_FAILURE_INFO", 9, 0x2018, 3, Some(COLOUR_WHITE), false),
        (16, "OSD_SPLIT_TIME_AHEAD", 9, 0x2018, 3, Some(red()), false),
        (17, "OSD_SPLIT_TIME_BEHIND", 9, 0x2018, 3, Some(green()), false),
        (20, "", 10, 8, 32, Some(COLOUR_WHITE), false),
        (21, "", 9, 8, 32, Some(black), false),
        // 22..24: graphic items (countdown / light textures), not ported
        (25, "OSD_WAITING_FOR_PLAYERS", 9, 0x2048, 512, Some(COLOUR_WHITE), false),
        (26, "OSD_PREPARING_FOR_START", 9, 0x2048, 512, Some(COLOUR_WHITE), false),
        (6, "", 0, 0, 0, None, false),
        (18, "", 0, 0, 0, None, false),
    ];
    for (slot, name, font, flags, mask, colour, visible) in table {
        if slot == 6 || slot == 18 {
            continue; // layout entries without an item
        }
        if player != 0 && slot >= 20 {
            continue; // items 20..26 only `if (+20 == 0)`
        }
        let font_id = font; // CFontHelper::GetFontIndex(set, n) = ids[n]
        let id = if name.is_empty() {
            Some(osd.create_text_item(fonts, tex, "", font_id, p[slot][0], p[slot][1], 0, -1.0, flags, mask))
        } else {
            osd.create_string_item(fonts, strings, tex, string_crc(name), font_id, p[slot][0], p[slot][1], 0, -1.0, flags, mask)
        };
        s.items[slot] = id;
        if let Some(c) = colour {
            set_colour(osd, &s, slot, c);
        }
        if !visible {
            set_visible(osd, &s, slot, false);
        }
    }
    if player != 0 {
        return s;
    }
    // 0x000F8E78: items 20 / 21 share text buffer 8, "Press Start" (a literal, not a string id)
    for slot in [20, 21] {
        set_text(osd, &mut s, slot, 8, "Press Start".to_string());
    }
    // 0x000F9491: item 22 = OSD_overlay_game_logo (OSDInGameScreen 0x000FAB40 loads \osd\osdT.vcd), 480 x 120
    let logo = tex.find(visos::handy::handy_crc::name_hash("osd_overlay_game_logo"));
    let id = osd.create_graphic_item(logo, p[22][0], p[22][1], 0, f32::from_bits(0x43F00000), f32::from_bits(0x42F00000), -1.0, 2, 32);
    osd.item_mut(id).set_visible(false);
    s.items[22] = Some(id);
    // 0x000F94xx: 23 / 24 = menu_top / menu_bottom (FAB40 texture slots +4 / +8), 1344 x 200 at (664, 900) /
    // (664, 100), layer 100, flags 0x48, mask 1024, hidden (no SetVisible(1) anywhere in Update)
    for (slot, name, y) in [(23, "menu_top", f32::from_bits(0x4461_0000)), (24, "menu_bottom", f32::from_bits(0x42C8_0000))] {
        let t = tex.find(visos::handy::handy_crc::name_hash(name));
        let id = osd.create_graphic_item(t, f32::from_bits(0x4426_0000), y, 100, f32::from_bits(0x44A8_0000), f32::from_bits(0x4348_0000), -1.0, 0x48, 1024);
        osd.item_mut(id).set_visible(false);
        s.items[slot] = Some(id);
    }
    s
}

/// OSDInGameScreen.cpp 0x000F9980 — the cinematic bars (items 23 / 24) shown / hidden
pub fn set_cinematic_bars(s: &COSDInGameScreen, osd: &mut COSDManager, show: bool) {
    for slot in [23, 24] {
        if let Some(id) = s.items.get(slot).copied().flatten() {
            osd.item_mut(id).set_visible(show);
        }
    }
}

/// OSDInGameScreen.cpp 0x000F9A70 / 0x000FAFB0 — (..0x000F9C08 of Update) the per-update part of `Update` while the attract mode plays:
/// "Press Start" fades out once the lap clock passed 1 s, swaps from item 20 to 21, fades back in and holds; the
/// quit fade takes 20 a step. Called once per game update (GameTick).
pub fn attract_step(s: &mut COSDInGameScreen, osd: &mut COSDManager, st: &OsdRaceStatus) {
    if st.attract {
        let f = &mut s.fade;
        if f.dir == -1 {
            f.alpha = f.alpha.wrapping_sub(f.step);
        } else if f.dir == 1 {
            f.alpha = f.alpha.wrapping_add(f.step);
        }
        let mut swap = false;
        if f.phase == 0 {
            if f.dir == -1 && f.alpha == 0 {
                f.dir = 1;
                f.phase = 1;
                swap = true;
            }
        } else if f.phase == 1 && f.alpha == 0x80 {
            f.dir = 0;
        }
        if st.attract_quitting {
            // 0x000FAFB0 = max(a, 0) on unsigned: never clamps (original)
            f.alpha = f.alpha.wrapping_sub(20).max(0);
        }
        let colour = visos_colour(f.alpha & 0xFF, 0x80, 0x80, 0x80);
        if f.lap_secs > 1 && f.phase != 1 {
            f.dir = -1;
        }
        if swap {
            set_visible(osd, s, 20, false);
            set_visible(osd, s, 21, true);
        }
        set_colour(osd, s, 20, colour);
        set_colour(osd, s, 21, colour);
    }
    if st.racing {
        // 0x000F9C03: _ftol of the lap time
        s.fade.lap_secs = st.lap_time as i32 as u32;
    }
}

fn set_colour(osd: &mut COSDManager, s: &COSDInGameScreen, slot: usize, c: Colour) {
    if let Some(id) = s.items[slot] {
        osd.item_mut(id).colour = c;
    }
}

fn set_visible(osd: &mut COSDManager, s: &COSDInGameScreen, slot: usize, v: bool) {
    if let Some(id) = s.items[slot] {
        osd.item_mut(id).set_visible(v);
    }
}

fn set_text(osd: &mut COSDManager, s: &mut COSDInGameScreen, slot: usize, buf: usize, t: String) {
    s.texts[buf] = t.clone();
    if let Some(id) = s.items[slot] {
        osd.item_mut(id).text = Some(t);
    }
}

/// one AI race vehicle (`VsRaceManager` +36 array, count +56) for the name tags
#[derive(Clone, Debug, Default)]
pub struct NameTagVehicle {
    /// vehicle +120 (the race slot: item 27 + slot)
    pub slot: usize,
    /// VsAIDriver 0x00016690, in the render world (the space `view_proj` maps)
    pub pos: [f32; 3],
    /// the driver profile's name (vehicle +564 -> 0x000153B0) or the police name (+36)
    pub name: String,
    /// the profile colour (0x00015F80), alpha replaced
    pub colour: Colour,
}

/// what the name tags read of the race: driver 0's position, the AI vehicles, and the camera
#[derive(Clone, Debug, Default)]
pub struct NameTagInput {
    pub player: [f32; 3],
    pub vehicles: Vec<NameTagVehicle>,
    /// world -> clip of the race camera (column-major, bevy `Mat4`); filled by the hud from the 3D camera
    pub view_proj: Option<[f32; 16]>,
    /// player 1's view camera (split screen, `TrackView { player: 1 }`): the 3D pass runs inside each player view
    pub view_proj1: Option<[f32; 16]>,
}

/// item index of race slot 0's name tag (`*a2 + 108`)
pub const NAME_TAG_FIRST: usize = 27;

/// OSDInGameScreen.cpp 0x000F82F0 (end): one text item per racer (font 6, flags 0x180000 = in 3D | centred,
/// mask 3). Created here when a slot is first seen; the original makes event +21 of them in Create.
fn name_tag_item(s: &mut COSDInGameScreen, osd: &mut COSDManager, fonts: &CFontHelper, tex: &TextureStore, slot: usize) -> ItemId {
    let i = NAME_TAG_FIRST + slot;
    if s.items.len() <= i {
        s.items.resize(i + 1, None);
    }
    if let Some(id) = s.items[i] {
        return id;
    }
    let id = osd.create_text_item(fonts, tex, "", crate::font_helper::font_helper::FONT_GAME_FE11, 0.0, 0.0, 0, -1.0, 0x180000, 3);
    s.items[i] = Some(id);
    id
}

/// `dword_6FD770` = (0, 1.6, 0) above the car (static init 0x00423860)
const NAME_TAG_OFFSET: [f32; 3] = [0.0, f32::from_bits(0x3FCCCCCD), 0.0];
/// `flt_6860C0` / `dword_6860C4` / `dword_6860C8`: full alpha up to 20 m, fading to `flt_6FD750` (bss 0) at 40 m
const NAME_TAG_NEAR_ALPHA: f32 = 1.0;
const NAME_TAG_NEAR: f32 = 20.0;
const NAME_TAG_FAR: f32 = 40.0;
const NAME_TAG_FAR_ALPHA: f32 = 0.0;

/// OSDInGameScreen.cpp 0x000F9340 / 0x000F98E0 / OSD3dInterfaceConfiguration.cpp 0x000ED550 / OSDManager.cpp 0x000EFB80 — the name tags (called by Update 0x000FA92A unless `g_AttractMode` plays):
/// shown with the Player names option (`byte_6F915F`) or the `byte_6FD754` debug flag (police names; 0 here)
pub fn update_name_tags(s: &mut COSDInGameScreen, osd: &mut COSDManager, fonts: &CFontHelper, tex: &TextureStore, input: &NameTagInput, player_names: bool) {
    let debug_names = false; // byte_6FD754
    if !(player_names || debug_names) {
        for v in &input.vehicles {
            if let Some(Some(id)) = s.items.get(NAME_TAG_FIRST + v.slot) {
                osd.item_mut(*id).set_visible(false);
            }
        }
        return;
    }
    let p = input.player;
    for v in &input.vehicles {
        let id = name_tag_item(s, osd, fonts, tex, v.slot);
        osd.item_mut(id).set_visible(true);
        let d = [v.pos[0] - p[0], v.pos[1] - p[1], v.pos[2] - p[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let mut off = NAME_TAG_OFFSET;
        off[1] = dist / NAME_TAG_FAR * 4.0 + off[1];
        let pos = [v.pos[0] + off[0], v.pos[1] + off[1], v.pos[2] + off[2]];
        let a = if NAME_TAG_NEAR < dist {
            if dist <= NAME_TAG_FAR {
                let t = 1.0 - (dist - NAME_TAG_NEAR) / (NAME_TAG_FAR - NAME_TAG_NEAR);
                // CRim 0x000452C0: (b - a) * t + a
                (NAME_TAG_NEAR_ALPHA - NAME_TAG_FAR_ALPHA) * t + NAME_TAG_FAR_ALPHA
            } else {
                NAME_TAG_FAR_ALPHA
            }
        } else {
            NAME_TAG_NEAR_ALPHA
        };
        let alpha = (a * 128.0) as i32 as u32 & 0xFF;
        let it = osd.item_mut(id);
        it.text = Some(v.name.clone());
        // the bridge data position (OSDBridge 0x00098470 -> 0x000F98E0 -> 0x000ED550 item +36..+44)
        it.x = pos[0];
        it.y = pos[1];
        it.z = pos[2];
        if alpha == 0 {
            it.set_visible(false);
        }
        it.colour = (v.colour & 0x00FF_FFFF) | (alpha << 24);
        // 0x000EFB80: +88 = 0.1 (non-zero: drawn on top), +92 = 2.0 (fade start distance)
        it.zoom_x = f32::from_bits(0x3DCCCCCD);
        it.zoom_y = 2.0;
    }
}

/// OSDInGameScreen.cpp 0x000F99E0 — `Update` (the race-status part)
pub fn update(s: &mut COSDInGameScreen, osd: &mut COSDManager, st: &OsdRaceStatus) {
    let lap_time = if st.racing {
        format_time(st.lap_time, st.no_hundredths)
    } else if st.no_hundredths {
        "--:--".to_string()
    } else {
        "--:--.--".to_string()
    };
    set_text(osd, s, 3, 1, lap_time);
    let lap = if st.drag {
        format!(" {}/{}", st.drag_round + 1, st.drag_rounds)
    } else if st.racing {
        format!("{}/{}", st.lap, st.laps)
    } else {
        format!("--/{}", st.laps)
    };
    set_text(osd, s, 5, 2, lap);
    let crc = match st.position {
        1..=10 if st.racing => string_crc(POSITIONS[st.position as usize - 1]),
        _ => string_crc("OSD_NO_POS"),
    };
    if let Some(id) = s.items[0] {
        osd.item_mut(id).string = Some(crc);
    }
    set_text(osd, s, 1, 0, format!("/{}", st.race_length));
    set_text(osd, s, 8, 4, format!("{}", st.points));
    if st.combo != 0 {
        set_text(osd, s, 9, 5, format!("{}", st.combo));
        set_visible(osd, s, 9, true);
    } else {
        set_visible(osd, s, 9, false);
    }
    if st.multiplier <= 1 {
        set_visible(osd, s, 10, false);
    } else {
        set_text(osd, s, 10, 6, format!("x{}", st.multiplier));
        set_visible(osd, s, 10, true);
    }
    set_visible(osd, s, 15, st.wrong_way);
    // the trailblazer's message (item +52: its string handle, shown while set)
    if st.trail_blazer_message != 0 {
        if let Some(id) = s.items[13] {
            osd.item_mut(id).string = Some(st.trail_blazer_message);
        }
        set_visible(osd, s, 13, true);
    } else {
        set_visible(osd, s, 13, false);
    }
    // 0x000FA915: name tags (`update_name_tags`, run by the hud after this) only outside the attract mode; then the session phase
    // (dword_6F5BEC) drives the OSD manager mask: bit 1 = the race hud (mask 3 items), bit 32 = the attract items
    let Some(state) = st.session_state else { return };
    if state != 0 {
        if state == 3 {
            if st.attract {
                osd.hidden_mask |= 1;
                osd.hidden_mask &= !32;
                set_visible(osd, s, 22, true);
                set_visible(osd, s, 20, true);
            } else {
                osd.hidden_mask &= !1;
                osd.hidden_mask |= 32;
            }
            set_visible(osd, s, 25, false);
            set_visible(osd, s, 26, false);
        } else if state == 7 {
            osd.hidden_mask |= 1;
        }
    } else {
        osd.hidden_mask |= 1;
        osd.hidden_mask |= 32;
        // network session state (GameManager 0x000BFE60) 15 / 6 = preparing; single player has none
        set_visible(osd, s, 25, true);
        set_visible(osd, s, 26, false);
    }
}
