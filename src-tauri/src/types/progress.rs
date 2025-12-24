//! Project progress types and states

use serde::{Deserialize, Serialize};

/// Project progress state
/// Represents the current stage of the translation workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProgressState {
    /// Initial state - project just added
    #[default]
    Initial,
    /// Assets have been unpacked (for encrypted games)
    AssetsUnpacked,
    /// Dialogues and text have been extracted
    DialoguesExtracted,
    /// Metadata translated (actor names, terms, etc.)
    MetadataTranslated,
    /// Main dialogues translated
    DialoguesTranslated,
    /// Assets repacked (for encrypted games)
    AssetsRepacked,
    /// Final version merged and ready
    Finalized,
}

impl ProgressState {
    /// Get all states in order
    pub fn all_states() -> Vec<Self> {
        vec![
            Self::Initial,
            Self::AssetsUnpacked,
            Self::DialoguesExtracted,
            Self::MetadataTranslated,
            Self::DialoguesTranslated,
            Self::AssetsRepacked,
            Self::Finalized,
        ]
    }

    /// Get the index of this state in the workflow
    pub fn index(&self) -> usize {
        match self {
            Self::Initial => 0,
            Self::AssetsUnpacked => 1,
            Self::DialoguesExtracted => 2,
            Self::MetadataTranslated => 3,
            Self::DialoguesTranslated => 4,
            Self::AssetsRepacked => 5,
            Self::Finalized => 6,
        }
    }

    /// Get the total number of states
    pub fn total_states() -> usize {
        7
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn progress_percentage(&self) -> f32 {
        self.index() as f32 / (Self::total_states() - 1) as f32
    }

    /// Check if this state is completed (past a certain state)
    pub fn is_past(&self, other: &Self) -> bool {
        self.index() > other.index()
    }

    /// Check if this state is at or past a certain state
    pub fn is_at_or_past(&self, other: &Self) -> bool {
        self.index() >= other.index()
    }

    /// Get the next recommended state
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Initial => Some(Self::AssetsUnpacked),
            Self::AssetsUnpacked => Some(Self::DialoguesExtracted),
            Self::DialoguesExtracted => Some(Self::MetadataTranslated),
            Self::MetadataTranslated => Some(Self::DialoguesTranslated),
            Self::DialoguesTranslated => Some(Self::AssetsRepacked),
            Self::AssetsRepacked => Some(Self::Finalized),
            Self::Finalized => None,
        }
    }

    /// Get the database string representation
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::AssetsUnpacked => "assets_unpacked",
            Self::DialoguesExtracted => "dialogues_extracted",
            Self::MetadataTranslated => "metadata_translated",
            Self::DialoguesTranslated => "dialogues_translated",
            Self::AssetsRepacked => "assets_repacked",
            Self::Finalized => "finalized",
        }
    }

    /// Parse from database string
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "initial" => Self::Initial,
            "assets_unpacked" => Self::AssetsUnpacked,
            "dialogues_extracted" => Self::DialoguesExtracted,
            "metadata_translated" => Self::MetadataTranslated,
            "dialogues_translated" => Self::DialoguesTranslated,
            "assets_repacked" => Self::AssetsRepacked,
            "finalized" => Self::Finalized,
            _ => Self::Initial,
        }
    }
}

impl std::fmt::Display for ProgressState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

impl From<String> for ProgressState {
    fn from(s: String) -> Self {
        Self::from_db_str(&s)
    }
}

impl From<&str> for ProgressState {
    fn from(s: &str) -> Self {
        Self::from_db_str(s)
    }
}

/// Log level for project logs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Log type for categorizing log entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    /// General system log
    System,
    /// File operation (read/write/extract)
    FileOperation,
    /// Translation operation
    Translation,
    /// Archive operation (pack/unpack)
    Archive,
    /// Parsing operation
    Parsing,
    /// User action
    UserAction,
}

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::FileOperation => "file_operation",
            Self::Translation => "translation",
            Self::Archive => "archive",
            Self::Parsing => "parsing",
            Self::UserAction => "user_action",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_state_order() {
        assert!(ProgressState::DialoguesExtracted.is_past(&ProgressState::Initial));
        assert!(ProgressState::Finalized.is_at_or_past(&ProgressState::Finalized));
        assert!(!ProgressState::Initial.is_past(&ProgressState::AssetsUnpacked));
    }

    #[test]
    fn test_progress_state_next() {
        assert_eq!(ProgressState::Initial.next(), Some(ProgressState::AssetsUnpacked));
        assert_eq!(ProgressState::Finalized.next(), None);
    }

    #[test]
    fn test_progress_percentage() {
        assert_eq!(ProgressState::Initial.progress_percentage(), 0.0);
        assert_eq!(ProgressState::Finalized.progress_percentage(), 1.0);
    }

    #[test]
    fn test_db_str_roundtrip() {
        for state in ProgressState::all_states() {
            let s = state.as_db_str();
            let parsed = ProgressState::from_db_str(s);
            assert_eq!(state, parsed);
        }
    }
}
