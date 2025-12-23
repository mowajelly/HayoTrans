//! Translation unit types for extracted text
//!
//! A TranslationUnit represents a single piece of translatable text with all its metadata.

use super::{EventCode, TranslationPath};
use serde::{Deserialize, Serialize};

/// A single translatable text unit with its location and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    /// Unique identifier for this translation unit
    pub id: String,
    /// Location path in the JSON structure
    pub path: TranslationPath,
    /// Event command code that produced this text
    pub code: EventCode,
    /// Original text to translate
    pub original: String,
    /// Translated text (None if not yet translated)
    pub translated: Option<String>,
    /// Speaker name (from code 101)
    pub speaker: Option<String>,
    /// Additional context for translation
    pub context: TranslationContext,
    /// Translation status
    pub status: TranslationStatus,
}

impl TranslationUnit {
    /// Create a new translation unit
    pub fn new(
        id: String,
        path: TranslationPath,
        code: EventCode,
        original: String,
    ) -> Self {
        Self {
            id,
            path,
            code,
            original,
            translated: None,
            speaker: None,
            context: TranslationContext::default(),
            status: TranslationStatus::Pending,
        }
    }

    /// Set the speaker
    pub fn with_speaker(mut self, speaker: Option<String>) -> Self {
        self.speaker = speaker;
        self
    }

    /// Set the context
    pub fn with_context(mut self, context: TranslationContext) -> Self {
        self.context = context;
        self
    }

    /// Set the translation
    pub fn with_translation(mut self, translation: String) -> Self {
        self.translated = Some(translation);
        self.status = TranslationStatus::Translated;
        self
    }

    /// Check if this unit has been translated
    pub fn is_translated(&self) -> bool {
        self.translated.is_some()
    }

    /// Check if the original text is empty or whitespace
    pub fn is_empty(&self) -> bool {
        self.original.trim().is_empty()
    }

    /// Get the effective text (translated if available, otherwise original)
    pub fn effective_text(&self) -> &str {
        self.translated.as_deref().unwrap_or(&self.original)
    }

    /// Check if the text contains translatable characters (CJK)
    pub fn needs_translation(&self) -> bool {
        self.original.chars().any(|c| {
            // Japanese (Hiragana, Katakana, Kanji)
            ('\u{3040}'..='\u{309F}').contains(&c)  // Hiragana
            || ('\u{30A0}'..='\u{30FF}').contains(&c)  // Katakana
            // Korean (Hangul)
            || ('\u{AC00}'..='\u{D7AF}').contains(&c)  // Hangul syllables
            // Chinese (CJK Unified Ideographs)
            || ('\u{4E00}'..='\u{9FFF}').contains(&c)
        })
    }
}

/// Context information for translation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationContext {
    /// Source file name
    pub file_name: Option<String>,
    /// Map name (for map files)
    pub map_name: Option<String>,
    /// Event name
    pub event_name: Option<String>,
    /// Page index within the event
    pub page_index: Option<usize>,
    /// Preceding dialogue lines for context
    pub preceding_lines: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl TranslationContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the file name
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
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

    /// Set the page index
    pub fn with_page_index(mut self, page_index: usize) -> Self {
        self.page_index = Some(page_index);
        self
    }

    /// Add a preceding line
    pub fn add_preceding_line(&mut self, line: impl Into<String>) {
        self.preceding_lines.push(line.into());
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }
}

/// Status of a translation unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStatus {
    /// Not yet translated
    #[default]
    Pending,
    /// Machine translated, needs review
    Translated,
    /// Reviewed and approved
    Reviewed,
    /// Needs revision
    NeedsRevision,
    /// Skipped (not translatable or intentionally skipped)
    Skipped,
}

impl TranslationStatus {
    /// Check if this status means the unit is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Reviewed | Self::Skipped)
    }

    /// Check if this status needs attention
    pub fn needs_attention(&self) -> bool {
        matches!(self, Self::Pending | Self::NeedsRevision)
    }
}

/// Collection of translation units from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    /// Version of the translation file format
    pub version: String,
    /// Source file path
    pub source_file: String,
    /// When the extraction was performed
    pub extracted_at: String,
    /// All translation units
    pub units: Vec<TranslationUnit>,
    /// File metadata
    pub metadata: TranslationFileMetadata,
}

impl TranslationFile {
    /// Create a new translation file
    pub fn new(source_file: impl Into<String>) -> Self {
        Self {
            version: "1.0".to_string(),
            source_file: source_file.into(),
            extracted_at: chrono::Utc::now().to_rfc3339(),
            units: Vec::new(),
            metadata: TranslationFileMetadata::default(),
        }
    }

    /// Add a translation unit
    pub fn add_unit(&mut self, unit: TranslationUnit) {
        self.units.push(unit);
        self.update_metadata();
    }

    /// Add multiple translation units
    pub fn add_units(&mut self, units: impl IntoIterator<Item = TranslationUnit>) {
        self.units.extend(units);
        self.update_metadata();
    }

    /// Update metadata based on current units
    fn update_metadata(&mut self) {
        self.metadata.total_units = self.units.len();
        self.metadata.translated = self.units.iter().filter(|u| u.is_translated()).count();
        self.metadata.reviewed = self
            .units
            .iter()
            .filter(|u| u.status == TranslationStatus::Reviewed)
            .count();

        // Collect unique speakers
        let speakers: std::collections::HashSet<_> = self
            .units
            .iter()
            .filter_map(|u| u.speaker.as_ref())
            .cloned()
            .collect();
        self.metadata.speakers = speakers.into_iter().collect();
    }

    /// Get units by status
    pub fn units_by_status(&self, status: TranslationStatus) -> Vec<&TranslationUnit> {
        self.units.iter().filter(|u| u.status == status).collect()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.units.is_empty() {
            100.0
        } else {
            let complete = self
                .units
                .iter()
                .filter(|u| u.status.is_complete() || u.is_translated())
                .count();
            (complete as f64 / self.units.len() as f64) * 100.0
        }
    }
}

/// Metadata for a translation file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationFileMetadata {
    /// Total number of translation units
    pub total_units: usize,
    /// Number of translated units
    pub translated: usize,
    /// Number of reviewed units
    pub reviewed: usize,
    /// List of unique speakers found
    pub speakers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_unit_creation() {
        let unit = TranslationUnit::new(
            "test_1".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Hello World".to_string(),
        );

        assert_eq!(unit.id, "test_1");
        assert_eq!(unit.original, "Hello World");
        assert!(!unit.is_translated());
        assert_eq!(unit.status, TranslationStatus::Pending);
    }

    #[test]
    fn test_translation_unit_with_translation() {
        let unit = TranslationUnit::new(
            "test_1".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "こんにちは".to_string(),
        )
        .with_translation("Hello".to_string());

        assert!(unit.is_translated());
        assert_eq!(unit.effective_text(), "Hello");
        assert_eq!(unit.status, TranslationStatus::Translated);
    }

    #[test]
    fn test_needs_translation() {
        let japanese = TranslationUnit::new(
            "1".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "こんにちは".to_string(),
        );
        assert!(japanese.needs_translation());

        let english = TranslationUnit::new(
            "2".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Hello".to_string(),
        );
        assert!(!english.needs_translation());
    }

    #[test]
    fn test_translation_file() {
        let mut file = TranslationFile::new("CommonEvents.json");

        file.add_unit(TranslationUnit::new(
            "1".to_string(),
            TranslationPath::new(),
            EventCode::ShowTextBody,
            "Text 1".to_string(),
        ));

        file.add_unit(
            TranslationUnit::new(
                "2".to_string(),
                TranslationPath::new(),
                EventCode::ShowTextBody,
                "Text 2".to_string(),
            )
            .with_speaker(Some("NPC".to_string()))
            .with_translation("Translated 2".to_string()),
        );

        assert_eq!(file.metadata.total_units, 2);
        assert_eq!(file.metadata.translated, 1);
        assert_eq!(file.metadata.speakers, vec!["NPC".to_string()]);
        assert_eq!(file.completion_percentage(), 50.0);
    }

    #[test]
    fn test_translation_status() {
        assert!(TranslationStatus::Reviewed.is_complete());
        assert!(TranslationStatus::Skipped.is_complete());
        assert!(!TranslationStatus::Pending.is_complete());

        assert!(TranslationStatus::Pending.needs_attention());
        assert!(TranslationStatus::NeedsRevision.needs_attention());
        assert!(!TranslationStatus::Reviewed.needs_attention());
    }
}
