//! `VsMovieStream` (VsMovieStreamXBOX.cpp): a queue of movie names, the open Bink handle (+132) and the
//! per-frame update. Bink calls: BinkOpen 0x0053E9A0, BinkWait 0x0053FFB0, BinkDoFrame 0x00541340,
//! BinkNextFrame 0x00541900, BinkCopyToBuffer 0x00541300, BinkGetRealtime 0x005404A0, BinkGetSummary
//! 0x005402D0, BinkSetVolume 0x00540D30, BinkClose 0x00541C60, BinkSoundUseDirectSound 0x0053E690,
//! BinkSetMemory 0x0053E670, BinkGetError 0x0053E150.
//!
//! Render modes (+84 / requested +88): 1 copy to a 32-bit buffer (BINKSURFACE32, format 3), 2 full screen
//! (YUY2 surface, format 13), 3 into a texture (+96; Bink surface from the texture's D3DFORMAT, 0x002F40B0: the TFT
//! texture "movie_test" is `D3DFMT_LIN_A8R8G8B8` (18) -> BINKSURFACE32A (5), the same BINK32 selector 0x00352260 as
//! format 3, X byte 0). The port always produces RGBA; the caller shows it. STAND-IN: mode 2 (YUY2, converted by
//! the GPU, opaque) is produced by the BINK32 maths with the X byte 255.
//! Timing (BinkWait): frame `n` is due `n * fps_den / fps_num` seconds of real time after the open.

use crate::bink::bink_audio::BinkAudio;
use crate::bink::{BinkFile, BinkTables, BinkVideo};
use std::collections::VecDeque;

/// D3DFORMAT of the TFT movie texture "movie_test" (`CTextureManager::Register` 0x0029F2E0 from OSD3dInterface
/// 0x000E9C90: 64x64, one level, format 18 passed straight to CreateTexture = `D3DFMT_LIN_A8R8G8B8`)
pub const MOVIE_TEXTURE_FORMAT: u32 = 18;

/// VsMovieStreamXBOX.cpp 0x002F40B0 — the Bink surface type for a texture's D3DFORMAT (swizzled / linear pairs)
pub fn bink_surface_type(d3dfmt: u32) -> u32 {
    match d3dfmt {
        2 | 16 => 8,
        3 | 28 => 9,
        4 | 29 => 7,
        5 | 17 => 10,
        6 | 18 => 5,
        7 | 30 => 3,
        36 => 13,
        37 => 14,
        _ => {
            log::error!("assert !\"Inappropriate texture format for movie rendering\" (VsMovieStreamXBOX.cpp 1111)");
            3
        }
    }
}

/// the top byte the surface's row blitter writes: selector 0x00352260 (surfaces 3 / 5 / 4 / 6) unpacks with
/// `qword_7D2A48` = 0. The other blitters (16-bit, YUY2) have no 32-bit top byte and are not ported.
fn surface_x_byte(surface: u32) -> u8 {
    match surface & 0xF {
        3..=6 => 0,
        s => {
            log::warn!("Bink surface {s}: row blitter not ported, BINK32 used");
            0
        }
    }
}

pub struct Movie {
    pub name: String,
    pub file: BinkFile,
    pub video: BinkVideo,
    pub audio: Vec<BinkAudio>,
    /// next frame to decode (Bink's FrameNum - 1)
    pub next: usize,
    /// real seconds since the open
    pub clock: f64,
}

#[derive(Default)]
pub struct VsMovieStream {
    /// +8 filename queue
    pub queue: VecDeque<String>,
    /// +88 requested render mode
    pub mode: u32,
    /// +108 loop the movie
    pub looping: bool,
    /// +132
    pub bink: Option<Movie>,
    /// +140 frames shown
    pub frames_shown: u32,
}

impl VsMovieStream {
    /// VsMovieStreamXBOX.cpp 0x002F4C90 `queSource`
    pub fn que_source(&mut self, name: &str) {
        self.queue.push_back(name.to_string());
    }

    /// VsMovieStreamXBOX.cpp 0x002F4BD0
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// VsMovieStreamXBOX.cpp 0x002F3CC0 open `\Movies\PAL\<name>.bik` (BinkOpen flags 0: no alpha, header scaling)
    pub fn open(&mut self, t: &BinkTables, name: &str) -> bool {
        self.close();
        let path = format!(r"\Movies\PAL\{name}.bik");
        let file = match visos::file::platform::read(&path).map_err(|e| e.to_string()).and_then(|d| BinkFile::parse(d, 0).map_err(|e| e.to_string())) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("BinkOpen failed with movie \"{name}\"! {e}");
                return false;
            }
        };
        let video = BinkVideo::new(&file);
        let audio = file.tracks.iter().map(|tr| BinkAudio::new(t, tr)).collect();
        self.frames_shown = 0;
        self.bink = Some(Movie { name: name.to_string(), file, video, audio, next: 0, clock: 0.0 });
        true
    }

    /// VsMovieStreamXBOX.cpp 0x002F3E40 BinkClose
    pub fn close(&mut self) {
        self.bink = None;
    }

    /// VsMovieStreamXBOX.cpp 0x002F3970: open the next queued movie when none is playing
    pub fn start_next(&mut self, t: &BinkTables) -> bool {
        while self.bink.is_none() {
            let Some(name) = self.queue.pop_front() else { return false };
            self.open(t, &name);
        }
        true
    }

    /// VsMovieStreamXBOX.cpp 0x002F3720 / 0x002F3E80 — per render frame with the real frame delta: decode every frame that is due (BinkWait /
    /// BinkDoFrame / BinkNextFrame), audio into `pcm` (interleaved at the first track's rate), the newest
    /// picture into `rgba`. Returns (still playing, picture changed).
    pub fn update(&mut self, t: &BinkTables, dt: f64, rgba: &mut Vec<u8>, pcm: &mut Vec<i16>) -> (bool, bool) {
        let Some(m) = self.bink.as_mut() else { return (false, false) };
        m.clock += dt;
        let fps = m.file.fps_num.max(1) as f64 / m.file.fps_den.max(1) as f64;
        let mut changed = false;
        let frames = m.file.frames as usize;
        // BinkWait: the next frame waits for its time; the first frame shows at once. A looping movie goes back to
        // frame 1 when the frame after the last is due, so the last frame is shown (FrameNum == Frames) until then
        loop {
            if m.next >= frames && self.looping && frames > 0 && frames as f64 <= m.clock * fps {
                m.next = 0;
                m.clock -= frames as f64 / fps;
            }
            if !(m.next < frames && m.next as f64 <= m.clock * fps) {
                break;
            }
            let Ok(pk) = m.file.packet(m.next) else { break };
            for (i, (a, d)) in m.audio.iter_mut().zip(&pk.audio).enumerate() {
                if i == 0 {
                    a.decode_packet(t, d, pcm);
                }
            }
            m.video.decode(t, pk.video);
            m.next += 1;
            self.frames_shown += 1;
            changed = true;
        }
        if changed {
            // BinkCopyToBuffer's surface (0x002F3720): mode 1 format 3, mode 3 the texture's (0x002F3E80), mode 2 YUY2
            let x = match self.mode {
                2 => 255,
                3 => surface_x_byte(bink_surface_type(MOVIE_TEXTURE_FORMAT)),
                _ => surface_x_byte(3),
            };
            m.video.to_rgba(rgba, x);
        }
        // `LastFrameNum != Frames || +108`
        let playing = m.next < frames || self.looping;
        if !playing {
            self.close();
        }
        (playing, changed)
    }

    /// VsMovie +112 / +116 (read by OSD3dInterface 0x000EB040): Bink's FrameNum of the frame shown and Frames
    pub fn frame_counts(&self) -> (u32, u32) {
        self.bink.as_ref().map_or((0, 0), |m| (m.next as u32, m.file.frames))
    }
}
