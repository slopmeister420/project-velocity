//! positive controls over every bank and group in the install.

use std::path::{Path, PathBuf};

use super::adpcm;
use super::xva::{decode_sample, XvaBank};
use super::xvg::{event_hash, XvgGroup};

fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, ext, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case(ext)) {
            out.push(p);
        }
    }
}

fn files(ext: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    walk(visos::file::game_dir(), ext, &mut v);
    v.sort();
    v
}

fn write_wav(path: &Path, pcm: &[i16], ch: u16, rate: u32) {
    let mut b = Vec::with_capacity(44 + pcm.len() * 2);
    let data = (pcm.len() * 2) as u32;
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&ch.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * ch as u32 * 2).to_le_bytes());
    b.extend_from_slice(&(ch * 2).to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for v in pcm {
        b.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(path, b).unwrap();
}

#[test]
fn every_xva_decodes() {
    let banks = files("xva");
    assert!(banks.len() > 100, "found {} banks", banks.len());
    let (mut n_samples, mut total_secs) = (0usize, 0f64);
    let (mut silent, mut stats) = (Vec::new(), Vec::new());
    for p in &banks {
        let bytes = std::fs::read(p).unwrap();
        let bank = XvaBank::from_bytes(&bytes, 1).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        for (i, h) in bank.samples.iter().enumerate() {
            let raw = bank.sample_bytes(i, Some(&bytes)).unwrap_or_else(|| panic!("{} #{i}: out of range", p.display()));
            assert!(h.channels() == 1 || h.channels() == 2, "{} #{i}: channels {}", p.display(), h.channels());
            if h.is_adpcm() {
                assert_eq!(super::xva::bits_from_format(h.format), 4);
                assert_eq!(raw.len() % (36 * h.channels() as usize), 0, "{} #{i}: partial block", p.display());
                // header step index must be in range in every block
                for blk in raw.chunks_exact(36 * h.channels() as usize) {
                    for ch in 0..h.channels() as usize {
                        assert!(blk[ch * 4 + 2] <= 88 && blk[ch * 4 + 3] == 0, "{} #{i}: bad block header", p.display());
                    }
                }
            } else {
                assert_eq!(super::xva::bits_from_format(h.format), 16);
            }
            let pcm = decode_sample(h, raw);
            assert_eq!(pcm.len(), h.frames() * h.channels() as usize);
            let rms = (pcm.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>() / pcm.len().max(1) as f64).sqrt();
            let peak = pcm.iter().map(|&v| (v as i32).abs()).max().unwrap_or(0);
            if rms < 1.0 {
                silent.push(format!("{} #{i}", p.display()));
            }
            stats.push((rms, peak));
            if let Some(dir) = std::env::var_os("VELOCITY_AUDIO_WAV_DIR") {
                if i == 0 && n_samples % 20 == 0 {
                    let name = format!("{}_{i}.wav", p.file_stem().unwrap().to_string_lossy());
                    write_wav(&Path::new(&dir).join(name), &pcm, h.channels() as u16, h.rate());
                }
            }
            n_samples += 1;
            total_secs += h.duration();
        }
    }
    eprintln!("{} banks, {n_samples} samples, {:.1} s of audio", banks.len(), total_secs);
    let mean_rms = stats.iter().map(|s| s.0).sum::<f64>() / stats.len() as f64;
    let clipped = stats.iter().filter(|s| s.1 >= 32767).count();
    eprintln!("mean rms {mean_rms:.0}, {clipped} samples touch full scale, silent: {silent:?}");
    // a broken decoder drifts into rails or noise; shipped audio sits well inside
    assert!(mean_rms > 500.0 && mean_rms < 12000.0);
    assert!(silent.len() < 5);
}

#[test]
fn every_xvg_parses_and_resolves() {
    let groups = files("xvg");
    assert!(groups.len() > 100);
    let (mut n_events, mut n_refs, mut dangling) = (0, 0, Vec::new());
    for p in &groups {
        let bytes = std::fs::read(p).unwrap();
        let (g, end) = XvgGroup::from_bytes(&bytes).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        assert_eq!(end, bytes.len(), "{}: trailing bytes", p.display());
        for e in &g.events {
            // ids are crc32(uppercase name), except tool-numbered groups (EnvFX: 100, 105, ...)
            assert!(e.hash == event_hash(&e.name) || e.hash < 0x10000, "{}: {}", p.display(), e.name);
            for s in &e.sounds {
                for c in &s.chains {
                    assert!((c.event as usize) < g.events.len(), "{}: chain out of range", p.display());
                }
            }
        }
        n_events += g.events.len();
        // each referenced bank must exist and hold the referenced sample
        let dir = p.parent().unwrap();
        let banks: Vec<Option<XvaBank>> = g
            .banks
            .iter()
            .map(|b| {
                let rel = if b.dir.is_empty() { b.name.clone() } else { format!("{}/{}", b.dir, b.name) };
                let path = visos::file::platform::resolve_in(dir, &format!("{rel}.XVA"));
                std::fs::read(&path).ok().map(|by| {
                    XvaBank::parse_header(&by).map(|(f, s)| XvaBank { flags: f, samples: s, ..Default::default() }).unwrap()
                })
            })
            .collect();
        for e in &g.events {
            for s in &e.sounds {
                for smp in &s.samples {
                    let bank = banks.get(smp.bank as usize).unwrap_or_else(|| panic!("{}: bank idx", p.display()));
                    if let Some(bank) = bank {
                        if (smp.sample as usize) < bank.samples.len() {
                            n_refs += 1;
                        } else {
                            // shipped data error; the original reads past the bank's header table
                            eprintln!("{}: {} -> sample {} of {}", p.display(), smp.name, smp.sample, bank.samples.len());
                            dangling.push(p.file_name().unwrap().to_string_lossy().into_owned());
                        }
                    } else {
                        eprintln!("{}: missing bank {:?}", p.display(), g.banks[smp.bank as usize].name);
                    }
                }
            }
        }
    }
    eprintln!("{} groups, {n_events} events, {n_refs} resolved sample refs", groups.len());
    dangling.dedup();
    assert!(dangling.iter().all(|f| f.eq_ignore_ascii_case("vvipesu0.xvg")), "{dangling:?}");
}

/// the CarDamageFX glass impact event (0x00078320) is in no shipped group: only its IMPACT_CAR_PHYSICS
/// follow-up sounds (as in the original, whose first PlayEvent just fails).
#[test]
fn glass_impact_event_is_dead_data() {
    let (mut glass, mut car) = (0, 0);
    for p in files("xvg") {
        let (g, _) = XvgGroup::from_bytes(&std::fs::read(&p).unwrap()).unwrap();
        glass += g.find_event(crate::vs_audio::GLASS_IMPACT).is_some() as usize;
        car += g.find_event(crate::vs_audio_vehics::IMPACT_CAR_PHYSICS).is_some() as usize;
    }
    assert_eq!(glass, 0);
    assert!(car > 0);
}

/// decides how many frames a 36-byte block holds, on a smooth engine loop: frame 63 must be one
/// step before the next block's header (frame 62 two steps) and nibble 64 must add nothing.
#[test]
fn adpcm_block_boundary() {
    let path = visos::file::resolve("Audio/vehicle/tier4/wrx/vimprrb0.XVA");
    let bytes = std::fs::read(path).unwrap();
    let bank = XvaBank::from_bytes(&bytes, 1).unwrap();
    let raw = bank.sample_bytes(0, Some(&bytes)).unwrap();
    let (mut e, mut step, mut n) = ([0f64; 3], 0f64, 0usize);
    let blocks: Vec<&[u8]> = raw.chunks_exact(36).collect();
    for w in blocks.windows(2) {
        let mut out = Vec::new();
        adpcm::decode_block(w[0], 1, &mut out, 65);
        let next = i16::from_le_bytes([w[1][0], w[1][1]]) as f64;
        for k in 0..3 {
            e[k] += (out[62 + k] as f64 - next).abs();
        }
        step += (1..64).map(|k| (out[k] as f64 - out[k - 1] as f64).abs()).sum::<f64>() / 63.0;
        n += 1;
    }
    let (e62, e63, e64, step) = (e[0] / n as f64, e[1] / n as f64, e[2] / n as f64, step / n as f64);
    eprintln!("blocks {n}: step {step:.0}, |f62-next| {e62:.0}, |f63-next| {e63:.0}, |f64-next| {e64:.0}");
    assert!((e63 - step).abs() < 0.2 * step && e62 > 1.7 * step && (e64 - e63).abs() < 0.1 * e63);
}

/// velocity.vol (the only shipped one): 4096 bytes, every preset float finite and within 0..=1.
#[test]
fn every_volume_preset_file_loads() {
    let vols = files("vol");
    assert!(!vols.is_empty());
    for p in &vols {
        assert_eq!(std::fs::metadata(p).unwrap().len(), 4096, "{}", p.display());
        let (mut a, _m) = super::system::AudioSystem::with_default_mixer();
        let rel = format!("\\{}", p.strip_prefix(visos::file::game_dir()).unwrap().display());
        assert_eq!(a.load_volume_presets(&rel), 0, "{}", p.display());
        assert!(a.presets.iter().flatten().all(|v| v.is_finite() && (0.0..=1.0).contains(v)), "{}", p.display());
    }
}
