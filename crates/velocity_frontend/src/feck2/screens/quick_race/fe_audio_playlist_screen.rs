//! `gAudioPlaylistScreen` (`dword_75FF98`, 44 bytes): the DJ booth's music playlist editor, reached from the hangout's
//! DJ character. The code sits at 0x0022B7E0..0x0022D7B0 in FEQuickRaceTextScreen.cpp's object range. State (+24):
//! 0 list kind / shuffle / "display tracks" rows, 1 the track list, 2 add by id (named master tracks), 3 the edit
//! menu (play / remove), 4 add by index (all songs of soundtracks 1..). The list being edited is
//! `Feck2.playlist.list` (+0, set by the hangout); `VsAudioTrackList` / `VsSoundtrackPlayer` are the mirror in
//! `Feck2.playlist` (see `feck2::playlist_link`).
//!
//! deviations: the camera is `FE3DCamera::CharacterCamera(3, SELECT)` as the foyer camera's position / look-at (the
//! layer setup calls 0x00271CF0 / 0x00272940 / 0x00271F10 / 0x00271FA0 are folded into it); the lists past the
//! fixed ones never have a working list (`BuildList` only for fixed lists, the original indexes past its array).

use std::sync::{Arc, Mutex};

use crate::feck2::screens::options::{enter, ov};
use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_GREY};
use crate::feck2::feck2_object::*;
use crate::feck2::playlist_link::{ListName, PlaylistRequest};
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::hangout::fe_hangout_screen::character_camera;
use crate::string_manager::string_manager::string_crc;

pub struct CAudioPlaylistScreen {
    pub screen: ScreenId,          // +4
    pub layer: LayerId,            // +8
    pub entered: bool,             // +12
    pub countdown: i32,            // +16 frames to the screen change, -1 = none
    pub next_screen: u32,          // +20
    pub state: u32,                // +24
    pub node: Option<usize>,       // +28 the selected track node (0 = head sentinel, None = null)
    pub texts: [ElementId; 2],     // +32 "play track" / "remove track"
    pub edit_sel: i32,             // +40
}

/// FEQuickRaceTextScreen.cpp 0x0022BA30 / 0x0022BB30 — create the screen: layer 4, two "mainmenutext" (font 1) at
/// x 300, y 290 / 310 with the track menu strings
pub fn create_audio_playlist_screen(fe: &mut Feck2) -> Arc<Mutex<CAudioPlaylistScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("AudioPlaylistScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("AudioPlaylistScreen_2DLayer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 1, crc: string_crc("mainmenutext"), ..Default::default() };
    let ids = [string_crc("AUDIO_TRACKMENU_PLAYTRACK"), string_crc("AUDIO_TRACKMENU_REMOVETRACK")]; // dword_760220 / 760224
    let mut texts = [ElementId::default(); 2];
    for (i, t) in texts.iter_mut().enumerate() {
        d.position = [f32::from_bits(1133903872), (20 * i + 290) as f32, 0.0, 1.0];
        d.string = ids[i];
        *t = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CAudioPlaylistScreen { screen, layer, entered: false, countdown: -1, next_screen: 0, state: 0, node: None, texts, edit_sel: 0 }));
    // 0x0022BDF0 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, on_start), ev::START);
    fe.map(layer, functor(&obj, |s, fe, _| step_if(s, fe, 1)), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| step_if(s, fe, -1)), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, on_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, on_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, on_update), ev::UPDATE);
    fe.map(layer, functor(&obj, on_option1), ev::OPTION1_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| on_shuffle(s, fe)), ev::OPTION2_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| move_edit(s, fe, -1) != 0), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| move_edit(s, fe, 1) != 0), ev::DOWN_PRESS);
    obj
}

/// FEQuickRaceTextScreen.cpp 0x0022B7E0 — the list kind's string (empty for a user list)
fn kind_name(fe: &Feck2) -> &'static str {
    match fe.playlist.list_type(fe.playlist.list) {
        1 => "FRONTEND",
        2 => "INGAME",
        3 => "LONDON",
        4 => "LA",
        5 => "TOKYO",
        6 => "NEVER",
        _ => "",
    }
}

/// FEQuickRaceTextScreen.cpp 0x0022B950 — the shuffle mode's string
fn shuffle_name(fe: &Feck2) -> &'static str {
    if fe.playlist.list_shuffle(fe.playlist.list) != 0 {
        "RANDOM"
    } else {
        "SEQUENTIAL"
    }
}

/// the list's name in the menu title (`CElementStaticText::SetString` / text of the soundtrack)
fn set_title(fe: &mut Feck2, o: &overlay::CFE2DOverlay) {
    match fe.playlist.list_name(fe.playlist.list) {
        ListName::Hash(h) => fe.text(o.menu_title).set_string(h),
        ListName::Text(t) => fe.text(o.menu_title).set_text(&t.encode_utf16().collect::<Vec<_>>()),
    }
    fe.el(o.menu_title).set_visible(true);
}

/// FEQuickRaceTextScreen.cpp 0x0022C250 — Enter: camera on the screen, the edit texts hidden
fn on_enter(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    for &t in &s.texts {
        fe.element_load(t);
        fe.el(t).set_visible(false);
    }
    if !s.entered {
        s.entered = true;
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022C430 — Activate: select / back, the DJ booth heading, 5 menu rows
fn on_activate(_s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_message(o, fe, 0);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::set_heading(o, fe, string_crc("HANGOUT_DJBOOTH"));
        menu::set_window(o, 5);
        overlay::enable(o);
    });
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022C310 — Start: the foyer behind the DJ (character 3's SELECT camera, FOV 50), the
/// first menu
fn on_start(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::show_message(o, fe, false));
    fe.foyer.show_background(true, true, 0);
    let (pos, look, _) = character_camera(fe, 3, 2, 0);
    fe.foyer.set_position_look_at([pos[0], pos[1], pos[2]], [look[0], look[1], look[2]]);
    fe.foyer.snap_to_target();
    fe.foyer.set_fov(f32::from_bits(1112014848)); // 50
    show_edit_texts(s, fe, false);
    s.state = 0;
    state0_menu(s, fe);
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022C810 — show / hide the edit texts (showing resets the selector)
fn show_edit_texts(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, show: bool) {
    if show {
        s.edit_sel = 0;
    }
    for &t in &s.texts {
        fe.el(t).set_visible(show);
    }
}

/// FEQuickRaceTextScreen.cpp 0x0022C690 — the two rows' info strings
fn refresh_info(fe: &mut Feck2) {
    let (k, sh) = (string_crc(kind_name(fe)), string_crc(shuffle_name(fe)));
    ov(fe, |o, fe| {
        menu::set_entry_info(o, fe, 0, k);
        menu::set_entry_info(o, fe, 1, sh);
    });
}

/// FEQuickRaceTextScreen.cpp 0x0022C560 — left / right: the add lists swap, else kind (wraps 0..6) or shuffle
fn step(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, d: i32) {
    let row = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
    let l = fe.playlist.list;
    if s.state == 2 {
        s.state = 4;
        songs_menu(s, fe);
    } else if s.state == 4 {
        s.state = 2;
        master_menu(s, fe);
    } else if row == 1 {
        let mut v = d + fe.playlist.list_shuffle(l) as i32;
        if v == 2 {
            v = 0;
        } else if v == -1 {
            v = 1;
        }
        fe.playlist.set_list_shuffle(l, v as u32);
    } else if row == 0 {
        let mut v = d + fe.playlist.list_type(l) as i32;
        if v == 7 {
            v = 0;
        } else if v == -1 {
            v = 6;
        }
        fe.playlist.set_list_type(l, v as u32);
    }
    refresh_info(fe);
}

/// FEQuickRaceTextScreen.cpp 0x0022C770 / 0x0022C7C0 — left / right in states 0, 2, 4
fn step_if(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, d: i32) -> bool {
    if matches!(s.state, 0 | 2 | 4) {
        step(s, fe, d);
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022C880 — Select
fn on_select(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let sel = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
    let l = fe.playlist.list;
    let fixed = l < fe.playlist.fixed_list_count();
    match s.state {
        0 => {
            if sel == 2 {
                s.state += 1;
                if fixed {
                    fe.playlist.tracks.build_list(l);
                    fe.playlist.dirty = true;
                }
                track_list(s, fe);
            }
        }
        1 => {
            s.state = 3;
            ov(fe, |o, fe| menu::set_active(o, fe, false));
            s.edit_sel = 0;
            move_edit(s, fe, 0);
            show_edit_texts(s, fe, true);
        }
        2 => {
            s.state -= 1;
            let id = fe.overlay.as_ref().map_or(0, |o| menu::entry_string(o, sel.max(0) as usize));
            if fixed {
                fe.playlist.tracks.add_track_by_id(l, s.node, id);
                fe.playlist.dirty = true;
            }
            track_list(s, fe);
        }
        4 => {
            s.state = 1;
            if fixed {
                let counts = fe.playlist.song_counts();
                fe.playlist.tracks.add_track_by_index(l, s.node, sel.max(0) as u32, &counts);
                fe.playlist.dirty = true;
            }
            track_list(s, fe);
        }
        3 => {
            s.state = 1;
            ov(fe, |o, fe| menu::set_active(o, fe, true));
            show_edit_texts(s, fe, false);
            if s.edit_sel != 0 {
                if let Some(n) = s.node {
                    if fixed {
                        let prev = fe.playlist.tracks.edit[l].as_ref().and_then(|e| e.nodes[n].prev);
                        fe.playlist.tracks.remove_track(l, prev);
                        fe.playlist.dirty = true;
                    }
                    track_list(s, fe);
                }
            } else if let Some(n) = s.node {
                // VsAudio::NextTrack(1, node) 0x0004F4B0 -> PlayTrack(1, node): the node's song
                let (soundtrack, song) = fe
                    .playlist
                    .tracks
                    .edit
                    .get(l)
                    .and_then(|e| e.as_ref())
                    .map_or((0, 0), |e| (e.nodes[n].soundtrack as usize, e.nodes[n].master as usize));
                fe.playlist.requests.push(PlaylistRequest::PlayTrack { soundtrack, song });
            } else {
                // PlayTrack(1, 0, list - (fixed - 1), selected row)
                let soundtrack = l.saturating_sub(fe.playlist.fixed_list_count() - 1);
                fe.playlist.requests.push(PlaylistRequest::PlayTrack { soundtrack, song: sel.max(0) as usize });
            }
        }
        _ => {}
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022CA90 — Cancel: one state back; from the first menu leave after a frame, saving
fn on_cancel(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let l = fe.playlist.list;
    match s.state {
        2 => {
            s.state -= 1;
            track_list(s, fe);
        }
        1 => {
            s.state -= 1;
            state0_menu(s, fe);
            if l < fe.playlist.fixed_list_count() {
                fe.playlist.tracks.commit_list(l, true);
                fe.playlist.dirty = true;
            }
        }
        0 => {
            fe.playlist.requests.push(PlaylistRequest::SaveLists); // 0x00105E10
            s.next_screen = string_crc("PREVIOUS");
            s.countdown = 1;
        }
        3 => {
            s.state = 1;
            ov(fe, |o, fe| menu::set_active(o, fe, true));
            show_edit_texts(s, fe, false);
        }
        _ => {}
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022CB90 — Update: the delayed screen change
fn on_update(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.countdown == -1 || s.next_screen == 0 {
        return true;
    }
    if s.countdown != 0 {
        s.countdown -= 1;
        return true;
    }
    s.countdown = -1;
    if s.next_screen == string_crc("PREVIOUS") {
        fe.goto_previous_screen(1, 0);
    } else {
        fe.characters.set_visible(false);
        fe.goto_screen(s.next_screen);
    }
    false
}

/// FEQuickRaceTextScreen.cpp 0x0022CC90 — option 1: the add-by-id list (from the track list)
fn on_option1(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.state == 1 {
        s.state += 1;
        master_menu(s, fe);
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022CCD0 — option 2: shuffle the working list
fn on_shuffle(s: &mut CAudioPlaylistScreen, fe: &mut Feck2) -> bool {
    if s.state == 1 {
        let l = fe.playlist.list;
        if l < fe.playlist.fixed_list_count() {
            let mut r = std::mem::take(&mut fe.playlist.rand);
            fe.playlist.tracks.shuffle(l, &mut || r.rand());
            fe.playlist.rand = r;
            fe.playlist.dirty = true;
        }
        track_list(s, fe);
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x0022CD20 / 0x0022CD50 / 0x0022CD80 — up / down: the node in the track list, the edit
/// selector in the edit menu (0 = handled by the menu itself)
fn move_edit(s: &mut CAudioPlaylistScreen, fe: &mut Feck2, d: i32) -> i32 {
    if s.state == 1 {
        let l = fe.playlist.list;
        if let Some(e) = fe.playlist.tracks.edit.get(l).and_then(|e| e.as_ref()) {
            if let Some(n) = s.node {
                if d == -1 {
                    // the previous node unless that is the head
                    if let Some(p) = e.nodes[n].prev {
                        if e.nodes[p].prev.is_some() {
                            s.node = Some(p);
                        }
                    }
                } else if let Some(nx) = e.nodes[n].next {
                    s.node = Some(nx);
                }
            }
        }
        return 1;
    }
    if s.state == 3 {
        s.edit_sel += d;
        if s.edit_sel == 2 {
            s.edit_sel = 0;
        }
        let on = visos_colour(128, 120, 77, 14);
        for i in 0..2 {
            let c = if i as i32 == s.edit_sel { on } else { COLOUR_GREY };
            fe.text(s.texts[i]).set_colour(c);
        }
        return 0;
    }
    1
}

/// FEQuickRaceTextScreen.cpp 0x0022CED0 — the first menu: when / how to play / display tracks
fn state0_menu(s: &mut CAudioPlaylistScreen, fe: &mut Feck2) {
    let _ = s;
    let (k, sh) = (string_crc(kind_name(fe)), string_crc(shuffle_name(fe)));
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        set_title(fe, o);
        menu::add_entry(o, fe, string_crc("AUDIO_LIST_WHENTOPLAY"), true, true, k);
        menu::add_entry(o, fe, string_crc("AUDIO_LIST_HOWTOPLAY"), true, true, sh);
        menu::add_entry(o, fe, string_crc("AUDIO_LIST_DISPLAYTRACKS"), true, false, 0);
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, true);
    });
}

/// FEQuickRaceTextScreen.cpp 0x0022D080 — the track list: a fixed list's working nodes, or a soundtrack's songs
fn track_list(s: &mut CAudioPlaylistScreen, fe: &mut Feck2) {
    let l = fe.playlist.list;
    if l >= fe.playlist.fixed_list_count() {
        let st = l - (fe.playlist.fixed_list_count() - 1);
        let songs = fe.playlist.soundtrack(st).map(|x| x.songs.clone()).unwrap_or_default();
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            set_title(fe, o);
            for name in &songs {
                menu::add_text_entry(o, fe, name, true);
            }
            menu::show(o, fe, true, false);
            menu::set_active(o, fe, true);
        });
        s.node = None;
        return;
    }
    // GetTrackId(list, n) until 0; -1 = a song of a dashboard soundtrack
    let mut rows: Vec<Result<u32, String>> = Vec::new();
    let mut n = 0;
    loop {
        let id = fe.playlist.tracks.get_track_id(l, n);
        if id == 0 {
            break;
        }
        if id == u32::MAX {
            let e = fe.playlist.tracks.edit[l].as_ref().unwrap();
            let node = &e.nodes[fe.playlist.tracks.get_track(l, n).unwrap()];
            let name = fe.playlist.soundtrack(node.soundtrack as usize).and_then(|x| x.songs.get(node.master as usize)).cloned().unwrap_or_default();
            rows.push(Err(name));
        } else {
            rows.push(Ok(id));
        }
        n += 1;
    }
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        set_title(fe, o);
        for r in &rows {
            match r {
                Ok(id) => {
                    menu::add_entry(o, fe, *id, true, false, 0);
                }
                Err(name) => {
                    menu::add_text_entry(o, fe, name, true);
                }
            }
        }
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, true);
    });
    s.node = fe.playlist.tracks.edit[l].as_ref().map(|e| e.nodes[0].next.unwrap_or(0));
}

/// FEQuickRaceTextScreen.cpp 0x0022D310 — the master list's named tracks
fn master_menu(_s: &mut CAudioPlaylistScreen, fe: &mut Feck2) {
    let hashes: Vec<u32> = fe.playlist.tracks.master.iter().map(|m| m.hash).take_while(|&h| h != 0).collect();
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        set_title(fe, o);
        for h in hashes {
            menu::add_entry(o, fe, h, true, false, 0);
        }
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, true);
    });
}

/// FEQuickRaceTextScreen.cpp 0x0022D3F0 — every song of soundtracks 1..
fn songs_menu(_s: &mut CAudioPlaylistScreen, fe: &mut Feck2) {
    let names: Vec<String> = fe.playlist.soundtracks.iter().skip(1).flat_map(|s| s.songs.iter().cloned()).collect();
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        set_title(fe, o);
        for n in &names {
            menu::add_text_entry(o, fe, n, true);
        }
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_and_names() {
        assert_eq!(visos_colour(128, 120, 77, 14), 0xFFF09A1C);
        assert_eq!(string_crc("PREVIOUS") as i32, -1579229447);
        assert_eq!(string_crc("AudioPlaylistScreen") as i32, -1476231937);
    }
}
