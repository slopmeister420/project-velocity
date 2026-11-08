//! RAD Bink 1 (`BIKi`) decoder, ported from the debug XBE's `BINK` section (user decision 2026-11-04: the one
//! XDK-library exception). The library has no source names; functions keep RAD's public names where VsMovie
//! calls them (BinkOpen 0x0053E9A0, BinkDoFrame 0x00541340, ...) and are otherwise named by what they do.
//! Static tables (huffman lookups, scan orders, quantisers, audio bands) are read from the install's
//! `default.xbe` at runtime, like `velocity_physics::c_bump_map`.

pub mod bink_audio;
pub mod bink_file;
pub mod bink_video;
pub mod bitstream;
pub mod tables;

pub use bink_file::{BinkFile, BinkError};
pub use bink_video::BinkVideo;
pub use tables::BinkTables;

/// every `.bik` under `dir` (recursive)
pub fn find_movies(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            find_movies(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("bik")) {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(pcm: &[i16], rate: u32, ch: u16) {
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fe_captures/bink_in003.wav");
        let mut b = Vec::with_capacity(44 + pcm.len() * 2);
        let n = (pcm.len() * 2) as u32;
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&(36 + n).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&ch.to_le_bytes());
        b.extend_from_slice(&rate.to_le_bytes());
        b.extend_from_slice(&(rate * 2 * ch as u32).to_le_bytes());
        b.extend_from_slice(&(2 * ch).to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&n.to_le_bytes());
        for s in pcm {
            b.extend_from_slice(&s.to_le_bytes());
        }
        let _ = std::fs::write(out, b);
    }

    /// positive control: every frame of every movie decodes and each plane ends inside its data
    #[test]
    fn decode_every_bik() {
        let dir = visos::file::game_dir();
        if !dir.exists() {
            return;
        }
        let t = BinkTables::load().expect("bink tables from default.xbe");
        let mut files = Vec::new();
        find_movies(&dir.join("Movies"), &mut files);
        assert!(files.len() >= 125, "{} movies", files.len());
        let (mut frames, mut bad) = (0usize, Vec::new());
        let mut rgba = Vec::new();
        for p in &files {
            let f = BinkFile::parse(std::fs::read(p).unwrap(), 0).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert_eq!(f.index.len(), f.frames as usize + 1);
            let mut v = BinkVideo::new(&f);
            let mut audio: Vec<_> = f.tracks.iter().map(|tr| bink_audio::BinkAudio::new(&t, tr)).collect();
            let mut pcm = Vec::new();
            let mut varied = false;
            for i in 0..f.frames as usize {
                let pk = f.packet(i).unwrap();
                for (a, d) in audio.iter_mut().zip(&pk.audio) {
                    a.decode_packet(&t, d, &mut pcm);
                }
                v.decode(&t, pk.video);
                let (lend, ldecl, cend, len) = v.last_check;
                if lend > ldecl || ldecl - lend >= 4 || cend > len + 3 || len - cend.min(len) >= 4 {
                    if bad.len() < 20 {
                        bad.push(format!("{} frame {i}: luma {lend}/{ldecl} chroma {cend}/{len}", p.display()));
                    }
                }
                let y = &v.frame().y.data;
                varied |= y.iter().any(|&b| b != y[0]);
                frames += 1;
                if i == 250 && p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("In003.bik")) {
                    v.to_rgba(&mut rgba, 255);
                }
            }
            assert!(varied, "{}: every frame flat", p.display());
            if let Some(tr) = f.tracks.first() {
                // audio covers the movie's length (within a block) and is not silent
                let secs = f.frames as f64 * f.fps_den as f64 / f.fps_num as f64;
                let want = secs * tr.rate as f64 * tr.channels() as f64;
                let ratio = pcm.len() as f64 / want;
                if !(0.8..1.25).contains(&ratio) || pcm.iter().all(|&s| s == 0) {
                    if bad.len() < 20 {
                        bad.push(format!("{} audio: {} samples, {:.0} expected", p.display(), pcm.len(), want));
                    }
                }
                if p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("In003.bik")) {
                    write_wav(&pcm, tr.rate, tr.channels() as u16);
                }
            }
        }
        if !rgba.is_empty() {
            let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fe_captures/bink_in003_250.png");
            let _ = std::fs::create_dir_all(out.parent().unwrap());
            let file = std::fs::File::create(&out).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 512, 512);
            enc.set_color(png::ColorType::Rgba);
            enc.write_header().unwrap().write_image_data(&rgba).unwrap();
        }
        eprintln!("bink: {} files, {} frames, {} inconsistent", files.len(), frames, bad.len());
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}
