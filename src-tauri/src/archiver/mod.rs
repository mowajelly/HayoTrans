//! Archiver module for game archive operations
//!
//! This module provides functionality for unpacking and repacking
//! game archive formats. Currently supports RGSS archives used by
//! RPG Maker XP, VX, and VX Ace.

pub mod rgss;

use std::io;
use std::path::Path;

/// Result type for archiver operations
pub type ArchiverResult<T> = Result<T, ArchiverError>;

/// Errors that can occur during archive operations
#[derive(Debug, thiserror::Error)]
pub enum ArchiverError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid archive format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Trait for archive readers (unpackers)
pub trait ArchiveReader {
    /// List of file entries in the archive
    type Entry;

    /// Open an archive from a path
    fn open<P: AsRef<Path>>(path: P) -> ArchiverResult<Self>
    where
        Self: Sized;

    /// Get the list of entries in the archive
    fn entries(&self) -> &[Self::Entry];

    /// Extract all files to a directory
    fn extract_all<P: AsRef<Path>>(&self, output_dir: P) -> ArchiverResult<usize>;

    /// Extract a single entry by name
    fn extract_entry<P: AsRef<Path>>(&self, entry_name: &str, output_dir: P) -> ArchiverResult<()>;
}

/// Trait for archive writers (repackers)
pub trait ArchiveWriter {
    /// Create a new archive writer
    fn new() -> Self;

    /// Add a file to the archive
    fn add_file<P: AsRef<Path>>(&mut self, path: P, archive_name: &str) -> ArchiverResult<()>;

    /// Add files from a directory recursively
    fn add_directory<P: AsRef<Path>>(&mut self, dir: P, base_path: Option<&str>)
        -> ArchiverResult<usize>;

    /// Write the archive to a file
    fn write<P: AsRef<Path>>(self, output_path: P) -> ArchiverResult<()>;
}

/// Archive format detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// RGSS archive (RPG Maker XP/VX/VX Ace)
    Rgss(rgss::RgssVersion),
}

impl ArchiveFormat {
    /// Detect the archive format from a file
    pub fn detect<P: AsRef<Path>>(path: P) -> Option<Self> {
        if let Ok(version) = rgss::RgssVersion::detect(&path) {
            return Some(ArchiveFormat::Rgss(version));
        }

        None
    }

    /// Detect the archive format from a file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rgssad" => Some(ArchiveFormat::Rgss(rgss::RgssVersion::V1)),
            "rgss2a" => Some(ArchiveFormat::Rgss(rgss::RgssVersion::V1)),
            "rgss3a" => Some(ArchiveFormat::Rgss(rgss::RgssVersion::V3)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert!(matches!(
            ArchiveFormat::from_extension("rgssad"),
            Some(ArchiveFormat::Rgss(rgss::RgssVersion::V1))
        ));
        assert!(matches!(
            ArchiveFormat::from_extension("rgss2a"),
            Some(ArchiveFormat::Rgss(rgss::RgssVersion::V1))
        ));
        assert!(matches!(
            ArchiveFormat::from_extension("rgss3a"),
            Some(ArchiveFormat::Rgss(rgss::RgssVersion::V3))
        ));
        assert!(ArchiveFormat::from_extension("zip").is_none());
    }
}
