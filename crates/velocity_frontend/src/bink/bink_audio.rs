//! Bink audio: decoder open (0x00543280), block decode (0x005434B0 -> 0x00542E80, coefficients 0x00542BA0,
//! s16 output 0x005429F0 mono / 0x00542AC0 stereo) and BinkDoFrame's per-track block loop (0x00541340).
//!
//! The transform is Takuya Ooura's `ddct(n, 1, ..)` (fftsg.c, compiled into the BINK section as
//! 0x0054E0B0 with makewt 0x005495B0 / makect 0x00549700 / dctsub 0x0054D070 / cftfsub 0x0054DD10 / rftfsub
//! 0x0054CEF0): `C[k] = sum_j a[j] cos(pi j (k + 1/2) / n)`. It is computed here with an f64 FFT (Makhoul's
//! DCT-III), i.e. the same values up to float rounding, not Ooura's butterfly order. The RDFT variant
//! (0x0054DF70, tracks without flag 0x10000000) is not ported: no shipped movie uses it.

use super::bink_file::BinkTrack;
use super::bitstream::BitReader;
use super::tables::BinkTables;

pub struct BinkAudio {
    /// +0 transform length
    pub frame_len: usize,
    /// +4 output scale `2 / sqrt(frame_len)`
    scale: f32,
    /// +8 output bytes per block (`2 * channels * frame_len`)
    block_bytes: usize,
    /// +12 crossfade bytes (`block_bytes / 16`)
    overlap_bytes: usize,
    /// +16
    pub channels: usize,
    /// +20 s16 block output
    out: Vec<i16>,
    /// +24 first block (no crossfade)
    first: bool,
    /// +32 band edges (`num_bands + 1`)
    bands: Vec<u32>,
    /// +44 the previous block's tail
    prev: Vec<i16>,
    /// +48 (bit 0 = DCT)
    dct: bool,
    coeffs: Vec<f32>,
    pub rate: u32,
}

impl BinkAudio {
    /// 0x00543280 (`rate`, `channels`, DCT flag), as BinkOpen calls it per track
    pub fn new(t: &BinkTables, track: &BinkTrack) -> Self {
        let channels = track.channels() as usize;
        let dct = track.dct();
        let rate = track.rate;
        let mut frame_len: u32 = if rate < 0xAC44 { if rate < 0x5622 { 512 } else { 1024 } } else { 2048 };
        let block_bytes = 2 * channels as u32 * frame_len;
        let (mut r, mut ch) = (rate, channels as u32);
        if !dct {
            r *= ch;
            frame_len *= ch;
            ch = 1;
        }
        let half = frame_len >> 1;
        let nyq = (r + 1) >> 1;
        let n = t.audio_bands.iter().position(|&f| f >= nyq).unwrap_or(25);
        let mut bands: Vec<u32> = (0..n).map(|i| (half * t.audio_bands[i] / nyq).max(1)).collect();
        bands.push(half);
        Self {
            frame_len: frame_len as usize,
            scale: (2.0 / (frame_len as f64).sqrt()) as f32,
            block_bytes: block_bytes as usize,
            overlap_bytes: (block_bytes >> 4) as usize,
            channels: ch as usize,
            out: vec![0; block_bytes as usize / 2],
            first: true,
            bands,
            prev: vec![0; (block_bytes >> 4) as usize / 2],
            dct,
            coeffs: vec![0.0; frame_len as usize * ch as usize],
            rate,
        }
    }

    /// 0x00542BA0 coefficients 2.. of one channel
    fn read_coeffs(t: &BinkTables, br: &mut BitReader, c: &mut [f32], quant: &[f32], bands: &[u32]) {
        let n = c.len();
        let (mut q, mut i) = (0.0f32, 0usize);
        while 2 * bands[i] < 2 {
            q = quant[i];
            i += 1;
        }
        let mut j = 2usize;
        while j < n {
            let mut end = if br.bit() { j + 8 * t.audio_rle[br.read(4) as usize] as usize } else { j + 8 };
            end = end.min(n);
            let width = br.read(4);
            if width != 0 {
                while j < end {
                    if i < bands.len() && j == 2 * bands[i] as usize {
                        q = quant[i];
                        i += 1;
                    }
                    let v = br.read(width);
                    c[j] = if v != 0 {
                        let s = if br.bit() { -q } else { q };
                        (v as f64 * s as f64) as f32
                    } else {
                        0.0
                    };
                    j += 1;
                }
            } else {
                for x in &mut c[j..end] {
                    *x = 0.0;
                }
                j = end;
                while i < bands.len() && end > 2 * bands[i] as usize {
                    q = quant[i];
                    i += 1;
                }
            }
        }
    }

    /// 0x00542E80 one block of all channels into `self.out`; returns the bytes consumed
    fn decode_raw(&mut self, t: &BinkTables, data: &[u8], at: usize) -> usize {
        let mut br = BitReader::new(data, at);
        if self.dct {
            br.skip(2);
        }
        let n = self.frame_len;
        let nb = self.bands.len() - 1;
        let mut quant = [0f32; 25];
        for ch in 0..self.channels {
            let c = &mut self.coeffs[ch * n..ch * n + n];
            for k in 0..2 {
                let v = br.read(29);
                let mut f = ((v >> 5) & 0x77F_FFFF) as f64 * t.audio_exp[(v & 0x1F) as usize] as f64;
                if v & 0x1000_0000 != 0 {
                    f = -f;
                }
                c[k] = f as f32;
            }
            for q in quant.iter_mut().take(nb) {
                *q = t.audio_quant[br.read(8).min(95) as usize];
            }
            Self::read_coeffs(t, &mut br, c, &quant[..nb], &self.bands);
            if self.dct {
                ddct_inverse(c);
            } else {
                log::warn!("bink: RDFT audio not ported");
            }
        }
        let conv = |x: f32| -> i16 {
            let v = (x * self.scale) as i32;
            v.clamp(-32768, 0x7FFF) as i16
        };
        if self.channels == 1 {
            for i in 0..n {
                self.out[i] = conv(self.coeffs[i]);
            }
        } else {
            for i in 0..n {
                self.out[2 * i] = conv(self.coeffs[i]);
                self.out[2 * i + 1] = conv(self.coeffs[n + i]);
            }
        }
        (br.word_end() - at) & 0x1FFF_FFFF
    }

    /// 0x005434B0: decode, crossfade the head with the last block's tail (unsigned division, as the original),
    /// keep the new tail; returns (samples to play, bytes consumed)
    pub fn decode_block(&mut self, t: &BinkTables, data: &[u8], at: usize) -> (&[i16], usize) {
        let used = self.decode_raw(t, data, at);
        if self.first {
            self.first = false;
        } else {
            let n = (self.overlap_bytes >> 1) as u32;
            for i in 0..n {
                let s = (i as i32).wrapping_mul(self.out[i as usize] as i32)
                    .wrapping_add(((n - i) as i32).wrapping_mul(self.prev[i as usize] as i32));
                self.out[i as usize] = ((s as u32) / n) as u16 as i16;
            }
        }
        let keep = (self.block_bytes - self.overlap_bytes) / 2;
        self.prev.copy_from_slice(&self.out[keep..keep + self.overlap_bytes / 2]);
        (&self.out[..keep], used)
    }

    /// BinkDoFrame's track loop: `u32 bytes` then blocks until that many bytes are out; appends to `pcm`
    pub fn decode_packet(&mut self, t: &BinkTables, packet: &[u8], pcm: &mut Vec<i16>) {
        if packet.len() < 4 {
            return;
        }
        let mut left = u32::from_le_bytes(packet[0..4].try_into().unwrap()) as usize;
        let mut at = 4usize;
        while left != 0 && at < packet.len() {
            let (s, used) = self.decode_block(t, packet, at);
            let n = (s.len() * 2).min(left);
            pcm.extend_from_slice(&s[..n / 2]);
            left -= n;
            at += used;
        }
    }
}

/// Ooura `ddct(n, 1, a)`: `C[k] = sum_j a[j] cos(pi j (k + 1/2) / n)`, via an n-point complex FFT
fn ddct_inverse(a: &mut [f32]) {
    let n = a.len();
    let mut re = vec![0f64; n];
    let mut im = vec![0f64; n];
    for j in 0..n {
        let th = std::f64::consts::PI * j as f64 / (2.0 * n as f64);
        re[j] = a[j] as f64 * th.cos();
        im[j] = a[j] as f64 * th.sin();
    }
    fft(&mut re, &mut im, 1.0);
    for m in 0..n / 2 {
        a[2 * m] = re[m] as f32;
        a[2 * m + 1] = re[n - 1 - m] as f32;
    }
}

/// in-place radix-2 FFT, `sign` = +1 for `e^{+2 pi i jm / n}`
fn fft(re: &mut [f64], im: &mut [f64], sign: f64) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = sign * 2.0 * std::f64::consts::PI / len as f64;
        for s in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (wr, wi) = ((ang * k as f64).cos(), (ang * k as f64).sin());
                let (a, b) = (s + k, s + k + len / 2);
                let (xr, xi) = (re[b] * wr - im[b] * wi, re[b] * wi + im[b] * wr);
                re[b] = re[a] - xr;
                im[b] = im[a] - xi;
                re[a] += xr;
                im[a] += xi;
            }
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddct_matches_definition() {
        let n = 16;
        let src: Vec<f32> = (0..n).map(|i| ((i * 7 % 5) as f32) - 2.0).collect();
        let mut a = src.clone();
        ddct_inverse(&mut a);
        for k in 0..n {
            let r: f64 = (0..n)
                .map(|j| src[j] as f64 * (std::f64::consts::PI * j as f64 * (k as f64 + 0.5) / n as f64).cos())
                .sum();
            assert!((r - a[k] as f64).abs() < 1e-4, "{k}: {r} vs {}", a[k]);
        }
    }
}
