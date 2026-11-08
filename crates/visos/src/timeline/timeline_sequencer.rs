//! Timeline\TimelineSequencer.cpp — plays a `FileTimeline`: per channel, per track, items become current at their
//! start time and finish at start + length. What an item does is the peer's business (`TimelinePeer`, the vtable
//! the game passes as "sequencer peer"); path items (kind 2) are evaluated here and handed to the peer as a matrix.

use glam::{Mat4, Vec3, Vec4};

use crate::timeline::file_tlf::{FileItem, FilePayload, FileTarget, FileTimeline, ITEM_KIND_BONE_ANIM, ITEM_KIND_PATH, ITEM_KIND_TYPED};

/// the peer interface (vtable offsets of the original in comments).
pub trait TimelinePeer {
    /// +0x00 BindToTarget: false fails the bind
    fn bind(&mut self, _channel: usize, _tl: &FileTimeline) -> bool {
        true
    }
    /// +0x04
    fn unbind(&mut self, _channel: usize) {}
    /// +0x1C current target matrix of the channel's object
    fn target_matrix(&mut self, _channel: usize) -> Mat4 {
        Mat4::IDENTITY
    }
    /// +0x20 path result
    fn set_target_matrix(&mut self, _channel: usize, _item: &FileItem, _m: Mat4) {}
    /// +0x24 typed event begins (local time, length)
    fn event_begin(&mut self, _channel: usize, _item: &FileItem, _t: f32, _length: f32) {}
    /// +0x28 typed event running (normalised time)
    fn event_update(&mut self, _channel: usize, _item: &FileItem, _u: f32) {}
    /// +0x2C typed event ends
    fn event_end(&mut self, _channel: usize, _item: &FileItem) {}
    /// bone animation items (TimelineSequencer.cpp 0x002B5810, owned by the animation code)
    fn bone_anim(&mut self, _channel: usize, _item: &FileItem, _local_t: f32) {}
}

/// `+0x04` state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ESequencerState {
    #[default]
    Unbound,
    Stopped,
    Playing,
}

/// per-track playback (the track's runtime +0x10 current item, +0x14 next index; -1 = disabled)
#[derive(Clone, Copy, Debug, Default)]
pub struct TrackState {
    pub current: Option<usize>,
    pub next: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TimelineSequencer {
    pub state: ESequencerState,      // +0x04
    pub prev_state: ESequencerState, // +0x08
    pub time: f32,                   // +0x0C
    pub prev_time: f32,              // +0x10
    pub played: bool,                // +0x1C
    /// [channel][track]
    pub tracks: Vec<Vec<TrackState>>,
}

fn is_typed_with_mode(item: &FileItem, mode: u32) -> bool {
    matches!(item.payload, FilePayload::Typed { mode: m, .. } if m == mode)
}

impl TimelineSequencer {
    /// TimelineSequencer.cpp 0x002B4970 BindToTimeline
    pub fn bind(&mut self, tl: &FileTimeline, peer: &mut dyn TimelinePeer) -> bool {
        self.tracks = tl.channels.iter().map(|c| vec![TrackState::default(); c.tracks.len()]).collect();
        for c in 0..tl.channels.len() {
            peer.unbind(c);
            if !peer.bind(c, tl) {
                log::warn!("[TimelineSequencer::BindToTimeline] BindToTarget() failed for channel {} ({:?})", c, tl.channels[c].name);
                return false;
            }
        }
        self.time = 0.0;
        self.prev_time = 0.0;
        self.prev_state = self.state;
        self.state = ESequencerState::Stopped;
        self.seek(tl, peer);
        true
    }

    /// TimelineSequencer.cpp 0x002B48B0 — timeline length
    pub fn length(tl: &FileTimeline) -> f32 {
        tl.length
    }

    /// TimelineSequencer.cpp 0x002B4CF0 Play
    pub fn play(&mut self, tl: &FileTimeline, peer: &mut dyn TimelinePeer) -> bool {
        if self.state == ESequencerState::Unbound {
            log::warn!("[TimelineSequencer::Play] No timeline bound to sequencer");
            return false;
        }
        if Self::length(tl) > self.time {
            self.played = true;
            self.prev_state = self.state;
            self.state = ESequencerState::Playing;
            self.seek(tl, peer);
            true
        } else {
            log::warn!("[TimelineSequencer::Play] Sequencer already at end of timeline");
            false
        }
    }

    /// TimelineSequencer.cpp 0x002B4D70 Stop
    pub fn stop(&mut self, tl: &FileTimeline, peer: &mut dyn TimelinePeer) {
        self.prev_state = self.state;
        self.state = ESequencerState::Stopped;
        self.seek(tl, peer);
    }

    /// TimelineSequencer.cpp 0x002B4DA0 Rewind / 0x002B4E00 SetTime
    pub fn set_time(&mut self, t: f32, tl: &FileTimeline, peer: &mut dyn TimelinePeer) {
        self.time = t;
        self.prev_time = t;
        self.seek(tl, peer);
    }

    /// TimelineSequencer.cpp 0x002B4B30 / 0x002B5E70 — re-derive each track's current item for the current time.
    fn seek(&mut self, tl: &FileTimeline, peer: &mut dyn TimelinePeer) {
        let t = self.time;
        for (c, ch) in tl.channels.iter().enumerate() {
            for (k, tr) in ch.tracks.iter().enumerate() {
                let st = &mut self.tracks[c][k];
                if tr.items.is_empty() || st.next < 0 {
                    continue;
                }
                let first = &tr.items[0];
                let last = &tr.items[tr.items.len() - 1];
                if t < first.start {
                    st.current = None;
                    st.next = 0;
                    continue;
                }
                if last.start + last.length < t {
                    st.current = None;
                    st.next = tr.items.len() as i32;
                    continue;
                }
                let mut found = false;
                for i in (0..tr.items.len()).rev() {
                    let it = &tr.items[i];
                    if it.start <= t {
                        if it.start + it.length >= t {
                            found = true;
                            if it.kind == ITEM_KIND_TYPED {
                                let cur = st.current.map(|ci| &tr.items[ci]);
                                seek_typed(self.state, self.prev_state, c, it, cur, t, peer);
                            }
                            st.current = Some(i);
                            st.next = i as i32;
                            if it.kind == ITEM_KIND_PATH {
                                path_update(tl, c, it, t, peer);
                            }
                        } else {
                            st.next = i as i32 + 1;
                        }
                        break;
                    }
                }
                if !found {
                    if let Some(cur) = st.current {
                        let it = &tr.items[cur];
                        if it.kind == ITEM_KIND_TYPED {
                            peer.event_end(c, it);
                        }
                    }
                    st.current = None;
                }
            }
        }
    }

    /// TimelineSequencer.cpp 0x002B4BB0 Update
    pub fn update(&mut self, dt: f32, tl: &FileTimeline, peer: &mut dyn TimelinePeer) {
        if self.state != ESequencerState::Playing {
            return;
        }
        self.prev_time = self.time;
        self.time += dt;
        let t = self.time;
        // TimelineSequencer.cpp 0x002B5770: channels only while the timeline runs
        if Self::length(tl) <= t {
            return;
        }
        for (c, ch) in tl.channels.iter().enumerate() {
            for (k, tr) in ch.tracks.iter().enumerate() {
                let st = &mut self.tracks[c][k];
                // TimelineSequencer.cpp 0x002B6140 UpdateTrack
                if tr.items.is_empty() {
                    continue;
                }
                if let Some(cur) = st.current {
                    let it = &tr.items[cur];
                    let end = it.start + it.length;
                    if t > end {
                        if it.kind == ITEM_KIND_TYPED && is_typed_with_mode(it, 1) {
                            peer.event_end(c, it);
                        }
                        st.current = None;
                        st.next += 1;
                    } else if it.kind == ITEM_KIND_PATH {
                        path_update(tl, c, it, t, peer);
                    } else if it.kind == ITEM_KIND_TYPED {
                        // TimelineSequencer.cpp 0x002B6C50
                        if !is_typed_with_mode(it, 2) {
                            peer.event_update(c, it, (t - it.start) / it.length);
                        }
                    } else if it.kind == ITEM_KIND_BONE_ANIM {
                        peer.bone_anim(c, it, t - it.start);
                    }
                }
                if st.current.is_none() && st.next >= 0 && (st.next as usize) < tr.items.len() {
                    let it = &tr.items[st.next as usize];
                    if t >= it.start {
                        st.current = Some(st.next as usize);
                        match it.kind {
                            ITEM_KIND_PATH => path_update(tl, c, it, it.start, peer),
                            ITEM_KIND_TYPED => peer.event_begin(c, it, 0.0, it.length),
                            ITEM_KIND_BONE_ANIM => peer.bone_anim(c, it, 0.0),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// TimelineSequencer.cpp 0x002B6040 — a seek landed inside typed item `it`: playing → begin it at its local time;
/// the end call for the track's previous item (playing after playing, or stopped after playing) tests that item's
/// kind against 1 (bone anim), not 4 (typed) — original bug kept, so on single-kind tracks it never fires
fn seek_typed(
    state: ESequencerState,
    prev: ESequencerState,
    c: usize,
    it: &FileItem,
    cur: Option<&FileItem>,
    t: f32,
    peer: &mut dyn TimelinePeer,
) {
    let cur_end = cur.filter(|i| i.kind == ITEM_KIND_BONE_ANIM);
    if state == ESequencerState::Playing {
        if prev == ESequencerState::Playing {
            if let Some(ci) = cur_end {
                peer.event_end(c, ci);
            }
        }
        peer.event_begin(c, it, t - it.start, it.length);
    } else if state == ESequencerState::Stopped && prev == ESequencerState::Playing {
        if let Some(ci) = cur_end {
            peer.event_end(c, ci);
        }
    }
}

/// TimelineSequencer.cpp 0x002B67B0 — sine ease in/out over the item's normalised time.
pub fn ease(ease_in: f32, ease_out: f32, a: f32) -> f32 {
    use std::f32::consts::PI;
    if a >= ease_in {
        if 1.0 - ease_out >= a {
            a
        } else {
            let x = (1.0 - a) / ease_out * PI / 2.0;
            1.0 - (1.0 - a) * (x as f64).sin() as f32
        }
    } else {
        let x = a / ease_in * PI / 2.0;
        a * (x as f64).sin() as f32
    }
}

/// TimelineSequencer.cpp 0x002B6860 — Catmull-Rom through the target's points at segment `seg` + `frac`.
pub fn spline_point(target: &FileTarget, seg: u32, frac: f32) -> Vec4 {
    let n = target.points.len() as u32;
    if n == 0 {
        return Vec4::new(0.0, 0.0, 0.0, 1.0);
    }
    let mut v = (seg as f64 + frac as f64).abs();
    while (n as f64) <= v {
        v -= n as f64;
    }
    let fl = v as i64;
    let f = v - fl as f64;
    let f2 = f * f;
    let f3 = f * f2;
    let b0 = -0.5 * f3 + f2 - 0.5 * f + 0.0;
    let b1 = 1.5 * f3 - 2.5 * f2 + 0.0 * f + 1.0;
    let b2 = -1.5 * f3 + 2.0 * f2 + 0.5 * f + 0.0;
    let b3 = 0.5 * f3 - 0.5 * f2 + 0.0 * f + 0.0;
    let i1 = fl as i64;
    let (mut i0, mut i2, mut i3) = (i1 - 1, i1 + 1, i1 + 2);
    let n = n as i64;
    if target.circular {
        if i0 < 0 {
            i0 += n;
        }
        i2 %= n;
        i3 %= n;
    } else {
        if i0 < 0 {
            i0 = 0;
        }
        i2 = i2.min(n - 1);
        i3 = i3.min(n - 1);
    }
    let p = |i: i64| {
        let q = target.points[i as usize];
        Vec4::new(q[0], q[1], q[2], 1.0)
    };
    let r = p(i0) * b0 as f32 + p(i1) * b1 as f32 + p(i2) * b2 as f32 + p(i3) * b3 as f32;
    Vec4::new(r.x, r.y, r.z, 1.0)
}

/// TimelineSequencer.cpp 0x002B6BA0 — point at normalised position `t` along the whole path.
pub fn spline_at(target: &FileTarget, t: f32) -> Vec4 {
    let n = target.points.len() as u32;
    let span = if target.circular { n } else { n.saturating_sub(1) };
    let v = span as f64 * t as f64;
    let seg = v as u32;
    spline_point(target, seg, (v - seg as f64) as f32)
}

/// TimelineSequencer.cpp 0x002B6310 UpdatePath — a path item moves the channel's object along its target's spline.
fn path_update(tl: &FileTimeline, c: usize, item: &FileItem, t: f32, peer: &mut dyn TimelinePeer) {
    let FilePayload::Path { target_crc, ease_in, ease_out, start, end, flags, axis_flags } = item.payload else { return };
    let Some(target) = tl.targets.iter().find(|x| x.crc == target_crc) else { return };
    let mut lt = t - item.start;
    if lt > item.length {
        lt = item.length;
    }
    let u = lt / item.length;
    let e = ease(ease_in, ease_out, u);
    let mut s = (end - start) * e + start;
    if s > 0.999 {
        s = 0.999;
    }
    let pos = spline_at(target, s);
    let ahead = spline_at(target, s + 0.001);
    let current = peer.target_matrix(c);
    let fwd = (ahead - pos).truncate().normalize_or_zero();
    let right = Vec3::Y.cross(fwd).normalize_or_zero();
    let up = fwd.cross(right).normalize_or_zero();
    // D3D rows (row 3 = translation); in glam these are the columns (same floats in memory order)
    let mut m = Mat4::IDENTITY;
    if flags & 1 != 0 || flags & 0x10 != 0 || flags & 8 != 0 {
        m.w_axis = pos;
    }
    if flags & 2 != 0 && flags & 0x10 == 0 && flags & 8 == 0 {
        m.x_axis = right.extend(0.0);
        m.y_axis = up.extend(0.0);
        m.z_axis = fwd.extend(0.0);
    }
    let w = Mat4::from_cols_array_2d(&target.world);
    // D3DXMatrixMultiply(out, m, world) on row vectors = world * m on columns
    let mut out = w * m;
    if flags & 1 != 0 {
        // axes the item does not move keep the object's current translation
        let cur = current.w_axis;
        if axis_flags & 1 == 0 {
            out.w_axis.x = cur.x;
        }
        if axis_flags & 2 == 0 {
            out.w_axis.y = cur.y;
        }
        if axis_flags & 4 == 0 {
            out.w_axis.z = cur.z;
        }
    }
    peer.set_target_matrix(c, item, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::file_tlf::{FileChannel, FileTrack};

    #[derive(Default)]
    struct Log(Vec<String>);
    impl TimelinePeer for Log {
        fn event_begin(&mut self, c: usize, _i: &FileItem, t: f32, _l: f32) {
            self.0.push(format!("begin {c} {t}"));
        }
        fn event_end(&mut self, c: usize, _i: &FileItem) {
            self.0.push(format!("end {c}"));
        }
    }

    #[test]
    fn items_fire_in_order() {
        let item = |s: f32, mode| FileItem {
            kind: ITEM_KIND_TYPED,
            start: s,
            length: 0.5,
            payload: FilePayload::Typed { crc: 0, mode, type_ref: None, value: None },
        };
        let tl = FileTimeline {
            version: 5,
            name_crc: 0,
            name: None,
            type_refs: vec![],
            targets: vec![],
            channels: vec![FileChannel {
                crc: 0,
                name: None,
                type_crc: 0,
                str_a: None,
                str_b: None,
                tracks: vec![FileTrack { crc: 0, name: None, items: vec![item(1.0, 1), item(2.0, 0)] }],
            }],
            length: 5.0,
        };
        let mut s = TimelineSequencer::default();
        let mut p = Log::default();
        assert!(s.bind(&tl, &mut p));
        assert!(s.play(&tl, &mut p));
        for _ in 0..72 {
            s.update(1.0 / 24.0, &tl, &mut p);
        }
        assert_eq!(p.0, vec!["begin 0 0", "end 0", "begin 0 0"]);
    }

    /// a path item on every axis puts the object on the spline: translation in `w_axis` (glam convention)
    #[test]
    fn path_moves_the_object() {
        use crate::timeline::file_tlf::FileTarget;
        #[derive(Default)]
        struct M(Vec<Mat4>);
        impl TimelinePeer for M {
            fn set_target_matrix(&mut self, _c: usize, _i: &FileItem, m: Mat4) {
                self.0.push(m);
            }
        }
        let item = FileItem {
            kind: ITEM_KIND_PATH,
            start: 0.0,
            length: 4.0,
            payload: FilePayload::Path { target_crc: 7, ease_in: 0.0, ease_out: 0.0, start: 0.0, end: 1.0, flags: 1, axis_flags: 7 },
        };
        let mut world = [[0.0f32; 4]; 4];
        for (i, r) in world.iter_mut().enumerate() {
            r[i] = 1.0;
        }
        world[3] = [0.0, 0.0, 100.0, 1.0];
        let tl = FileTimeline {
            version: 5,
            name_crc: 0,
            name: None,
            type_refs: vec![],
            targets: vec![FileTarget { crc: 7, name: None, world, circular: false, points: vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]] }],
            channels: vec![FileChannel {
                crc: 0,
                name: None,
                type_crc: 0,
                str_a: None,
                str_b: None,
                tracks: vec![FileTrack { crc: 0, name: None, items: vec![item] }],
            }],
            length: 5.0,
        };
        let mut s = TimelineSequencer::default();
        let mut p = M::default();
        assert!(s.bind(&tl, &mut p));
        assert!(s.play(&tl, &mut p));
        for _ in 0..48 {
            s.update(1.0 / 24.0, &tl, &mut p);
        }
        let last = *p.0.last().expect("path matrices");
        let t = last.w_axis;
        assert!(t.x > 4.0 && t.x < 10.0 && t.z == 100.0, "{last:?}");
    }

    #[test]
    fn ease_ends() {
        assert_eq!(ease(0.2, 0.2, 0.5), 0.5);
        assert!(ease(0.2, 0.2, 0.0).abs() < 1e-6);
        assert!((ease(0.2, 0.2, 1.0) - 1.0).abs() < 1e-6);
    }
}
