//! RGSS Decryption Key implementation
//!
//! The RGSS archive uses a simple LCG (Linear Congruential Generator)
//! for its encryption key. The key is stepped using the formula:
//!
//! ```text
//! state = state * multiplier + accumulator
//! ```
//!
//! For V1 (XP/VX): multiplier = 7, accumulator = 3
//! For V3 (VX Ace): multiplier = 9, accumulator = 3

use super::RgssVersion;

/// RGSS decryption/encryption key using LCG
#[derive(Debug, Clone, Copy)]
pub struct RgssKey {
    /// Current key state
    state: u32,
    /// Multiplier for LCG
    multiplier: u32,
    /// Accumulator for LCG
    accumulator: u32,
}

impl RgssKey {
    /// Create a new key for the specified version
    pub fn new(version: RgssVersion) -> Self {
        match version {
            RgssVersion::V1 => Self {
                state: 0,
                multiplier: 7,
                accumulator: 3,
            },
            RgssVersion::V3 => Self {
                state: 0,
                multiplier: 9,
                accumulator: 3,
            },
        }
    }

    /// Create a key with a specific initial state
    pub fn with_state(version: RgssVersion, state: u32) -> Self {
        let mut key = Self::new(version);
        key.state = state;
        key
    }

    /// Get the current key state
    #[inline]
    pub fn current(&self) -> u32 {
        self.state
    }

    /// Set the key state
    #[inline]
    pub fn set_state(&mut self, state: u32) {
        self.state = state;
    }

    /// Step the key to the next state
    #[inline]
    pub fn step(&mut self) {
        self.state = self.state.wrapping_mul(self.multiplier).wrapping_add(self.accumulator);
    }

    /// Decrypt a 32-bit integer and step the key
    #[inline]
    pub fn decrypt_int(&mut self, value: u32) -> u32 {
        let result = value ^ self.state;
        self.step();
        result
    }

    /// Decrypt a 32-bit integer without stepping the key
    #[inline]
    pub fn decrypt_int_no_step(&self, value: u32) -> u32 {
        value ^ self.state
    }

    /// Encrypt a 32-bit integer and step the key
    #[inline]
    pub fn encrypt_int(&mut self, value: u32) -> u32 {
        // XOR encryption is symmetric
        self.decrypt_int(value)
    }

    /// Decrypt a string (byte array) for V1 format
    /// Each byte is XORed with the lowest byte of the key, then the key is stepped
    pub fn decrypt_string_v1(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        for &byte in data {
            result.push(byte ^ (self.state & 0xFF) as u8);
            self.step();
        }
        result
    }

    /// Encrypt a string (byte array) for V1 format
    pub fn encrypt_string_v1(&mut self, data: &[u8]) -> Vec<u8> {
        // XOR encryption is symmetric
        self.decrypt_string_v1(data)
    }

    /// Decrypt a string (byte array) for V3 format
    /// Each byte is XORed with a different byte of the key (cyclic)
    pub fn decrypt_string_v3(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        for (i, &byte) in data.iter().enumerate() {
            let shift = (i % 4) * 8;
            result.push(byte ^ ((self.state >> shift) & 0xFF) as u8);
        }
        result
    }

    /// Encrypt a string (byte array) for V3 format
    pub fn encrypt_string_v3(&self, data: &[u8]) -> Vec<u8> {
        // XOR encryption is symmetric
        self.decrypt_string_v3(data)
    }

    /// Decrypt file content
    /// Each 4 bytes are XORed with the key, then the key is stepped
    pub fn decrypt_content(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut j = 0;

        for &byte in data {
            let shift = j * 8;
            result.push(byte ^ ((self.state >> shift) & 0xFF) as u8);

            j += 1;
            if j == 4 {
                j = 0;
                self.step();
            }
        }

        result
    }

    /// Encrypt file content
    pub fn encrypt_content(&mut self, data: &[u8]) -> Vec<u8> {
        // XOR encryption is symmetric
        self.decrypt_content(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_v1_step() {
        let mut key = RgssKey::with_state(RgssVersion::V1, 0xDEADCAFE);
        assert_eq!(key.current(), 0xDEADCAFE);

        key.step();
        // 0xDEADCAFE * 7 + 3 = 0x60F8A8EB (with wrapping)
        let expected = 0xDEADCAFEu32.wrapping_mul(7).wrapping_add(3);
        assert_eq!(key.current(), expected);
    }

    #[test]
    fn test_key_v3_step() {
        let mut key = RgssKey::with_state(RgssVersion::V3, 0x12345678);
        assert_eq!(key.current(), 0x12345678);

        key.step();
        // 0x12345678 * 9 + 3 = 0xA5D70ACB (with wrapping)
        let expected = 0x12345678u32.wrapping_mul(9).wrapping_add(3);
        assert_eq!(key.current(), expected);
    }

    #[test]
    fn test_decrypt_int() {
        let mut key = RgssKey::with_state(RgssVersion::V1, 0xDEADCAFE);
        let encrypted = 0x12345678 ^ 0xDEADCAFE;
        let decrypted = key.decrypt_int(encrypted);
        assert_eq!(decrypted, 0x12345678);
    }

    #[test]
    fn test_encrypt_decrypt_symmetry() {
        let data = b"Hello, World!";
        let mut key1 = RgssKey::with_state(RgssVersion::V1, 0xDEADCAFE);
        let mut key2 = RgssKey::with_state(RgssVersion::V1, 0xDEADCAFE);

        let encrypted = key1.encrypt_content(data);
        let decrypted = key2.decrypt_content(&encrypted);

        assert_eq!(decrypted, data);
    }
}
