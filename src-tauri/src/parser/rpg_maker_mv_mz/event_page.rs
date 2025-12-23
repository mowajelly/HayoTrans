//! Event page parsing logic
//!
//! This module provides the core logic for extracting and injecting translations
//! from event page command lists.

use super::command::{parse_commands, EventCommand};
use super::handlers::HandlerRegistry;
use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, InjectionOptions, InjectionResult,
    TranslationPath, TranslationUnit,
};
use serde_json::Value;
use std::collections::HashMap;

/// Parser for event pages (command lists)
pub struct EventPageParser {
    /// Handler registry
    handlers: HandlerRegistry,
}

impl EventPageParser {
    /// Create a new event page parser
    pub fn new() -> Self {
        Self {
            handlers: HandlerRegistry::with_defaults(),
        }
    }

    /// Create with a custom handler registry
    pub fn with_handlers(handlers: HandlerRegistry) -> Self {
        Self { handlers }
    }

    /// Extract translation units from a page's command list
    pub fn extract_from_list(
        &self,
        list: &Value,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> Vec<TranslationUnit> {
        let commands = parse_commands(list);
        self.extract_from_commands(&commands, path_prefix, context, options)
    }

    /// Extract translation units from parsed commands
    pub fn extract_from_commands(
        &self,
        commands: &[EventCommand],
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> Vec<TranslationUnit> {
        let mut units = Vec::new();
        let list_path = path_prefix.append_key("list");
        let mut index = 0;

        while index < commands.len() {
            let cmd = &commands[index];
            let code = EventCode::from(cmd.code);

            // Get handler for this code
            if let Some(handler) = self.handlers.get(code) {
                let result = handler.extract(commands, index, &list_path, context, options);

                // Update speaker if the handler provided one
                if let Some(speaker) = result.speaker_update {
                    context.set_speaker(speaker);
                }

                // Add extracted units
                units.extend(result.units);

                // Add to preceding lines if applicable
                if let Some(text) = result.add_to_preceding {
                    context.add_preceding_line(text);
                }

                // Skip consumed commands
                index += result.consumed;
            } else {
                // No handler, skip this command
                index += 1;
            }
        }

        units
    }

    /// Inject translations into a page's command list
    pub fn inject_to_list(
        &self,
        list: &mut Value,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        context: &ExtractionContext,
        options: &InjectionOptions,
    ) -> InjectionResult {
        let mut commands = parse_commands(list);
        let result = self.inject_to_commands(
            &mut commands,
            translations,
            path_prefix,
            context,
            options,
        );

        // Update the JSON list with modified commands
        if result.commands_modified > 0 {
            *list = super::command::commands_to_json(&commands);
        }

        result
    }

    /// Inject translations into parsed commands
    pub fn inject_to_commands(
        &self,
        commands: &mut Vec<EventCommand>,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        context: &ExtractionContext,
        options: &InjectionOptions,
    ) -> InjectionResult {
        let mut result = InjectionResult::new();
        let list_path = path_prefix.append_key("list");
        let mut index = 0;

        while index < commands.len() {
            let code = EventCode::from(commands[index].code);

            if let Some(handler) = self.handlers.get(code) {
                let handler_result = handler.inject(
                    commands,
                    index,
                    translations,
                    &list_path,
                    context,
                    options,
                );
                result.merge(handler_result);
            }

            index += 1;
        }

        result
    }

    /// Extract from multiple pages
    pub fn extract_from_pages(
        &self,
        pages: &Value,
        path_prefix: &TranslationPath,
        base_context: &ExtractionContext,
        options: &ExtractionOptions,
    ) -> Vec<TranslationUnit> {
        let mut units = Vec::new();

        if let Some(pages_arr) = pages.as_array() {
            for (page_idx, page) in pages_arr.iter().enumerate() {
                if page.is_null() {
                    continue;
                }

                let page_path = path_prefix.append_key("pages").append_index(page_idx);
                let mut context = base_context.for_page(page_idx);

                if let Some(list) = page.get("list") {
                    let page_units = self.extract_from_list(list, &page_path, &mut context, options);
                    units.extend(page_units);
                }
            }
        }

        units
    }

    /// Inject into multiple pages
    pub fn inject_to_pages(
        &self,
        pages: &mut Value,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        base_context: &ExtractionContext,
        options: &InjectionOptions,
    ) -> InjectionResult {
        let mut result = InjectionResult::new();

        if let Some(pages_arr) = pages.as_array_mut() {
            for (page_idx, page) in pages_arr.iter_mut().enumerate() {
                if page.is_null() {
                    continue;
                }

                let page_path = path_prefix.append_key("pages").append_index(page_idx);
                let context = base_context.for_page(page_idx);

                if let Some(list) = page.get_mut("list") {
                    let page_result = self.inject_to_list(
                        list,
                        translations,
                        &page_path,
                        &context,
                        options,
                    );
                    result.merge(page_result);
                }
            }
        }

        result
    }
}

impl Default for EventPageParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of extraction from a file
#[derive(Debug, Clone)]
pub struct FileExtractionResult {
    /// Extracted translation units
    pub units: Vec<TranslationUnit>,
    /// Source file name
    pub source_file: String,
    /// List of unique speakers found
    pub speakers: Vec<String>,
    /// Any warnings encountered
    pub warnings: Vec<String>,
}

impl FileExtractionResult {
    /// Create a new result
    pub fn new(source_file: impl Into<String>) -> Self {
        Self {
            units: Vec::new(),
            source_file: source_file.into(),
            speakers: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add units and update metadata
    pub fn add_units(&mut self, units: Vec<TranslationUnit>) {
        // Collect unique speakers
        for unit in &units {
            if let Some(speaker) = &unit.speaker {
                if !self.speakers.contains(speaker) {
                    self.speakers.push(speaker.clone());
                }
            }
        }
        self.units.extend(units);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Get total unit count
    pub fn unit_count(&self) -> usize {
        self.units.len()
    }
}

/// Result of injection to a file
#[derive(Debug, Clone, Default)]
pub struct FileInjectionResult {
    /// Total translations applied
    pub applied: usize,
    /// Translations not found
    pub not_found: usize,
    /// Commands modified
    pub commands_modified: usize,
    /// Warnings
    pub warnings: Vec<String>,
    /// Whether the file was modified
    pub modified: bool,
}

impl FileInjectionResult {
    /// Create a new result
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge an InjectionResult
    pub fn merge(&mut self, result: InjectionResult) {
        self.applied += result.applied;
        self.not_found += result.not_found;
        self.commands_modified += result.commands_modified;
        self.warnings.extend(result.warnings);
        if result.commands_modified > 0 {
            self.modified = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_test_list() -> Value {
        json!([
            {"code": 101, "indent": 0, "parameters": ["Actor1", 0, 0, 2, "村人A"]},
            {"code": 401, "indent": 0, "parameters": ["こんにちは！"]},
            {"code": 401, "indent": 0, "parameters": ["元気ですか？"]},
            {"code": 102, "indent": 0, "parameters": [["はい", "いいえ"], 0, 1, 2, 0]},
            {"code": 402, "indent": 1, "parameters": [0, "はい"]},
            {"code": 0, "indent": 1, "parameters": []},
            {"code": 402, "indent": 1, "parameters": [1, "いいえ"]},
            {"code": 0, "indent": 1, "parameters": []},
            {"code": 0, "indent": 0, "parameters": []}
        ])
    }

    #[test]
    fn test_extract_from_list() {
        let parser = EventPageParser::new();
        let list = make_test_list();
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let units = parser.extract_from_list(&list, &path, &mut context, &options);

        // Should have: 1 dialogue block + 2 choice options + 2 choice branches = 5 units
        assert!(units.len() >= 3); // At least dialogue + 2 choices in 102
        
        // Check dialogue was extracted with speaker
        let dialogue = units.iter().find(|u| u.code == EventCode::ShowTextBody);
        assert!(dialogue.is_some());
        let dialogue = dialogue.unwrap();
        assert_eq!(dialogue.speaker, Some("村人A".to_string()));
        assert!(dialogue.original.contains("こんにちは"));
    }

    #[test]
    fn test_inject_to_list() {
        let parser = EventPageParser::new();
        let mut list = make_test_list();
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        // First extract to get the unit IDs
        let mut extract_context = ExtractionContext::new("test.json");
        let units = parser.extract_from_list(&list, &path, &mut extract_context, &ExtractionOptions::default());
        
        // Create translations map
        let mut translations = HashMap::new();
        for unit in &units {
            if unit.code == EventCode::ShowTextBody {
                translations.insert(unit.id.clone(), "Hello!\nHow are you?".to_string());
            }
        }

        // Inject
        let result = parser.inject_to_list(&mut list, &translations, &path, &context, &options);

        assert!(result.applied > 0 || result.commands_modified > 0);
    }

    #[test]
    fn test_extract_from_pages() {
        let parser = EventPageParser::new();
        let pages = json!([
            {
                "list": [
                    {"code": 101, "indent": 0, "parameters": ["", 0, 0, 2, "NPC1"]},
                    {"code": 401, "indent": 0, "parameters": ["Page 1 dialogue"]},
                    {"code": 0, "indent": 0, "parameters": []}
                ]
            },
            null,
            {
                "list": [
                    {"code": 101, "indent": 0, "parameters": ["", 0, 0, 2, "NPC2"]},
                    {"code": 401, "indent": 0, "parameters": ["Page 3 dialogue"]},
                    {"code": 0, "indent": 0, "parameters": []}
                ]
            }
        ]);

        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let units = parser.extract_from_pages(&pages, &path, &context, &options);

        // Should have 2 dialogue units (page 0 and page 2)
        assert_eq!(units.len(), 2);
        
        // Check both speakers are found
        let speakers: Vec<_> = units.iter().filter_map(|u| u.speaker.as_ref()).collect();
        assert!(speakers.contains(&&"NPC1".to_string()));
        assert!(speakers.contains(&&"NPC2".to_string()));
    }

    #[test]
    fn test_file_extraction_result() {
        let mut result = FileExtractionResult::new("test.json");
        
        let unit1 = TranslationUnit::new(
            "1".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Test".to_string(),
        ).with_speaker(Some("NPC1".to_string()));

        let unit2 = TranslationUnit::new(
            "2".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Test 2".to_string(),
        ).with_speaker(Some("NPC2".to_string()));

        let unit3 = TranslationUnit::new(
            "3".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Test 3".to_string(),
        ).with_speaker(Some("NPC1".to_string())); // Duplicate speaker

        result.add_units(vec![unit1, unit2, unit3]);

        assert_eq!(result.unit_count(), 3);
        assert_eq!(result.speakers.len(), 2); // Only unique
        assert!(result.speakers.contains(&"NPC1".to_string()));
        assert!(result.speakers.contains(&"NPC2".to_string()));
    }
}
