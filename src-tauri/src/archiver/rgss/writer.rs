//! RGSS Archive Writer (Repacker)

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use super::{RgssKey, RgssVersion, MAGIC, V1_INITIAL_KEY};
use crate::archiver::{ArchiveWriter, ArchiverError, ArchiverResult};

/// File entry to be packed
#[derive(Debug)]
struct PackEntry {
    /// Name in the archive (with backslashes for Windows-style paths)
    archive_name: String,
    /// File data
    data: Vec<u8>,
}

/// RGSS Archive Writer for creating .rgssad, .rgss2a, and .rgss3a files
pub struct RgssWriter {
    /// Archive version to create
    version: RgssVersion,
    /// Files to pack
    entries: Vec<PackEntry>,
    /// Initial key for V3 archives (optional, random if not set)
    v3_initial_key: Option<u32>,
}

impl RgssWriter {
    /// Create a new writer for the specified version
    pub fn for_version(version: RgssVersion) -> Self {
        Self {
            version,
            entries: Vec::new(),
            v3_initial_key: None,
        }
    }

    /// Set a specific initial key for V3 archives
    /// If not set, a random key will be generated
    pub fn with_v3_key(mut self, key: u32) -> Self {
        self.v3_initial_key = Some(key);
        self
    }

    /// Write V1 format archive
    fn write_v1<P: AsRef<Path>>(&self, output_path: P) -> ArchiverResult<()> {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        writer.write_all(MAGIC)?;
        writer.write_all(&[self.version.header_byte()])?;

        // Initialize key
        let mut key = RgssKey::with_state(RgssVersion::V1, V1_INITIAL_KEY);

        for entry in &self.entries {
            // Encrypt and write name length
            let name_bytes = entry.archive_name.as_bytes();
            let encrypted_len = key.encrypt_int(name_bytes.len() as u32);
            writer.write_all(&encrypted_len.to_le_bytes())?;

            // Encrypt and write name
            let encrypted_name = key.encrypt_string_v1(name_bytes);
            writer.write_all(&encrypted_name)?;

            // Encrypt and write size
            let encrypted_size = key.encrypt_int(entry.data.len() as u32);
            writer.write_all(&encrypted_size.to_le_bytes())?;

            // Encrypt and write file data
            let mut content_key = RgssKey::with_state(RgssVersion::V1, key.current());
            let encrypted_data = content_key.encrypt_content(&entry.data);
            writer.write_all(&encrypted_data)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Write V3 format archive
    fn write_v3<P: AsRef<Path>>(&self, output_path: P) -> ArchiverResult<()> {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        // Generate or use provided initial key
        let initial_key = self.v3_initial_key.unwrap_or_else(|| {
            // Generate a random key
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u32)
                .unwrap_or(0xDEADBEEF);
            seed ^ 0x8E73A5C1
        });

        // Write header
        writer.write_all(MAGIC)?;
        writer.write_all(&[self.version.header_byte()])?;

        // Write initial key
        writer.write_all(&initial_key.to_le_bytes())?;

        // Initialize key
        let mut key = RgssKey::with_state(RgssVersion::V3, initial_key);
        key.step();

        // First pass: calculate offsets
        // Header (8) + Initial Key (4) + Entry headers + End marker (4)
        let header_size: u64 = 8 + 4;
        let end_marker_size: u64 = 4;

        // Calculate entry header sizes
        let mut entry_headers_size: u64 = 0;
        for entry in &self.entries {
            // offset(4) + size(4) + key(4) + name_len(4) + name
            entry_headers_size += 16 + entry.archive_name.len() as u64;
        }

        // Calculate data offsets
        let mut current_offset = header_size + entry_headers_size + end_marker_size;
        let mut entry_offsets = Vec::new();
        let mut entry_keys = Vec::new();

        for entry in &self.entries {
            entry_offsets.push(current_offset as u32);
            // Generate a unique key for each file
            let file_key = initial_key.wrapping_mul(current_offset as u32 + 1);
            entry_keys.push(file_key);
            current_offset += entry.data.len() as u64;
        }

        // Write entry headers
        for (i, entry) in self.entries.iter().enumerate() {
            let name_bytes = entry.archive_name.as_bytes();

            // Encrypt and write offset
            let encrypted_offset = key.encrypt_int(entry_offsets[i]);
            writer.write_all(&encrypted_offset.to_le_bytes())?;

            // Encrypt and write size
            let encrypted_size = key.encrypt_int(entry.data.len() as u32);
            writer.write_all(&encrypted_size.to_le_bytes())?;

            // Encrypt and write file key
            let encrypted_key = key.encrypt_int(entry_keys[i]);
            writer.write_all(&encrypted_key.to_le_bytes())?;

            // Encrypt and write name length
            let encrypted_name_len = key.encrypt_int(name_bytes.len() as u32);
            writer.write_all(&encrypted_name_len.to_le_bytes())?;

            // Encrypt and write name (V3 uses different string encryption)
            let encrypted_name = key.encrypt_string_v3(name_bytes);
            writer.write_all(&encrypted_name)?;
        }

        // Write end marker (encrypted 0)
        let encrypted_end = key.encrypt_int(0);
        writer.write_all(&encrypted_end.to_le_bytes())?;

        // Write file data
        for (i, entry) in self.entries.iter().enumerate() {
            let mut content_key = RgssKey::with_state(RgssVersion::V3, entry_keys[i]);
            let encrypted_data = content_key.encrypt_content(&entry.data);
            writer.write_all(&encrypted_data)?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl ArchiveWriter for RgssWriter {
    fn new() -> Self {
        // Default to V3 format
        Self::for_version(RgssVersion::V3)
    }

    fn add_file<P: AsRef<Path>>(&mut self, path: P, archive_name: &str) -> ArchiverResult<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ArchiverError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Normalize path separators to backslash (Windows-style for RGSS)
        let normalized_name = archive_name.replace('/', "\\");

        self.entries.push(PackEntry {
            archive_name: normalized_name,
            data,
        });

        Ok(())
    }

    fn add_directory<P: AsRef<Path>>(
        &mut self,
        dir: P,
        base_path: Option<&str>,
    ) -> ArchiverResult<usize> {
        let dir = dir.as_ref();
        let mut count = 0;

        if !dir.exists() {
            return Err(ArchiverError::FileNotFound(
                dir.to_string_lossy().to_string(),
            ));
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                // Calculate archive name
                let relative_path = path.strip_prefix(dir).map_err(|e| {
                    ArchiverError::InvalidFormat(format!("Failed to get relative path: {}", e))
                })?;

                let archive_name = match base_path {
                    Some(base) => format!(
                        "{}\\{}",
                        base,
                        relative_path.to_string_lossy().replace('/', "\\")
                    ),
                    None => relative_path.to_string_lossy().replace('/', "\\"),
                };

                self.add_file(path, &archive_name)?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn write<P: AsRef<Path>>(self, output_path: P) -> ArchiverResult<()> {
        match self.version {
            RgssVersion::V1 => self.write_v1(output_path),
            RgssVersion::V3 => self.write_v3(output_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archiver::ArchiveReader;
    use super::super::RgssReader;
    use std::fs;
    use std::io::Write as IoWrite;
    use tempfile::TempDir;

    #[test]
    fn test_pack_unpack_v1() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let test_file = data_dir.join("test.txt");
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"Hello, RGSS!").unwrap();

        // Pack
        let archive_path = temp_dir.path().join("test.rgssad");
        let mut writer = RgssWriter::for_version(RgssVersion::V1);
        writer.add_file(&test_file, "Data\\test.txt").unwrap();
        writer.write(&archive_path).unwrap();

        // Verify archive exists
        assert!(archive_path.exists());

        // Unpack
        let reader = RgssReader::open(&archive_path).unwrap();
        assert_eq!(reader.entries().len(), 1);
        assert_eq!(reader.entries()[0].name, "Data\\test.txt");

        let extracted = reader.extract_to_memory(&reader.entries()[0]).unwrap();
        assert_eq!(extracted, b"Hello, RGSS!");
    }

    #[test]
    fn test_pack_unpack_v3() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let test_file = data_dir.join("test.txt");
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"Hello, RGSS3!").unwrap();

        // Pack
        let archive_path = temp_dir.path().join("test.rgss3a");
        let mut writer = RgssWriter::for_version(RgssVersion::V3);
        writer.add_file(&test_file, "Data\\test.txt").unwrap();
        writer.write(&archive_path).unwrap();

        // Verify archive exists
        assert!(archive_path.exists());

        // Unpack
        let reader = RgssReader::open(&archive_path).unwrap();
        assert_eq!(reader.entries().len(), 1);
        assert_eq!(reader.entries()[0].name, "Data\\test.txt");

        let extracted = reader.extract_to_memory(&reader.entries()[0]).unwrap();
        assert_eq!(extracted, b"Hello, RGSS3!");
    }
}
