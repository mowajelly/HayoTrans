//! RGSS Archive module for RPG Maker XP, VX, and VX Ace
//!
//! This module provides functionality to read and write RGSS archives:
//! - `.rgssad` - RPG Maker XP
//! - `.rgss2a` - RPG Maker VX
//! - `.rgss3a` - RPG Maker VX Ace
//!
//! ## Archive Format
//!
//! All RGSS archives share a common header:
//! - Bytes 0-5: Magic "RGSSAD"
//! - Byte 6: Null byte (0x00)
//! - Byte 7: Version (1 for XP/VX, 3 for VX Ace)
//!
//! The encryption uses a simple LCG (Linear Congruential Generator):
//! - V1: state = state * 7 + 3, initial state = 0xDEADCAFE
//! - V3: state = state * 9 + 3, initial state from archive

mod key;
mod reader;
mod version;
mod writer;

pub use key::RgssKey;
pub use reader::RgssReader;
pub use version::RgssVersion;
pub use writer::RgssWriter;

use std::path::PathBuf;

/// Magic bytes for RGSS archives
pub const MAGIC: &[u8; 7] = b"RGSSAD\0";

/// Initial key for V1 archives (XP/VX)
pub const V1_INITIAL_KEY: u32 = 0xDEADCAFE;

/// File entry in an RGSS archive
#[derive(Debug, Clone)]
pub struct RgssEntry {
    /// File name (relative path within archive)
    pub name: String,
    /// File size in bytes
    pub size: u32,
    /// Offset in the archive file
    pub offset: u64,
    /// Decryption key for this file
    pub key: u32,
}

impl RgssEntry {
    /// Get the output path for extraction
    pub fn output_path(&self, base_dir: &std::path::Path) -> PathBuf {
        // Convert backslashes to forward slashes and normalize the path
        let normalized_name = self.name.replace('\\', "/");
        base_dir.join(normalized_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_entry_output_path() {
        let entry = RgssEntry {
            name: "Data\\Map001.rxdata".to_string(),
            size: 1024,
            offset: 0,
            key: 0,
        };

        let output = entry.output_path(Path::new("/output"));
        assert!(output.to_string_lossy().contains("Data"));
        assert!(output.to_string_lossy().contains("Map001.rxdata"));
    }
}
