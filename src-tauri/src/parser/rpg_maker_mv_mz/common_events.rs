//! Parser for CommonEvents.json files
//!
//! CommonEvents.json contains an array of common event objects,
//! each with a list of commands that can be extracted for translation.

use super::event_page::{EventPageParser, FileExtractionResult, FileInjectionResult};
use crate::parser::types::{
    ExtractionContext, ExtractionOptions, InjectionOptions, TranslationFile, TranslationPath,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parser for CommonEvents.json
pub struct CommonEventsParser {
    /// Event page parser
    page_parser: EventPageParser,
}

impl CommonEventsParser {
    /// Create a new CommonEvents parser
    pub fn new() -> Self {
        Self {
            page_parser: EventPageParser::new(),
        }
    }

    /// Create with a custom page parser
    pub fn with_page_parser(page_parser: EventPageParser) -> Self {
        Self { page_parser }
    }

    /// Extract translations from CommonEvents.json content
    pub fn extract(
        &self,
        json: &Value,
        file_name: &str,
        options: &ExtractionOptions,
    ) -> FileExtractionResult {
        let mut result = FileExtractionResult::new(file_name);

        // CommonEvents.json is an array of event objects
        let events = match json.as_array() {
            Some(arr) => arr,
            None => {
                result.add_warning("CommonEvents.json is not an array");
                return result;
            }
        };

        for (event_idx, event) in events.iter().enumerate() {
            // Skip null events (common events array can have null entries)
            if event.is_null() {
                continue;
            }

            // Get event name for context
            let event_name = event
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Get event ID
            let event_id = event
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|id| id as usize)
                .unwrap_or(event_idx);

            // Create context for this event
            let mut context = ExtractionContext::new(file_name)
                .with_event_id(event_id)
                .with_max_preceding_lines(options.max_preceding_lines);

            if let Some(name) = &event_name {
                context = context.with_event_name(name.clone());
            }

            // Get the command list
            if let Some(list) = event.get("list") {
                let event_path = TranslationPath::new().append_index(event_idx);
                let units = self.page_parser.extract_from_list(
                    list,
                    &event_path,
                    &mut context,
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
    ) -> Result<FileExtractionResult, CommonEventsError> {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("CommonEvents.json");
        
        Ok(self.extract(&json, file_name, options))
    }

    /// Inject translations back into CommonEvents.json content
    pub fn inject(
        &self,
        json: &mut Value,
        translations: &HashMap<String, String>,
        options: &InjectionOptions,
    ) -> FileInjectionResult {
        let mut result = FileInjectionResult::new();

        let events = match json.as_array_mut() {
            Some(arr) => arr,
            None => {
                result.warnings.push("CommonEvents.json is not an array".to_string());
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

            let context = ExtractionContext::new("CommonEvents.json")
                .with_event_id(event_id);

            if let Some(list) = event.get_mut("list") {
                let event_path = TranslationPath::new().append_index(event_idx);
                let inject_result = self.page_parser.inject_to_list(
                    list,
                    translations,
                    &event_path,
                    &context,
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
    ) -> Result<FileInjectionResult, CommonEventsError> {
        let content = fs::read_to_string(path)?;
        let mut json: Value = serde_json::from_str(&content)?;
        
        let result = self.inject(&mut json, translations, options);
        
        if result.modified {
            // Write back with pretty formatting
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

impl Default for CommonEventsParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for CommonEvents parsing
#[derive(Debug, thiserror::Error)]
pub enum CommonEventsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_common_events() -> Value {
        json!([
            null,
            {
                "id": 1,
                "name": "Welcome Event",
                "list": [
                    {"code": 101, "indent": 0, "parameters": ["Actor1", 0, 0, 2, "村長"]},
                    {"code": 401, "indent": 0, "parameters": ["ようこそ、勇者よ！"]},
                    {"code": 401, "indent": 0, "parameters": ["この村へ来てくれて嬉しいぞ。"]},
                    {"code": 0, "indent": 0, "parameters": []}
                ]
            },
            {
                "id": 2,
                "name": "Shop Event",
                "list": [
                    {"code": 101, "indent": 0, "parameters": ["Actor2", 0, 0, 2, "商人"]},
                    {"code": 401, "indent": 0, "parameters": ["いらっしゃい！"]},
                    {"code": 102, "indent": 0, "parameters": [["買う", "売る", "やめる"], 0, 2, 2, 0]},
                    {"code": 402, "indent": 1, "parameters": [0, "買う"]},
                    {"code": 0, "indent": 1, "parameters": []},
                    {"code": 402, "indent": 1, "parameters": [1, "売る"]},
                    {"code": 0, "indent": 1, "parameters": []},
                    {"code": 402, "indent": 1, "parameters": [2, "やめる"]},
                    {"code": 0, "indent": 1, "parameters": []},
                    {"code": 0, "indent": 0, "parameters": []}
                ]
            }
        ])
    }

    #[test]
    fn test_extract_common_events() {
        let parser = CommonEventsParser::new();
        let json = make_common_events();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "CommonEvents.json", &options);

        // Should have extracted dialogue from both events
        assert!(result.unit_count() > 0);
        
        // Check speakers
        assert!(result.speakers.contains(&"村長".to_string()));
        assert!(result.speakers.contains(&"商人".to_string()));
    }

    #[test]
    fn test_extract_with_context() {
        let parser = CommonEventsParser::new();
        let json = make_common_events();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "CommonEvents.json", &options);

        // Find a unit and check its context
        let dialogue_unit = result.units.iter()
            .find(|u| u.original.contains("ようこそ"));
        
        assert!(dialogue_unit.is_some());
        let unit = dialogue_unit.unwrap();
        assert_eq!(unit.speaker, Some("村長".to_string()));
        assert_eq!(unit.context.file_name, Some("CommonEvents.json".to_string()));
    }

    #[test]
    fn test_inject_common_events() {
        let parser = CommonEventsParser::new();
        let mut json = make_common_events();
        let options = InjectionOptions::default();

        // First extract to get unit IDs
        let extract_result = parser.extract(&json, "CommonEvents.json", &ExtractionOptions::default());
        
        // Create translations
        let mut translations = HashMap::new();
        for unit in &extract_result.units {
            if unit.original.contains("ようこそ") {
                translations.insert(unit.id.clone(), "Welcome, hero!".to_string());
            }
        }

        // Inject
        let result = parser.inject(&mut json, &translations, &options);

        assert!(result.applied > 0 || translations.is_empty());
    }

    #[test]
    fn test_to_translation_file() {
        let parser = CommonEventsParser::new();
        let json = make_common_events();
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "CommonEvents.json", &options);
        let trans_file = parser.to_translation_file(result);

        assert_eq!(trans_file.source_file, "CommonEvents.json");
        assert!(trans_file.units.len() > 0);
        assert_eq!(trans_file.version, "1.0");
    }

    #[test]
    fn test_null_handling() {
        let parser = CommonEventsParser::new();
        let json = json!([
            null,
            null,
            null,
        ]);
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "CommonEvents.json", &options);

        // Should handle all nulls gracefully
        assert_eq!(result.unit_count(), 0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_empty_list() {
        let parser = CommonEventsParser::new();
        let json = json!([
            {
                "id": 1,
                "name": "Empty Event",
                "list": []
            }
        ]);
        let options = ExtractionOptions::default();

        let result = parser.extract(&json, "CommonEvents.json", &options);

        assert_eq!(result.unit_count(), 0);
    }
}
