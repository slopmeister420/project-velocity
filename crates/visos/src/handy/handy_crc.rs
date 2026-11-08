//! Handy\HANDY_CRC.cpp — reflected crc32 (poly 0xEDB88320), caller-supplied seed, no final xor.
//! file names are hashed with `crc32_no_case(name, 0)`.

const POLY: u32 = 0xEDB8_8320;

/// HANDY_CRC.cpp 0x002FD300
pub fn crc32_byte(mut a: u32) -> u32 {
    for _ in 0..8 {
        if a & 1 != 0 {
            a = (a >> 1) ^ POLY;
        } else {
            a >>= 1;
        }
    }
    a
}

/// HANDY_CRC.cpp 0x002FD200
pub fn crc32(bytes: &[u8], mut crc: u32) -> u32 {
    debug_assert!(!bytes.is_empty(), "dLen != 0");
    for &b in bytes {
        crc = crc32_byte((crc ^ b as u32) & 0xFF) ^ (crc >> 8);
    }
    crc
}

/// HANDY_CRC.cpp 0x002FD360
pub fn crc32_wide(chars: &[u16], mut crc: u32) -> u32 {
    debug_assert!(!chars.is_empty(), "dLen != 0");
    for &c in chars {
        // only the low byte of (crc ^ c) is used
        crc = crc32_byte((crc ^ c as u32) & 0xFF) ^ (crc >> 8);
    }
    crc
}

/// HANDY_CRC.cpp 0x002FD460
pub fn crc32_no_case(bytes: &[u8], mut crc: u32) -> u32 {
    debug_assert!(!bytes.is_empty(), "dLen != 0");
    for &b in bytes {
        crc = crc32_byte((crc ^ tolower(b as u32)) & 0xFF) ^ (crc >> 8);
    }
    crc
}

/// HANDY_CRC.cpp 0x002FD560
pub fn crc32_wide_no_case(chars: &[u16], mut crc: u32) -> u32 {
    debug_assert!(!chars.is_empty(), "dLen != 0");
    for &c in chars {
        crc = crc32_byte((crc ^ tolower(c as u32)) & 0xFF) ^ (crc >> 8);
    }
    crc
}

/// crt `tolower` in the "C" locale (0x003E6140): only A-Z fold.
fn tolower(c: u32) -> u32 {
    if (b'A' as u32..=b'Z' as u32).contains(&c) { c + 32 } else { c }
}

/// hash of a name as the file system keys it (`VsHashedString`, 0x002D7F30): empty → 0.
pub fn name_hash(name: &str) -> u32 {
    if name.is_empty() { 0 } else { crc32_no_case(name.as_bytes(), 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vcd_names() {
        // hashes stored in asp/asp.vcd
        assert_eq!(name_hash("asp.gas"), 0xE13A_A739);
        assert_eq!(name_hash("event_1.gaf"), 0xA5BD_FDC8);
        assert_eq!(name_hash("ASP.GAS"), 0xE13A_A739);
    }
}
