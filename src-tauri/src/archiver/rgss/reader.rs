//! RGSS Archive Reader (Unpacker)

use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::{RgssEntry, RgssKey, RgssVersion, V1_INITIAL_KEY};
use crate::archiver::{ArchiveReader, ArchiverError, ArchiverResult};

/// RGSS Archive Reader for unpacking .rgssad, .rgss2a, and .rgss3a files
pub struct RgssReader {
    /// Path to the archive file
    path: std::path::PathBuf,
    /// Archive version
    version: RgssVersion,
    /// File entries in the archive
    entries: Vec<RgssEntry>,
}

impl RgssReader {
    /// Read a V1 format archive (RPG Maker XP/VX)
    fn read_v1(path: &Path) -> ArchiverResult<Vec<RgssEntry>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        // Skip header
        reader.seek(SeekFrom::Start(8))?;

        // Initialize key
        let mut key = RgssKey::with_state(RgssVersion::V1, V1_INITIAL_KEY);

        loop {
            // Check if we've reached the end of the file
            let current_pos = reader.stream_position()?;
            let end_pos = reader.seek(SeekFrom::End(0))?;

            if current_pos >= end_pos {
                break;
            }

            reader.seek(SeekFrom::Start(current_pos))?;

            // Read encrypted name length
            let mut len_bytes = [0u8; 4];
            if reader.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let name_len = key.decrypt_int(u32::from_le_bytes(len_bytes)) as usize;

            // Sanity check for name length
            if name_len == 0 || name_len > 1024 {
                break;
            }

            // Read encrypted name
            let mut name_bytes = vec![0u8; name_len];
            if reader.read_exact(&mut name_bytes).is_err() {
                break;
            }
            let decrypted_name = key.decrypt_string_v1(&name_bytes);
            let name = String::from_utf8_lossy(&decrypted_name).to_string();

            // Read encrypted size
            let mut size_bytes = [0u8; 4];
            if reader.read_exact(&mut size_bytes).is_err() {
                break;
            }
            let size = key.decrypt_int(u32::from_le_bytes(size_bytes));

            // Current position is the file data offset
            let offset = reader.stream_position()?;

            // Save the current key state for this file
            let file_key = key.current();

            // Skip file data
            reader.seek(SeekFrom::Current(size as i64))?;

            entries.push(RgssEntry {
                name,
                size,
                offset,
                key: file_key,
            });
        }

        Ok(entries)
    }

    /// Read a V3 format archive (RPG Maker VX Ace)
    fn read_v3(path: &Path) -> ArchiverResult<Vec<RgssEntry>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        // Skip header
        reader.seek(SeekFrom::Start(8))?;

        // Read initial key
        let mut initial_key_bytes = [0u8; 4];
        reader.read_exact(&mut initial_key_bytes)?;
        let initial_key = u32::from_le_bytes(initial_key_bytes);

        // Initialize and step key
        let mut key = RgssKey::with_state(RgssVersion::V3, initial_key);
        key.step();

        loop {
            // Read encrypted offset
            let mut offset_bytes = [0u8; 4];
            if reader.read_exact(&mut offset_bytes).is_err() {
                break;
            }
            let offset = key.decrypt_int(u32::from_le_bytes(offset_bytes));

            // Offset of 0 signals end of entries
            if offset == 0 {
                break;
            }

            // Read encrypted size
            let mut size_bytes = [0u8; 4];
            reader.read_exact(&mut size_bytes)?;
            let size = key.decrypt_int(u32::from_le_bytes(size_bytes));

            // Read encrypted file key
            let mut file_key_bytes = [0u8; 4];
            reader.read_exact(&mut file_key_bytes)?;
            let file_key = key.decrypt_int(u32::from_le_bytes(file_key_bytes));

            // Read encrypted name length
            let mut name_len_bytes = [0u8; 4];
            reader.read_exact(&mut name_len_bytes)?;
            let name_len = key.decrypt_int(u32::from_le_bytes(name_len_bytes)) as usize;

            // Read encrypted name
            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let decrypted_name = key.decrypt_string_v3(&name_bytes);
            let name = String::from_utf8_lossy(&decrypted_name).to_string();

            entries.push(RgssEntry {
                name,
                size,
                offset: offset as u64,
                key: file_key,
            });
        }

        Ok(entries)
    }

    /// Extract a single entry to a byte vector
    pub fn extract_to_memory(&self, entry: &RgssEntry) -> ArchiverResult<Vec<u8>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        reader.seek(SeekFrom::Start(entry.offset))?;

        let mut encrypted_data = vec![0u8; entry.size as usize];
        reader.read_exact(&mut encrypted_data)?;

        let mut key = RgssKey::with_state(self.version, entry.key);
        let decrypted_data = key.decrypt_content(&encrypted_data);

        Ok(decrypted_data)
    }

    /// Get the archive version
    pub fn version(&self) -> RgssVersion {
        self.version
    }

    /// Get the archive path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveReader for RgssReader {
    type Entry = RgssEntry;

    fn open<P: AsRef<Path>>(path: P) -> ArchiverResult<Self> {
        let path = path.as_ref();

        // Detect version
        let version = RgssVersion::detect(path)?;

        // Read entries based on version
        let entries = match version {
            RgssVersion::V1 => Self::read_v1(path)?,
            RgssVersion::V3 => Self::read_v3(path)?,
        };

        Ok(Self {
            path: path.to_path_buf(),
            version,
            entries,
        })
    }

    fn entries(&self) -> &[RgssEntry] {
        &self.entries
    }

    fn extract_all<P: AsRef<Path>>(&self, output_dir: P) -> ArchiverResult<usize> {
        let output_dir = output_dir.as_ref();
        let mut count = 0;

        for entry in &self.entries {
            self.extract_entry(&entry.name, output_dir)?;
            count += 1;
        }

        Ok(count)
    }

    fn extract_entry<P: AsRef<Path>>(&self, entry_name: &str, output_dir: P) -> ArchiverResult<()> {
        let output_dir = output_dir.as_ref();

        // Find the entry
        let entry = self
            .entries
            .iter()
            .find(|e| e.name == entry_name)
            .ok_or_else(|| ArchiverError::FileNotFound(entry_name.to_string()))?;

        // Extract to memory
        let data = self.extract_to_memory(entry)?;

        // Write to file
        let output_path = entry.output_path(output_dir);

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgss_entry_output_path() {
        let entry = RgssEntry {
            name: "Data\\Scripts.rxdata".to_string(),
            size: 1024,
            offset: 0,
            key: 0,
        };

        let output = entry.output_path(Path::new("/tmp/output"));
        assert!(output.to_string_lossy().contains("Data"));
    }
}
