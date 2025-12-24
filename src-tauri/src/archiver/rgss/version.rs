//! RGSS Version detection and constants

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::MAGIC;
use crate::archiver::ArchiverError;

/// RGSS archive version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgssVersion {
    /// Version 1 - RPG Maker XP (.rgssad) and VX (.rgss2a)
    V1,
    /// Version 3 - RPG Maker VX Ace (.rgss3a)
    V3,
}

impl RgssVersion {
    /// Detect the RGSS version from a file
    pub fn detect<P: AsRef<Path>>(path: P) -> Result<Self, ArchiverError> {
        let mut file = File::open(&path)?;
        Self::detect_from_file(&mut file)
    }

    /// Detect the RGSS version from an open file handle
    pub fn detect_from_file(file: &mut File) -> Result<Self, ArchiverError> {
        let mut header = [0u8; 8];

        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut header)?;

        // Check magic bytes
        if &header[0..7] != MAGIC {
            return Err(ArchiverError::InvalidFormat(
                "Not a valid RGSS archive: invalid magic bytes".to_string(),
            ));
        }

        // Get version from byte 7
        match header[7] {
            1 => Ok(RgssVersion::V1),
            3 => Ok(RgssVersion::V3),
            v => Err(ArchiverError::UnsupportedVersion(v)),
        }
    }

    /// Get the file extension for this version
    pub fn extension(&self) -> &'static str {
        match self {
            RgssVersion::V1 => "rgssad",
            RgssVersion::V3 => "rgss3a",
        }
    }

    /// Get the RPG Maker versions that use this archive format
    pub fn rpg_maker_versions(&self) -> &'static [&'static str] {
        match self {
            RgssVersion::V1 => &["RPG Maker XP", "RPG Maker VX"],
            RgssVersion::V3 => &["RPG Maker VX Ace"],
        }
    }

    /// Get the version byte for the archive header
    pub fn header_byte(&self) -> u8 {
        match self {
            RgssVersion::V1 => 1,
            RgssVersion::V3 => 3,
        }
    }
}

impl std::fmt::Display for RgssVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RgssVersion::V1 => write!(f, "RGSS V1 (XP/VX)"),
            RgssVersion::V3 => write!(f, "RGSS V3 (VX Ace)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_version_extension() {
        assert_eq!(RgssVersion::V1.extension(), "rgssad");
        assert_eq!(RgssVersion::V3.extension(), "rgss3a");
    }

    #[test]
    fn test_version_header_byte() {
        assert_eq!(RgssVersion::V1.header_byte(), 1);
        assert_eq!(RgssVersion::V3.header_byte(), 3);
    }

    #[test]
    fn test_detect_v1() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"RGSSAD\0\x01").unwrap();
        file.flush().unwrap();

        let detected = RgssVersion::detect(file.path()).unwrap();
        assert_eq!(detected, RgssVersion::V1);
    }

    #[test]
    fn test_detect_v3() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"RGSSAD\0\x03").unwrap();
        file.flush().unwrap();

        let detected = RgssVersion::detect(file.path()).unwrap();
        assert_eq!(detected, RgssVersion::V3);
    }

    #[test]
    fn test_detect_invalid() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"INVALID\0").unwrap();
        file.flush().unwrap();

        let result = RgssVersion::detect(file.path());
        assert!(matches!(result, Err(ArchiverError::InvalidFormat(_))));
    }

    #[test]
    fn test_detect_unsupported_version() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"RGSSAD\0\x02").unwrap();
        file.flush().unwrap();

        let result = RgssVersion::detect(file.path());
        assert!(matches!(result, Err(ArchiverError::UnsupportedVersion(2))));
    }
}
