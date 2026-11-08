//! wma song streaming (replaces the xdk wma decoder xmo, `WMADEC` section, that `VsSoundtrackPlayer` feeds
//! its two dsound streams from). a minimal asf demuxer extracts the audio stream's `nBlockAlign` codec
//! packets; `oxideav-wma` decodes them on a worker thread into pcm chunks queued on the music voice.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use oxideav_wma::header::Version;
use oxideav_wma::packet::PacketAssembler;
use oxideav_wma::stream_config::StreamConfig;
use oxideav_wma::vendor_decode::BlockSynth;
use oxideav_wma::vendor_frame::FrameParser;

use crate::audio::mixer::{SampleData, SharedMixer};
use crate::audio::xbox::voice::MUSIC_VOICE;

const ASF_HEADER: [u8; 16] = [0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C];
const ASF_DATA: [u8; 16] = [0x36, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C];
const ASF_STREAM_PROPS: [u8; 16] = [0x91, 0x07, 0xDC, 0xB7, 0xB7, 0xA9, 0xCF, 0x11, 0x8E, 0xE6, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];
const ASF_FILE_PROPS: [u8; 16] = [0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65];

/// the audio stream of an asf file: WAVEFORMATEX fields and its codec packets.
pub struct AsfAudio {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub avg_bytes: u32,
    pub block_align: u16,
    pub flags2: u16,
    pub packets: Vec<u8>,
}

fn rd16(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?))
}
fn rd32(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}
fn rd64(b: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(o..o + 8)?.try_into().ok()?))
}

/// parse an asf file far enough to get the single audio stream's payload bytes.
pub fn demux(b: &[u8]) -> Option<AsfAudio> {
    if b.get(..16)? != ASF_HEADER {
        return None;
    }
    let hsize = rd64(b, 16)? as usize;
    let nobj = rd32(b, 24)?;
    let mut o = 30;
    let mut fmt: Option<&[u8]> = None;
    let mut pkt = 0usize;
    for _ in 0..nobj {
        let g = b.get(o..o + 16)?;
        let sz = rd64(b, o + 16)? as usize;
        if g == ASF_STREAM_PROPS {
            let tsl = rd32(b, o + 64)? as usize;
            let ts = o + 78;
            fmt = b.get(ts..ts + tsl);
        } else if g == ASF_FILE_PROPS {
            pkt = rd32(b, o + 92)? as usize;
        }
        o += sz;
    }
    let w = fmt?;
    let mut a = AsfAudio {
        format_tag: rd16(w, 0)?,
        channels: rd16(w, 2)?,
        sample_rate: rd32(w, 4)?,
        avg_bytes: rd32(w, 8)?,
        block_align: rd16(w, 12)?,
        flags2: rd16(w, 22).unwrap_or(0),
        packets: Vec::new(),
    };
    if b.get(hsize..hsize + 16)? != ASF_DATA || pkt == 0 {
        return None;
    }
    let count = rd64(b, hsize + 40)? as usize;
    let var = |p: &mut usize, t: u8| -> Option<usize> {
        Some(match t {
            1 => {
                *p += 1;
                *b.get(*p - 1)? as usize
            }
            2 => {
                *p += 2;
                rd16(b, *p - 2)? as usize
            }
            3 => {
                *p += 4;
                rd32(b, *p - 4)? as usize
            }
            _ => 0,
        })
    };
    let mut p = hsize + 50;
    for _ in 0..count {
        let s = p;
        let mut q = p;
        let ec = *b.get(q)?;
        q += 1;
        if ec & 0x80 != 0 {
            q += (ec & 0x0F) as usize;
        }
        let lf = *b.get(q)?;
        let pf = *b.get(q + 1)?;
        q += 2;
        var(&mut q, (lf >> 5) & 3)?;
        var(&mut q, (lf >> 1) & 3)?;
        let pad = var(&mut q, (lf >> 3) & 3)?;
        q += 6; // send time, duration
        let multi = lf & 1 != 0;
        let (n, len_type) = if multi {
            let c = *b.get(q)?;
            q += 1;
            ((c & 0x3F) as usize, c >> 6)
        } else {
            (1, 0)
        };
        for _ in 0..n {
            q += 1; // stream number
            var(&mut q, (pf >> 4) & 3)?;
            var(&mut q, (pf >> 2) & 3)?;
            let rl = var(&mut q, pf & 3)?;
            q += rl;
            let len = if multi { var(&mut q, len_type)? } else { pkt.checked_sub(q - s + pad)? };
            a.packets.extend_from_slice(b.get(q..q + len)?);
            q += len;
        }
        p = s + pkt;
    }
    Some(a)
}

/// decode a demuxed wma v2 stream, handing pcm chunks (interleaved s16) to `sink` as frames complete.
pub fn decode(a: &AsfAudio, mut sink: impl FnMut(Vec<i16>) -> bool) -> Option<()> {
    if a.format_tag != 0x161 || a.block_align == 0 {
        return None;
    }
    let ch = a.channels as usize;
    let cfg = StreamConfig::derive(Version::V2, a.sample_rate, ch as u8, a.avg_bytes, a.block_align, a.flags2).ok()?;
    let mut asm = PacketAssembler::new(&cfg);
    for pk in a.packets.chunks_exact(a.block_align as usize) {
        asm.push_packet(pk).ok()?;
    }
    let st = asm.finish();
    let body: Vec<u64> = st.packets.iter().map(|p| p.body_start_bit).collect();
    let mut parser = FrameParser::new(&cfg, &body);
    let mut synth = BlockSynth::new(&cfg);
    let mut cursor = st.packets.first()?.frames_start_bit();
    let mut clean = false;
    let emit = |chans: Vec<Vec<f64>>| -> Vec<i16> {
        let n = chans.first().map(|c| c.len()).unwrap_or(0);
        let mut v = Vec::with_capacity(n * ch);
        for i in 0..n {
            for c in &chans {
                v.push((c[i].clamp(-1.0, 1.0) * 32767.0) as i16);
            }
        }
        v
    };
    for (i, rec) in st.packets.iter().enumerate() {
        if cursor != rec.frames_start_bit() {
            cursor = rec.frames_start_bit();
            parser.raise_latch();
            if !clean {
                synth.reset();
            }
        }
        let mut r = st.reader_at(cursor);
        let mut failed = false;
        let mut chunk: Vec<i16> = Vec::new();
        for f in 0..rec.header.frame_count {
            match parser.parse_frame(&mut r) {
                Ok(fr) => {
                    for bl in &fr.blocks {
                        chunk.extend(emit(synth.block(bl)));
                    }
                }
                Err(_) => {
                    let rem = (rec.header.frame_count - f) as usize;
                    chunk.extend(std::iter::repeat_n(0, cfg.frame_length as usize * rem * ch));
                    synth.reset();
                    failed = true;
                    break;
                }
            }
        }
        cursor = r.position() as u64;
        clean = !failed && st.packets.get(i + 1).is_some_and(|n| n.header.carry_bits == 0 && cursor <= n.body_start_bit);
        if !chunk.is_empty() && !sink(chunk) {
            return Some(());
        }
    }
    let tail = emit(synth.flush());
    if !tail.is_empty() {
        sink(tail);
    }
    Some(())
}

/// a song playing on the music voice.
pub struct WmaStream {
    mixer: SharedMixer,
    stop: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

impl WmaStream {
    /// read and start decoding `path` (a game path like `D:\Audio\Music\X.wma`).
    pub fn open(path: &str, mixer: SharedMixer) -> Option<Self> {
        let bytes = visos::file::platform::read(path).ok()?;
        let a = demux(&bytes)?;
        let fmt = (xva_rate_index(a.sample_rate) << 12) | (16 << 7) | ((a.channels as u32) << 4);
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));
        {
            let mut m = mixer.lock().ok()?;
            m.set_buffer(MUSIC_VOICE, None);
            m.set_freq(MUSIC_VOICE, a.sample_rate as f32);
        }
        let (mx, st, dn) = (mixer.clone(), stop.clone(), done.clone());
        std::thread::Builder::new()
            .name("wma".into())
            .spawn(move || {
                let mut first = true;
                let mut pending: Vec<i16> = Vec::new();
                decode(&a, |pcm| {
                    if st.load(Ordering::Relaxed) {
                        return false;
                    }
                    pending.extend(pcm);
                    // hand over about a quarter second at a time
                    if pending.len() >= a.sample_rate as usize / 2 {
                        push(&mx, std::mem::take(&mut pending), fmt, &mut first);
                    }
                    true
                });
                if !pending.is_empty() && !st.load(Ordering::Relaxed) {
                    push(&mx, pending, fmt, &mut first);
                }
                dn.store(true, Ordering::Relaxed);
            })
            .ok()?;
        Some(Self { mixer, stop, done })
    }

    pub fn finished(&self) -> bool {
        self.done.load(Ordering::Relaxed) && !self.mixer.lock().map(|m| m.is_playing(MUSIC_VOICE)).unwrap_or(false)
    }

    pub fn set_gain(&self, g: f32) {
        if let Ok(mut m) = self.mixer.lock() {
            m.set_gains(MUSIC_VOICE, g, g);
        }
    }

    pub fn set_paused(&self, paused: bool) {
        if let Ok(mut m) = self.mixer.lock() {
            if paused {
                m.stop(MUSIC_VOICE);
            } else {
                m.play(MUSIC_VOICE, false);
            }
        }
    }
}

impl Drop for WmaStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(mut m) = self.mixer.lock() {
            m.set_buffer(MUSIC_VOICE, None);
        }
    }
}

fn push(mx: &SharedMixer, pcm: Vec<i16>, fmt: u32, first: &mut bool) {
    let bytes: Vec<u8> = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
    let d = Arc::new(SampleData { bytes, format: fmt });
    if let Ok(mut m) = mx.lock() {
        if *first || !m.is_playing(MUSIC_VOICE) {
            m.set_buffer(MUSIC_VOICE, Some(d));
            m.play(MUSIC_VOICE, false);
            *first = false;
        } else {
            m.queue_buffer(MUSIC_VOICE, d);
        }
    }
}

fn xva_rate_index(rate: u32) -> u32 {
    crate::audio::xva::rate_index(rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_wma_demuxes() {
        let dir = visos::file::resolve(r"\Audio\Music");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if !p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wma")) {
                continue;
            }
            let b = std::fs::read(&p).unwrap();
            let a = demux(&b).unwrap_or_else(|| panic!("{}", p.display()));
            assert_eq!(a.format_tag, 0x161, "{}", p.display());
            assert_eq!(a.packets.len() % a.block_align as usize, 0, "{}", p.display());
            n += 1;
        }
        assert!(n >= 60);
    }
}
