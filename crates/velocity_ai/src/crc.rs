//! Name hashing used by the ai lookups (stand-in for visos `Handy/HANDY_CRC.cpp` until core publishes it).

/// reflected crc32 (poly 0xEDB88320), caller-supplied seed, no final xor.
/// HANDY_CRC.cpp 0x002FD200
pub fn handy_crc32(bytes: &[u8], mut crc: u32) -> u32 {
    for &b in bytes {
        let mut c = (crc ^ b as u32) & 0xFF;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
        crc = c ^ (crc >> 8);
    }
    crc
}

/// crc of a c string, 0 for an empty one.
/// VsStringAnsi.cpp 0x002B46B0
pub fn string_crc(s: &str) -> u32 {
    if s.is_empty() { 0 } else { handy_crc32(s.as_bytes(), 0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_values() {
        // VS_CHECKED_CRC("NONE") in the xbe, and an OPPONENT.BIN name crc
        assert_eq!(string_crc("NONE"), 0x6841_3267);
        assert_eq!(string_crc("TO_COP2"), 0x13DC_1CFD);
    }
}
