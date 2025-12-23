//! Parser for Map*.json files
//!
//! Map files contain event data with pages and command lists.
//! Each event can have multiple pages with different conditions and commands.

use super::event_page::{EventPageParser, FileExtractionResult, FileInjectionResult};
use crate::parser::types::{
    ExtractionContext, ExtractionOptions, InjectionOptions, TranslationFile, TranslationPath,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parser for Map*.json files
pub struct MapParser {
    /// Event page parser
    page_parser: EventPageParser,
}

impl MapParser {
    /// Create a new Map parser
    pub fn new() -> Self {
        Self {
            page_parser: EventPageParser::new(),
        }
    }

    /// Create with a custom page parser
    pub fn with_page_parser(page_parser: EventPageParser) -> Self {
        Self { page_parser }
    }

    /// Extract translations from a Map JSON content
    pub fn extract(
        &self,
        json: &Value,
        file_name: &str,
        options: &ExtractionOptions,
    ) -> FileExtractionResult {
        let mut result = FileExtractionResult::new(file_name);

        // Get map display name for context
        let map_name = json
            .get("displayName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Fallback to file name without extension
                Path::new(file_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            });

        // Get events array
        let events = match json.get("events") {
            Some(Value::Array(arr)) => arr,
            _ => {
                result.add_warning("Map file does not contain events array");
                return result;
            }
        };

        for (event_idx, event) in events.iter().enumerate() {
            // Skip null events
            if event.is_null() {
                continue;
            }

            // Get event metadata
            let event_name = event
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let event_id = event
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|id| id as usize)
                .unwrap_or(event_idx);

            // Create base context for this event
            let mut base_context = ExtractionContext::new(file_name)
                .with_event_id(event_id)
                .with_max_preceding_lines(options.max_preceding_lines);

            if let Some(name) = &map_name {
                base_context = base_context.with_map_name(name.clone());
            }

            if let Some(name) = &event_name {
                base_context = base_context.with_event_name(name.clone());
            }

            // Get event pages
            if let Some(pages) = event.get("pages") {
                let event_path = TranslationPath::new()
                    .append_key("events")
                    .append_index(event_idx);

                let units = self.page_parser.extract_from_pages(
                    pages,
                    &event_path,
                    &base_context,
                    options,
                );
                result.add_units(units);
            }
        }

        result
    }

    /// Extract from a file path
    pub fn extract_file(
        &self,
        path: &Path,
        options: &ExtractionOptions,
    ) -> Result<FileExtractionResult, MapError> {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Map.json");

        Ok(self.extract(&json, file_name, options))
    }

    /// Inject translations back into Map JSON content
    pub fn inject(
        &self,
        json: &mut Value,
        translations: &HashMap<String, String>,
        options: &InjectionOptions,
    ) -> FileInjectionResult {
        let mut result = FileInjectionResult::new();

        let events = match json.get_mut("events") {
            Some(Value::Array(arr)) => arr,
            _ => {
                result.warnings.push("Map file does not contain events array".to_string());
                return result;
            }
        };

        for (event_idx, event) in events.iter_mut().enumerate() {
            if event.is_null() {
                continue;
            }

            let event_id = event
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|id| id as usize)
                .unwrap_or(event_idx);

            let base_context = ExtractionContext::new("Map.json")
                .with_event_id(event_id);

            if let Some(pages) = event.get_mut("pages") {
                let event_path = TranslationPath::new()
                    .append_key("events")
                    .append_index(event_idx);

                let inject_result = self.page_parser.inject_to_pages(
                    pages,
                    translations,
                    &event_path,
                    &base_context,
                    options,
                );
                result.merge(inject_result);
            }
        }

        result
    }

    /// Inject translations to a file
    pub fn inject_file(
        &self,
        path: &Path,
        translations: &HashMap<String, String>,
        options: &InjectionOptions,
    ) -> Result<FileInjectionResult, MapError> {
        let content = fs::read_to_string(path)?;
        let mut json: Value = serde_json::from_str(&content)?;

        let result = self.inject(&mut json, translations, options);

        if result.modified {
            let output = serde_json::to_string_pretty(&json)?;
            fs::write(path, output)?;
        }

        Ok(result)
    }

    /// Convert extraction result to TranslationFile
    pub fn to_translation_file(&self, result: FileExtractionResult) -> TranslationFile {
        let mut file = TranslationFile::new(&result.source_file);
        file.add_units(result.units);
        file
    }
}

impl Default for MapParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for Map parsing
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid structure: {0}")]
    InvalidStructure(String),
}

/// Check if a file is a map file
pub fn is_map_file(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        // Map files are named Map001.json, Map002.json, etc.
        name.starts_with("Map") && name.ends_with(".json") && name != "MapInfos.json"
    } else {
        false
    }
}

/// Get all map files in a directory
pub fn find_map_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut maps = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_map_file(&path) {
                maps.push(path);
            }
        }
    }

    // Sort by name for consistent order
    maps.sort();
    maps
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_map_json() -> Value {
        json!({
            "autoplayBgm": false,
            "displayName": "村",
            "width": 17,
            "height": 13,
            "events": [
                null,
                {
                    "id": 1,
                    "name": "村人A",
                    "x": 10,
                    "y": 8,
                    "pages": [
                        {
                            "list": [
                                {"code": 101, "indent": 0, "parameters": ["Actor1", 0, 0, 2, "村人A"]},
                                {"code": 401, "indent": 0, "parameters": ["こんにちは！"]},
                                {"code": 401, "indent": 0, "parameters": ["良い天気ですね。"]},
                                {"code": 0, "indent": 0, "parameters": []}
                            ]
                        },
                        {
                            "list": [
                                {"code": 101, "indent": 0, "parameters": ["Actor1", 0, 0, 2, "村人A"]},
                                {"code": 401, "indent": 0, "parameters": ["また会いましたね！"]},
                                {"code": 0, "indent": 0, "parameters": []}
                            ]
                        }
                    ]
                },
                {
                    "id": 2,
                    "name": "看板",
                    "x": 5,
                    "y": 3,
                    "pages": [
                        {
                            "list": [
                                {"code": 401, "indent": 0, "parameters": ["「村への入り口」"]},
                                {"code": 0, "indent": 0, "parameters": []}
                            ]
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn test_extract_map() {
        let parser = MapParser::new();
        let json = make_map_json();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "Map001.json", &options);

        // Should have extracted dialogue from both events
        assert!(result.unit_count() > 0);

        // Check speakers
        assert!(result.speakers.contains(&"村人A".to_string()));
    }

    #[test]
    fn test_extract_map_with_context() {
        let parser = MapParser::new();
        let json = make_map_json();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "Map001.json", &options);

        // Check that units have map context
        for unit in &result.units {
            assert_eq!(unit.context.map_name, Some("村".to_string()));
        }
    }

    #[test]
    fn test_extract_multiple_pages() {
        let parser = MapParser::new();
        let json = make_map_json();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "Map001.json", &options);

        // Should have dialogue from both pages of event 1 and from event 2
        let unique_texts: std::collections::HashSet<_> = result.units
            .iter()
            .map(|u| &u.original)
            .collect();

        assert!(unique_texts.iter().any(|t| t.contains("こんにちは")));
        assert!(unique_texts.iter().any(|t| t.contains("また会いました")));
        assert!(unique_texts.iter().any(|t| t.contains("村への入り口")));
    }

    #[test]
    fn test_inject_map() {
        let parser = MapParser::new();
        let mut json = make_map_json();
        let options = InjectionOptions::default();

        // First extract to get unit IDs
        let extract_result = parser.extract(&json, "Map001.json", &ExtractionOptions::default());

        // Create translations
        let mut translations = HashMap::new();
        for unit in &extract_result.units {
            if unit.original.contains("こんにちは") {
                translations.insert(unit.id.clone(), "Hello!\nNice weather.".to_string());
            }
        }

        // Inject
        let result = parser.inject(&mut json, &translations, &options);

        assert!(result.applied > 0 || translations.is_empty());
    }

    #[test]
    fn test_null_events() {
        let parser = MapParser::new();
        let json = json!({
            "displayName": "Empty Map",
            "events": [null, null, null]
        });
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "Map001.json", &options);

        assert_eq!(result.unit_count(), 0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_is_map_file() {
        assert!(is_map_file(Path::new("Map001.json")));
        assert!(is_map_file(Path::new("Map123.json")));
        assert!(!is_map_file(Path::new("MapInfos.json")));
        assert!(!is_map_file(Path::new("CommonEvents.json")));
        assert!(!is_map_file(Path::new("Actors.json")));
    }

    #[test]
    fn test_to_translation_file() {
        let parser = MapParser::new();
        let json = make_map_json();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "Map001.json", &options);
        let trans_file = parser.to_translation_file(result);

        assert_eq!(trans_file.source_file, "Map001.json");
        assert!(trans_file.units.len() > 0);
    }
}
