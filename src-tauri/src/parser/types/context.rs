//! Extraction context for tracking state during parsing
//!
//! The ExtractionContext maintains state while traversing event command lists,
//! such as the current speaker and preceding dialogue.

use super::TranslationContext;
use std::collections::VecDeque;

/// Context maintained during extraction of a single event page
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// Source file name being processed
    pub file_name: String,
    /// Map name (for map files)
    pub map_name: Option<String>,
    /// Current event name
    pub event_name: Option<String>,
    /// Current event ID
    pub event_id: Option<usize>,
    /// Current page index
    pub page_index: usize,
    /// Current speaker from most recent ShowText (101) command
    pub current_speaker: Option<String>,
    /// Recent dialogue lines for providing context
    preceding_lines: VecDeque<String>,
    /// Maximum number of preceding lines to keep
    max_preceding_lines: usize,
    /// Counter for generating unique IDs within this context
    unit_counter: usize,
}

impl ExtractionContext {
    /// Create a new extraction context
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            map_name: None,
            event_name: None,
            event_id: None,
            page_index: 0,
            current_speaker: None,
            preceding_lines: VecDeque::new(),
            max_preceding_lines: 5,
            unit_counter: 0,
        }
    }

    /// Set the map name
    pub fn with_map_name(mut self, map_name: impl Into<String>) -> Self {
        self.map_name = Some(map_name.into());
        self
    }

    /// Set the event name
    pub fn with_event_name(mut self, event_name: impl Into<String>) -> Self {
        self.event_name = Some(event_name.into());
        self
    }

    /// Set the event ID
    pub fn with_event_id(mut self, event_id: usize) -> Self {
        self.event_id = Some(event_id);
        self
    }

    /// Set the page index
    pub fn with_page_index(mut self, page_index: usize) -> Self {
        self.page_index = page_index;
        self
    }

    /// Set the maximum number of preceding lines to keep
    pub fn with_max_preceding_lines(mut self, max: usize) -> Self {
        self.max_preceding_lines = max;
        self
    }

    /// Update the current speaker
    pub fn set_speaker(&mut self, speaker: Option<String>) {
        self.current_speaker = speaker;
    }

    /// Add a preceding dialogue line for context
    pub fn add_preceding_line(&mut self, line: impl Into<String>) {
        let line = line.into();
        if !line.trim().is_empty() {
            if self.preceding_lines.len() >= self.max_preceding_lines {
                self.preceding_lines.pop_front();
            }
            self.preceding_lines.push_back(line);
        }
    }

    /// Get the preceding lines as a vector
    pub fn get_preceding_lines(&self) -> Vec<String> {
        self.preceding_lines.iter().cloned().collect()
    }

    /// Clear preceding lines (e.g., when starting a new scene)
    pub fn clear_preceding_lines(&mut self) {
        self.preceding_lines.clear();
    }

    /// Get the next unit ID within this context
    pub fn next_unit_id(&mut self) -> usize {
        let id = self.unit_counter;
        self.unit_counter += 1;
        id
    }

    /// Generate a unique unit ID string
    pub fn generate_unit_id(&mut self, suffix: &str) -> String {
        let id = self.next_unit_id();
        if let Some(event_id) = self.event_id {
            format!("e{}.p{}.{}_{}", event_id, self.page_index, id, suffix)
        } else {
            format!("p{}.{}_{}", self.page_index, id, suffix)
        }
    }

    /// Convert current state to a TranslationContext
    pub fn to_translation_context(&self) -> TranslationContext {
        TranslationContext {
            file_name: Some(self.file_name.clone()),
            map_name: self.map_name.clone(),
            event_name: self.event_name.clone(),
            page_index: Some(self.page_index),
            preceding_lines: self.get_preceding_lines(),
            tags: Vec::new(),
        }
    }

    /// Clone for a new event
    pub fn for_event(&self, event_id: usize, event_name: Option<String>) -> Self {
        Self {
            file_name: self.file_name.clone(),
            map_name: self.map_name.clone(),
            event_name,
            event_id: Some(event_id),
            page_index: 0,
            current_speaker: None,
            preceding_lines: VecDeque::new(),
            max_preceding_lines: self.max_preceding_lines,
            unit_counter: 0,
        }
    }

    /// Clone for a new page within the same event
    pub fn for_page(&self, page_index: usize) -> Self {
        Self {
            file_name: self.file_name.clone(),
            map_name: self.map_name.clone(),
            event_name: self.event_name.clone(),
            event_id: self.event_id,
            page_index,
            current_speaker: None,
            preceding_lines: VecDeque::new(),
            max_preceding_lines: self.max_preceding_lines,
            unit_counter: 0,
        }
    }
}

impl Default for ExtractionContext {
    fn default() -> Self {
        Self::new("unknown")
    }
}

/// Result of extracting from a single command or command group
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Translation units extracted
    pub units: Vec<super::TranslationUnit>,
    /// Number of commands consumed (for skipping in the main loop)
    pub consumed: usize,
    /// Speaker name if this command updated it
    pub speaker_update: Option<Option<String>>,
    /// Dialogue text to add to preceding lines
    pub add_to_preceding: Option<String>,
}

impl ExtractionResult {
    /// Create an empty result (no extraction)
    pub fn empty() -> Self {
        Self {
            units: Vec::new(),
            consumed: 1,
            speaker_update: None,
            add_to_preceding: None,
        }
    }

    /// Create a result with consumed count
    pub fn skip(consumed: usize) -> Self {
        Self {
            units: Vec::new(),
            consumed,
            speaker_update: None,
            add_to_preceding: None,
        }
    }

    /// Create a result with a single unit
    pub fn single(unit: super::TranslationUnit, consumed: usize) -> Self {
        Self {
            units: vec![unit],
            consumed,
            speaker_update: None,
            add_to_preceding: None,
        }
    }

    /// Create a result with multiple units
    pub fn multiple(units: Vec<super::TranslationUnit>, consumed: usize) -> Self {
        Self {
            units,
            consumed,
            speaker_update: None,
            add_to_preceding: None,
        }
    }

    /// Set the speaker update
    pub fn with_speaker_update(mut self, speaker: Option<String>) -> Self {
        self.speaker_update = Some(speaker);
        self
    }

    /// Set text to add to preceding lines
    pub fn with_preceding(mut self, text: impl Into<String>) -> Self {
        self.add_to_preceding = Some(text.into());
        self
    }
}

/// Result of injecting translations back into JSON
#[derive(Debug, Clone, Default)]
pub struct InjectionResult {
    /// Number of translations successfully applied
    pub applied: usize,
    /// Number of translation IDs not found in the source
    pub not_found: usize,
    /// Number of commands modified
    pub commands_modified: usize,
    /// Any warnings encountered
    pub warnings: Vec<String>,
}

impl InjectionResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: InjectionResult) {
        self.applied += other.applied;
        self.not_found += other.not_found;
        self.commands_modified += other.commands_modified;
        self.warnings.extend(other.warnings);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_context_creation() {
        let ctx = ExtractionContext::new("CommonEvents.json")
            .with_event_id(5)
            .with_event_name("Test Event")
            .with_page_index(0);

        assert_eq!(ctx.file_name, "CommonEvents.json");
        assert_eq!(ctx.event_id, Some(5));
        assert_eq!(ctx.event_name, Some("Test Event".to_string()));
        assert_eq!(ctx.page_index, 0);
    }

    #[test]
    fn test_preceding_lines() {
        let mut ctx = ExtractionContext::new("test.json")
            .with_max_preceding_lines(3);

        ctx.add_preceding_line("Line 1");
        ctx.add_preceding_line("Line 2");
        ctx.add_preceding_line("Line 3");
        ctx.add_preceding_line("Line 4");

        let lines = ctx.get_preceding_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 2");
        assert_eq!(lines[2], "Line 4");
    }

    #[test]
    fn test_generate_unit_id() {
        let mut ctx = ExtractionContext::new("test.json")
            .with_event_id(3)
            .with_page_index(1);

        assert_eq!(ctx.generate_unit_id("dialogue"), "e3.p1.0_dialogue");
        assert_eq!(ctx.generate_unit_id("choice"), "e3.p1.1_choice");
    }

    #[test]
    fn test_for_event() {
        let ctx = ExtractionContext::new("Map001.json")
            .with_map_name("Forest");

        let event_ctx = ctx.for_event(5, Some("NPC Dialogue".to_string()));

        assert_eq!(event_ctx.file_name, "Map001.json");
        assert_eq!(event_ctx.map_name, Some("Forest".to_string()));
        assert_eq!(event_ctx.event_id, Some(5));
        assert_eq!(event_ctx.event_name, Some("NPC Dialogue".to_string()));
        assert_eq!(event_ctx.page_index, 0);
    }

    #[test]
    fn test_to_translation_context() {
        let mut ctx = ExtractionContext::new("test.json")
            .with_map_name("Town")
            .with_event_name("Shopkeeper");

        ctx.add_preceding_line("Hello!");
        ctx.set_speaker(Some("NPC".to_string()));

        let trans_ctx = ctx.to_translation_context();

        assert_eq!(trans_ctx.file_name, Some("test.json".to_string()));
        assert_eq!(trans_ctx.map_name, Some("Town".to_string()));
        assert_eq!(trans_ctx.event_name, Some("Shopkeeper".to_string()));
        assert_eq!(trans_ctx.preceding_lines, vec!["Hello!".to_string()]);
    }

    #[test]
    fn test_injection_result_merge() {
        let mut result1 = InjectionResult {
            applied: 5,
            not_found: 1,
            commands_modified: 10,
            warnings: vec!["Warning 1".to_string()],
        };

        let result2 = InjectionResult {
            applied: 3,
            not_found: 2,
            commands_modified: 5,
            warnings: vec!["Warning 2".to_string()],
        };

        result1.merge(result2);

        assert_eq!(result1.applied, 8);
        assert_eq!(result1.not_found, 3);
        assert_eq!(result1.commands_modified, 15);
        assert_eq!(result1.warnings.len(), 2);
    }
}
