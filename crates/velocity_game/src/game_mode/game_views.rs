//! GameManager.cpp "Init Game Views" (InGameInit 0x000B6498 -> 0x000B3EC0) and the view rect setter 0x000B3960
//! (in FontHelper's idb range): two sets of 10 views at GameManager +588 (world views) and +988 (second set),
//! 40 bytes each (`x, y, w, h` in screen pixels, +28 player, +32 kind, +36 used). The split-screen race draws
//! each player's world camera into its view (`race_cars::{player_cameras, place_camera}`).

use bevy::prelude::*;

/// `dword_68797C` / `dword_687980`: the screen the view rects are made for (CreateDevice 640x480)
pub const SCREEN_W: u32 = 640;
pub const SCREEN_H: u32 = 480;
pub const NUM_VIEWS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameView {
    pub x: u32,      // +0
    pub y: u32,      // +4
    pub w: u32,      // +8
    pub h: u32,      // +12
    pub player: u32, // +28
    pub kind: i32,   // +32 (-1 none)
    pub used: bool,  // +36
}

impl Default for GameView {
    fn default() -> Self {
        Self { x: 0, y: 0, w: 0, h: 0, player: 0, kind: -1, used: false }
    }
}

impl GameView {
    /// FontHelper.cpp-attributed 0x000B3960 — kinds: 0 full screen, 1 mirror (middle third, top sixth), 2 / 3 top /
    /// bottom half, 4 / 5 left / right half, 6..9 quarters (+28 is not written for those)
    pub fn set(&mut self, kind: i32) {
        let (w, h) = (SCREEN_W, SCREEN_H);
        self.used = true;
        self.kind = kind;
        let r = match kind {
            0 => Some((0, 0, w, h, Some(0))),
            1 => Some((w / 3, 0, w / 3, h / 6, Some(0))),
            2 => Some((0, 0, w, h >> 1, Some(0))),
            3 => Some((0, h >> 1, w, h >> 1, Some(1))),
            4 => Some((0, 0, w >> 1, h, Some(0))),
            5 => Some((w >> 1, 0, w >> 1, h, Some(1))),
            6 => Some((0, 0, w >> 1, h >> 1, None)),
            7 => Some((w >> 1, 0, w >> 1, h >> 1, None)),
            8 => Some((0, h >> 1, w >> 1, h >> 1, None)),
            9 => Some((w >> 1, h >> 1, w >> 1, h >> 1, None)),
            _ => None,
        };
        if let Some((x, y, vw, vh, p)) = r {
            (self.x, self.y, self.w, self.h) = (x, y, vw, vh);
            if let Some(p) = p {
                self.player = p;
            }
        }
    }
}

/// GameManager +588 / +988
#[derive(Resource, Clone, Debug, Default)]
pub struct GameViews {
    pub players: u32,
    pub views: [GameView; NUM_VIEWS],
    pub second: [GameView; NUM_VIEWS],
}

impl GameViews {
    /// GameManager.cpp 0x000B3EC0 (`vertical` = `byte_6F9180`, options block +0x24: 2 players side by side)
    pub fn init(&mut self, players: u32, vertical: bool) {
        self.players = players;
        for i in 0..NUM_VIEWS {
            self.views[i].used = false;
            self.views[i].kind = -1;
            self.second[i].used = false;
            self.second[i].kind = -1;
        }
        match players {
            1 => {
                self.views[0].set(0);
                self.second[0].used = false;
                self.second[0].kind = -1;
                self.second[0].set(0);
                self.views[1].set(1);
            }
            2 => {
                self.views[0].set(0);
                let (a, b) = if vertical { (4, 5) } else { (2, 3) };
                self.views[1].set(a);
                self.views[2].set(b);
                self.views[0].used = false;
                self.second[0].used = false;
                self.second[0].kind = -1;
                self.second[0].set(a);
                self.second[1].used = false;
                self.second[1].kind = -1;
                self.second[1].set(b);
            }
            3 | 4 => {
                self.views[0].set(0);
                for k in 0..players as usize {
                    self.views[1 + k].set(6 + k as i32);
                }
                self.views[0].used = false;
            }
            _ => {}
        }
    }

    /// the world view player `p` is drawn into: view 0 (full screen) for one player, else view 1 + p (kind 1, view
    /// 1 of a one-player race, is the rear-view mirror)
    pub fn player_view(&self, p: usize) -> Option<&GameView> {
        let v = if self.players <= 1 { self.views.first() } else { self.views.get(1 + p) }?;
        v.used.then_some(v)
    }

    /// `v` as a fraction of the screen (x, y, w, h)
    pub fn fraction(v: &GameView) -> Vec4 {
        Vec4::new(
            v.x as f32 / SCREEN_W as f32,
            v.y as f32 / SCREEN_H as f32,
            v.w as f32 / SCREEN_W as f32,
            v.h as f32 / SCREEN_H as f32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_players_split_top_bottom() {
        let mut v = GameViews::default();
        v.init(2, false);
        let a = v.player_view(0).unwrap();
        let b = v.player_view(1).unwrap();
        assert_eq!((a.x, a.y, a.w, a.h, a.player), (0, 0, 640, 240, 0));
        assert_eq!((b.x, b.y, b.w, b.h, b.player), (0, 240, 640, 240, 1));
        assert!(!v.views[0].used);
        v.init(2, true);
        let b = v.player_view(1).unwrap();
        assert_eq!((b.x, b.y, b.w, b.h, b.player), (320, 0, 320, 480, 1));
        v.init(1, false);
        let a = v.player_view(0).unwrap();
        assert_eq!((a.w, a.h), (640, 480));
        assert_eq!((v.views[1].x, v.views[1].w, v.views[1].h, v.views[1].kind), (213, 213, 80, 1));
    }
}
