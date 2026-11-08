//! Audio\_AUDIO_EventUtils.cpp and the event entry points: sound/sample/chain selection, starting events
//! (with chains and child events), stopping them and the per-frame active event update.

use super::system::*;
use super::xvg::select;

use event_flags as ef;

/// AUDIO_PlayEvent's "use the sound's own value" marker (-5000000.0f).
pub const DEFAULT_VALUE: f32 = -5_000_000.0;

/// fields shared by the sound-level selectors: (flags, count, last).
struct Pick {
    flags: u32,
    count: usize,
    last: i32,
}

impl AudioSystem {
    /// _AUDIO_EventUtils.cpp 0x002BC070 (_AUDIO_WeightedSelect)
    fn weighted_select(weights: &[u32], r: f32) -> usize {
        let total: u32 = weights.iter().sum();
        let target = (total as f64 * r as f64) as i32;
        let mut acc: i32 = 0;
        for (j, &w) in weights.iter().enumerate() {
            acc += w as i32;
            if acc >= target {
                return j;
            }
        }
        weights.len().saturating_sub(1)
    }

    /// _AUDIO_EventUtils.cpp 0x002BB920 / 0x002BBD50 / 0x002BBEE0: the common body of `_AUDIO_SelectSound`,
    /// `_AUDIO_SelectSample` and `_AUDIO_SelectChain`. returns the index and the new "last".
    fn select(&mut self, p: Pick, weights: &[u32]) -> i32 {
        if p.count == 1 {
            return 0;
        }
        if p.count == 0 {
            return -1;
        }
        let mut v: usize = 0;
        match p.flags & 0xF00 {
            select::RANDOM => loop {
                let r = self.random_float();
                v = if p.flags & select::WEIGHTED != 0 {
                    Self::weighted_select(weights, r)
                } else {
                    ((p.count - 1) as f64 * r as f64 + 0.5) as usize
                };
                if !(v as i32 == p.last && p.flags & select::NO_REPEAT != 0) {
                    break;
                }
            },
            select::SEQUENTIAL => v = (p.last + 1) as usize,
            select::PARAMETER => {
                let r = self.control_param((p.flags & 0xFF) as usize);
                v = if p.flags & select::WEIGHTED != 0 {
                    Self::weighted_select(weights, r)
                } else {
                    ((p.count - 1) as f64 * r as f64 + 0.5) as usize
                };
            }
            _ => {}
        }
        if v >= p.count {
            v = 0;
        }
        v as i32
    }

    fn select_sound(&mut self, r: EventRef) -> i32 {
        let Some(e) = self.event(r) else { return -1 };
        let p = Pick { flags: e.flags, count: e.sounds.len(), last: e.last };
        let w: Vec<u32> = e.sounds.iter().map(|s| s.def.weight).collect();
        let v = self.select(p, &w);
        self.event_mut(r).unwrap().last = if v < 0 { 0 } else { v };
        v
    }

    fn select_sample(&mut self, r: EventRef, sound: usize) -> i32 {
        let Some(s) = self.event(r).and_then(|e| e.sounds.get(sound)) else { return -1 };
        let p = Pick { flags: s.def.sample_flags, count: s.samples.len(), last: s.last_sample };
        let w: Vec<u32> = s.samples.iter().map(|x| x.weight).collect();
        let v = self.select(p, &w);
        self.event_mut(r).unwrap().sounds[sound].last_sample = if v < 0 { 0 } else { v };
        v
    }

    fn select_chain(&mut self, r: EventRef, sound: usize) -> i32 {
        let Some(s) = self.event(r).and_then(|e| e.sounds.get(sound)) else { return -1 };
        let p = Pick { flags: s.def.chain_flags, count: s.chains.len(), last: s.last_chain };
        let w: Vec<u32> = s.chains.iter().map(|x| x.weight).collect();
        let v = self.select(p, &w);
        self.event_mut(r).unwrap().sounds[sound].last_chain = if v < 0 { 0 } else { v };
        v
    }

    /// _AUDIO_EventUtils.cpp 0x002BBAB0 / 0x002BBB90 / 0x002BBC70: the common body of `_AUDIO_SoundVolume` (`default` 1.0),
    /// `_AUDIO_SoundPitch` and `_AUDIO_SoundDelay` (`default` 0.0): a value in [min, max] by the range's selection mode.
    fn range_value(&mut self, flags: u32, min: f32, max: f32, default: f32) -> f32 {
        if max == min {
            return min;
        }
        let mut t = default;
        match flags & 0xF00 {
            0x400 => return (min + max) / 2.0,
            0x500 => return max,
            0x300 => return min,
            0x200 => t = self.control_param((flags & 0xFF) as usize),
            0x000 => t = self.random_float(),
            _ => {}
        }
        (max - min) * t + min
    }

    fn sound_volume(&mut self, r: EventRef, sound: usize) -> f32 {
        let d = &self.event(r).unwrap().sounds[sound].def;
        let (f, a, b) = (d.volume_flags, d.volume_min, d.volume_max);
        self.range_value(f, a, b, 1.0)
    }
    fn sound_pitch(&mut self, r: EventRef, sound: usize) -> f32 {
        let d = &self.event(r).unwrap().sounds[sound].def;
        let (f, a, b) = (d.pitch_flags, d.pitch_min, d.pitch_max);
        self.range_value(f, a, b, 0.0)
    }
    fn sound_delay(&mut self, r: EventRef, sound: usize) -> f32 {
        let d = &self.event(r).unwrap().sounds[sound].def;
        let (f, a, b) = (d.delay_flags, d.delay_min, d.delay_max);
        self.range_value(f, a, b, 0.0)
    }

    /// _AUDIO_LinkActiveEvent.cpp 0x002BB5F0
    fn link_active_event(&mut self, e: EventId) {
        if let Some(h) = self.event_head {
            self.events.get_mut(h).unwrap().prev = Some(e);
        }
        let a = self.events.get_mut(e).unwrap();
        a.next = self.event_head;
        a.prev = None;
        self.event_head = Some(e);
    }

    /// _AUDIO_UnlinkActiveEvent.cpp 0x002BB630
    fn unlink_active_event(&mut self, e: EventId) {
        let (next, prev) = {
            let a = self.events.get(e).unwrap();
            (a.next, a.prev)
        };
        if let Some(n) = next {
            self.events.get_mut(n).unwrap().prev = prev;
        }
        match prev {
            Some(p) => self.events.get_mut(p).unwrap().next = next,
            None => self.event_head = next,
        }
        let a = self.events.get_mut(e).unwrap();
        a.next = None;
        a.prev = None;
    }

    /// _AUDIO_ReplaceEventHandle.cpp 0x002BB870: hand the owner's variable on to `to`.
    fn replace_event_handle(&mut self, e: EventId, to: Option<EventId>) {
        if let Some(s) = self.events.get(e).and_then(|a| a.owner) {
            if self.slot(s) == Some(e) {
                self.set_slot(s, to);
            }
        }
    }

    /// _AUDIO_AdoptChildEvents.cpp 0x002BB8A0: make `children` (a sibling chain) children of `parent`.
    fn adopt_child_events(&mut self, parent: EventId, children: EventId) {
        let mut i = Some(children);
        while let Some(c) = i {
            let a = self.events.get_mut(c).unwrap();
            a.parent = Some(parent);
            i = a.next_sibling;
        }
        match self.events.get(parent).unwrap().child {
            Some(mut j) => {
                while let Some(n) = self.events.get(j).unwrap().next_sibling {
                    j = n;
                }
                self.events.get_mut(j).unwrap().next_sibling = Some(children);
                self.events.get_mut(children).unwrap().prev_sibling = Some(j);
            }
            None => self.events.get_mut(parent).unwrap().child = Some(children),
        }
    }

    /// _AUDIO_DetachActiveEvent.cpp 0x002BB690: take an event out of its family, re-homing its children
    /// and its owner's handle.
    fn detach_active_event(&mut self, e: EventId) {
        let a = self.events.get_mut(e).unwrap();
        a.flags &= 0xFFFF_FFE0;
        let (parent, child, next_s, prev_s, owner) = (a.parent, a.child, a.next_sibling, a.prev_sibling, a.owner);
        if let Some(s) = owner {
            if parent.is_none() && child.is_none() && prev_s.is_none() && next_s.is_none() {
                self.set_slot(s, None);
            }
        }
        if let Some(p) = prev_s {
            self.events.get_mut(p).unwrap().next_sibling = next_s;
            self.replace_event_handle(e, next_s);
        }
        if let Some(n) = next_s {
            self.events.get_mut(n).unwrap().prev_sibling = prev_s;
            self.replace_event_handle(e, next_s);
        }
        if let Some(p) = parent {
            if self.events.get(p).unwrap().child == Some(e) {
                self.events.get_mut(p).unwrap().child = if next_s.is_some() { next_s } else { prev_s };
                self.replace_event_handle(e, Some(p));
            }
        }
        if let Some(c) = child {
            self.replace_event_handle(e, Some(c));
            if let Some(p) = parent {
                self.adopt_child_events(p, c);
            } else if let Some(n) = next_s {
                self.adopt_child_events(n, c);
            } else if let Some(p) = prev_s {
                self.adopt_child_events(p, c);
            } else {
                // the first child becomes the head of the family
                let rest = self.events.get(c).unwrap().next_sibling;
                self.events.get_mut(e).unwrap().child = rest;
                let v2 = self.events.get_mut(c).unwrap();
                v2.parent = None;
                v2.prev_sibling = None;
                v2.next_sibling = None;
                if let Some(r) = rest {
                    self.events.get_mut(r).unwrap().prev_sibling = None;
                    self.adopt_child_events(c, r);
                }
            }
        }
    }

    fn kill_active_event(&mut self, e: EventId) {
        self.detach_active_event(e);
        self.unlink_active_event(e);
        self.events.free(e);
    }

    /// _AUDIO_PlayEvent.cpp 0x00316FB0
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn play_event_internal(
        &mut self,
        emitter: EmitterId,
        hash: u32,
        callback: Option<EventCallback>,
        data: u32,
        out: Option<EventSlot>,
        parent: Option<EventId>,
        vol: f32,
        pitch: f32,
    ) -> i32 {
        if let Some(s) = out {
            self.set_slot(s, None);
        }
        let idx = self.find_event(hash);
        if idx < 0 {
            return err::UNKNOWN_EVENT;
        }
        let lookup = self.event_table[idx as usize];
        let Some(group) = lookup.group else { return err::UNKNOWN_EVENT };
        let r = lookup.event;
        let owner = match out {
            Some(s) => Some(s),
            None => parent.and_then(|p| self.events.get(p).and_then(|a| a.owner)),
        };
        let ae = ActiveEvent {
            next: None,
            flags: ef::ALIVE,
            prev: None,
            event: r,
            sound: 0,
            sample: None,
            delay: 0.0,
            parent: None,
            child: None,
            next_sibling: None,
            prev_sibling: None,
            emitter,
            playing: None,
            owner,
            callback: None,
            data: 0,
        };
        let Some(id) = self.events.alloc(ae) else { return err::NO_EVENT };
        if let Some(p) = parent {
            let first = self.events.get(p).unwrap().child;
            self.events.get_mut(id).unwrap().next_sibling = first;
            if let Some(f) = first {
                self.events.get_mut(f).unwrap().prev_sibling = Some(id);
            }
            self.events.get_mut(id).unwrap().parent = Some(p);
            self.events.get_mut(p).unwrap().child = Some(id);
        }
        let si = self.select_sound(r);
        if si < 0 {
            // the original frees without detaching, leaving parent->child dangling
            self.detach_active_event(id);
            self.events.free(id);
            return err::UNKNOWN_EVENT;
        }
        let si = si as usize;
        let smi = self.select_sample(r, si);
        let (mut v15, mut v20) = (0.0f32, 0.0f32);
        let mut sample: Option<GSample> = None;
        if smi >= 0 {
            let s = self.event(r).unwrap().sounds[si].samples[smi as usize].clone();
            let v16 = if vol == DEFAULT_VALUE { self.sound_volume(r, si) } else { vol };
            let gvol = self.groups.get(group).map(|g| g.volume).unwrap_or(0.0);
            v15 = v16 * gvol * s.volume;
            v20 = if pitch == DEFAULT_VALUE { self.sound_pitch(r, si) } else { pitch };
            sample = Some(s);
        }
        let sd = self.event(r).unwrap().sounds[si].def.clone();
        let nchains = self.event(r).unwrap().sounds[si].chains.len();
        if sd.chain_flags & super::xvg::chain_flags::ON_END != 0 && nchains != 0 {
            self.events.get_mut(id).unwrap().flags |= ef::CHAIN_ON_END;
        } else if sd.chain_flags & super::xvg::chain_flags::DELAYED != 0 && nchains != 0 {
            let d = self.sound_delay(r, si) + self.frame_time / 2.0;
            let a = self.events.get_mut(id).unwrap();
            a.delay = d;
            a.flags |= ef::DELAYED_CHAIN;
        }
        {
            let a = self.events.get_mut(id).unwrap();
            a.sound = si;
            a.sample = if smi >= 0 { Some(smi as usize) } else { None };
            a.callback = callback;
            a.data = data;
        }
        self.link_active_event(id);
        if let Some(s) = out {
            self.set_slot(s, Some(id));
        }
        let Some(s) = sample else { return 0 };
        let Some(bank) = s.bank else {
            self.kill_active_event(id);
            if let Some(o) = out {
                self.set_slot(o, None);
            }
            return err::BAD_BANK;
        };
        let event_flags_ = self.event(r).unwrap().flags;
        let mut fl: u16 = (event_flags_ & super::xvg::event_flags::LOOP != 0
            && sd.chain_flags & super::xvg::chain_flags::ON_END == 0) as u16;
        if data == 256 {
            fl |= 0x100;
        }
        if event_flags_ & super::xvg::event_flags::GLOBAL_PITCH != 0 {
            fl |= 0x200;
        }
        let category = self.groups.get(group).map(|g| g.category_mask).unwrap_or(0);
        let parent_queue = parent.and_then(|p| {
            let a = self.events.get(p)?;
            if a.flags & ef::CHAIN_ON_END != 0 { a.playing } else { None }
        });
        let result = if let Some(after) = parent_queue {
            match self.queue_sample(bank, s.sample as usize, after, fl, Some(SampleCallback::Event(id))) {
                Ok(_) => {
                    let mut i = Some(after);
                    let mut last = after;
                    while let Some(x) = i {
                        last = x;
                        i = self.samples.get(x).and_then(|p| p.queue_next);
                    }
                    self.events.get_mut(id).unwrap().playing = Some(last);
                    self.events.get_mut(parent.unwrap()).unwrap().flags |= 0x10;
                    self.events.get_mut(id).unwrap().flags |= ef::PLAYING;
                    0
                }
                Err(e) => e,
            }
        } else {
            match self.play_sample(bank, s.sample as usize, emitter, fl, category, v15, v20, Some(SampleCallback::Event(id))) {
                Ok(ps) => {
                    let a = self.events.get_mut(id).unwrap();
                    a.playing = Some(ps);
                    a.flags |= ef::PLAYING;
                    0
                }
                Err(e) => e,
            }
        };
        if result != 0 {
            self.kill_active_event(id);
            if let Some(o) = out {
                self.set_slot(o, None);
            }
        }
        result
    }

    /// AUDIO_PlayEvent.cpp 0x002BE440: start the event `hash` on `emitter`. returns `err::ALREADY_PLAYING`
    /// when `out` already holds an event.
    pub fn play_event(
        &mut self,
        emitter: EmitterId,
        hash: u32,
        callback: Option<EventCallback>,
        data: u32,
        out: Option<EventSlot>,
    ) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if let Some(s) = out {
            if self.slot(s).is_some() {
                return err::ALREADY_PLAYING;
            }
        }
        let r = self.play_event_internal(emitter, hash, callback, data, out, None, DEFAULT_VALUE, DEFAULT_VALUE);
        if let Some(t) = &mut self.trace_plays {
            t.push((hash, r));
        }
        r
    }

    /// AUDIO_StopEvent.cpp 0x002BCB80. `None` stops every active event (`-1`).
    pub fn stop_event(&mut self, e: Option<EventId>) -> i32 {
        if !self.initialised {
            return not_init();
        }
        match e {
            None => {
                while let Some(h) = self.event_head {
                    let r = self.stop_event(Some(h));
                    if r != 0 {
                        return r;
                    }
                }
            }
            Some(e) => {
                if !self.events.contains(e) {
                    return 0;
                }
                while let Some(c) = self.events.get(e).and_then(|a| a.child) {
                    self.stop_event(Some(c));
                }
                if let Some(ps) = self.events.get_mut(e).unwrap().playing.take() {
                    self.stop_sample(ps);
                }
                let a = self.events.get(e).unwrap();
                if let Some(cb) = a.callback {
                    cb(2, a.data);
                }
                self.kill_active_event(e);
            }
        }
        0
    }

    /// stop the event a caller's handle holds and clear it (the common `if (h) { StopEvent(h); h = 0; }`).
    pub fn stop_slot(&mut self, s: EventSlot) {
        if let Some(e) = self.slot(s) {
            self.stop_event(Some(e));
        }
        self.set_slot(s, None);
    }

    /// the playing sample of an event (`ev->pPlayingSamp`, +56).
    pub fn event_sample(&self, e: EventId) -> Option<SampleId> {
        self.events.get(e).and_then(|a| a.playing)
    }

    /// the sample of the event held by a slot.
    pub fn slot_sample(&self, s: EventSlot) -> Option<SampleId> {
        self.slot(s).and_then(|e| self.event_sample(e))
    }

    /// _AUDIO_EventSampleCallback.cpp 0x00342E10: the event's sample ended (1/3) or was killed (2).
    pub(crate) fn event_sample_callback(&mut self, code: i32, e: EventId) {
        let Some(a) = self.events.get(e).cloned() else { return };
        if code == 2 {
            self.events.get_mut(e).unwrap().flags |= 0xC;
            return;
        }
        {
            let a = self.events.get_mut(e).unwrap();
            a.flags &= !ef::PLAYING;
            a.playing = None;
        }
        let (nchains, cflags) = self
            .event(a.event)
            .map(|ev| (ev.sounds[a.sound].chains.len(), ev.sounds[a.sound].def.chain_flags))
            .unwrap_or((0, 0));
        if nchains != 0 && cflags & 0x300_0000 == 0 {
            let d = self.sound_delay(a.event, a.sound) + self.frame_time / 2.0;
            let x = self.events.get_mut(e).unwrap();
            x.delay = d;
            x.flags |= ef::DELAYED_CHAIN;
        }
        if self.events.get(e).unwrap().flags & ef::CHAIN_ON_END != 0 {
            let ci = self.select_chain(a.event, a.sound);
            if ci >= 0 {
                let target = self.event(a.event).unwrap().sounds[a.sound].chains[ci as usize].event as usize;
                let hash = self.groups.get(a.event.group).unwrap().events[target].hash;
                let r = self.play_event_internal(
                    a.emitter,
                    hash,
                    a.callback,
                    a.data,
                    None,
                    Some(e),
                    DEFAULT_VALUE,
                    DEFAULT_VALUE,
                );
                if let Some(x) = self.events.get_mut(e) {
                    if r != 0 {
                        x.flags |= ef::DONE;
                    } else {
                        x.flags &= !ef::CHAIN_ON_END;
                    }
                }
            }
        }
        if let Some(x) = self.events.get_mut(e) {
            if x.flags & ef::DELAYED_CHAIN == 0 {
                x.flags |= ef::DONE;
            }
        }
    }

    /// _AUDIO_UpdateActiveEvent.cpp 0x00314DF0
    pub(crate) fn update_active_event(&mut self, e: EventId) -> i32 {
        self.active_event_count += 1;
        let Some(a) = self.events.get(e).cloned() else { return 0 };
        let r = a.event;
        if a.flags & ef::PLAYING != 0 {
            if let Some(sd) = self.event(r).map(|ev| ev.sounds[a.sound].def.clone()) {
                if sd.volume_flags & 0xF00 == select::PARAMETER && sd.volume_flags & select::CONTINUOUS != 0 {
                    let mut v = self.control_param((sd.volume_flags & 0xFF) as usize) * (sd.volume_max - sd.volume_min)
                        + sd.volume_min;
                    v *= self.groups.get(r.group).map(|g| g.volume).unwrap_or(0.0);
                    if let Some(si) = a.sample {
                        v *= self.event(r).unwrap().sounds[a.sound].samples[si].volume;
                    }
                    if let Some(ps) = a.playing {
                        self.set_sample_volume(ps, v);
                    }
                }
                if sd.pitch_flags & 0xF00 == select::PARAMETER && sd.pitch_flags & select::CONTINUOUS != 0 {
                    let p = self.control_param((sd.pitch_flags & 0xFF) as usize) * (sd.pitch_max - sd.pitch_min)
                        + sd.pitch_min;
                    if let Some(ps) = a.playing {
                        self.set_sample_pitch(ps, p);
                    }
                }
            }
            if a.flags & ef::CHAIN_ON_END != 0
                && a.playing.is_none_or(|ps| self.samples.get(ps).is_some_and(|p| p.flags & 0x2000 != 0))
                && !self.paused
            {
                let ci = self.select_chain(r, a.sound);
                if ci >= 0 {
                    let target = self.event(r).unwrap().sounds[a.sound].chains[ci as usize].event as usize;
                    let hash = self.groups.get(r.group).unwrap().events[target].hash;
                    if self.play_event_internal(a.emitter, hash, a.callback, a.data, None, Some(e), DEFAULT_VALUE, DEFAULT_VALUE) == 0 {
                        self.events.get_mut(e).unwrap().flags &= !ef::CHAIN_ON_END;
                    }
                }
            }
        }
        if self.events.get(e).is_some_and(|x| x.flags & ef::DELAYED_CHAIN != 0) && !self.paused {
            let x = self.events.get_mut(e).unwrap();
            x.delay -= self.frame_time;
            if x.delay <= 0.0 {
                let ci = self.select_chain(r, a.sound);
                if ci >= 0 {
                    let target = self.event(r).unwrap().sounds[a.sound].chains[ci as usize].event as usize;
                    let hash = self.groups.get(r.group).unwrap().events[target].hash;
                    if self.play_event_internal(a.emitter, hash, a.callback, a.data, None, Some(e), DEFAULT_VALUE, DEFAULT_VALUE) == 0 {
                        let x = self.events.get_mut(e).unwrap();
                        x.flags &= !ef::DELAYED_CHAIN;
                        if x.flags & ef::PLAYING == 0 {
                            x.flags |= ef::DONE;
                        }
                    }
                }
            }
        }
        if let Some(x) = self.events.get(e) {
            if x.flags & ef::DONE != 0 {
                if let Some(cb) = x.callback {
                    if x.child.is_none() {
                        cb(1, x.data);
                    }
                }
                self.kill_active_event(e);
            }
        }
        0
    }
}
