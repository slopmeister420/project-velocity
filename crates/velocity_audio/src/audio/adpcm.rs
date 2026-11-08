//! xbox adpcm (`WAVE_FORMAT_XBOX_ADPCM` 0x69) decoder, replacing the mcpx hardware decoder dsound feeds.
//! ima adpcm with 36-byte blocks per channel: `s16 predictor, u8 step index, u8 0` then 32 bytes of nibbles,
//! low nibble first. channels interleave in 4-byte words after the per-channel headers (72-byte stereo block).
//! one block = 64 frames: the header sample plus the first 63 nibbles (the 64th nibble is padding — the
//! encoder's next-block predictor follows nibble 63, verified on the shipped data).

pub const BLOCK_BYTES: usize = 36;
pub const FRAMES_PER_BLOCK: usize = 64;

pub const STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97,
    107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796,
    876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871,
    5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623,
    27086, 29794, 32767,
];

pub const INDEX_TABLE: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

#[derive(Clone, Copy, Debug, Default)]
struct ImaState {
    predictor: i32,
    index: i32,
}

impl ImaState {
    fn decode(&mut self, nibble: u8) -> i16 {
        let step = STEP_TABLE[self.index as usize];
        let mut diff = step >> 3;
        if nibble & 1 != 0 {
            diff += step >> 2;
        }
        if nibble & 2 != 0 {
            diff += step >> 1;
        }
        if nibble & 4 != 0 {
            diff += step;
        }
        if nibble & 8 != 0 {
            self.predictor -= diff;
        } else {
            self.predictor += diff;
        }
        self.predictor = self.predictor.clamp(-32768, 32767);
        self.index = (self.index + INDEX_TABLE[nibble as usize]).clamp(0, 88);
        self.predictor as i16
    }
}

/// number of frames `bytes` of adpcm data hold (`36 * channels` bytes per 64 frames).
pub fn frames_in(bytes: usize, channels: usize) -> usize {
    bytes / (BLOCK_BYTES * channels) * FRAMES_PER_BLOCK
}

/// decode whole blocks of `data` into interleaved s16 frames. a trailing partial block is ignored.
pub fn decode(data: &[u8], channels: usize) -> Vec<i16> {
    let block = BLOCK_BYTES * channels;
    let mut out = Vec::with_capacity(frames_in(data.len(), channels) * channels);
    for b in data.chunks_exact(block) {
        decode_block(b, channels, &mut out, FRAMES_PER_BLOCK);
    }
    out
}

/// decode one block (`36 * channels` bytes) appending `frames` (≤ 65) interleaved frames to `out`.
pub fn decode_block(b: &[u8], channels: usize, out: &mut Vec<i16>, frames: usize) {
    debug_assert!(channels > 0 && channels <= 8 && b.len() >= BLOCK_BYTES * channels);
    let mut st = [ImaState::default(); 8];
    let base = out.len();
    out.resize(base + frames * channels, 0);
    for ch in 0..channels {
        let h = &b[ch * 4..ch * 4 + 4];
        st[ch].predictor = i16::from_le_bytes([h[0], h[1]]) as i32;
        st[ch].index = (h[2] as i32).min(88);
        if frames > 0 {
            out[base + ch] = st[ch].predictor as i16;
        }
    }
    // 8 words of 4 bytes per channel, interleaved word by word
    let data = &b[channels * 4..];
    for word in 0..8 {
        for ch in 0..channels {
            let w = &data[(word * channels + ch) * 4..(word * channels + ch) * 4 + 4];
            for (k, &byte) in w.iter().enumerate() {
                for (half, nib) in [byte & 0x0F, byte >> 4].into_iter().enumerate() {
                    let n = word * 8 + k * 2 + half; // nibble index 0..64
                    let frame = n + 1;
                    let s = st[ch].decode(nib);
                    if frame < frames {
                        out[base + frame * channels + ch] = s;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_block_decodes_to_predictor() {
        // nibble 0 with index 0 adds step>>3 = 0 each time
        let mut b = [0u8; 36];
        b[0..2].copy_from_slice(&100i16.to_le_bytes());
        let out = decode(&b, 1);
        assert_eq!(out.len(), 64);
        assert!(out.iter().all(|&s| s == 100));
    }

    #[test]
    fn stereo_interleave() {
        let mut b = [0u8; 72];
        b[0..2].copy_from_slice(&10i16.to_le_bytes());
        b[4..6].copy_from_slice(&(-20i16).to_le_bytes());
        let out = decode(&b, 2);
        assert_eq!(out.len(), 128);
        assert_eq!((out[0], out[1], out[126], out[127]), (10, -20, 10, -20));
    }
}
