//! Translation path for navigating JSON structures
//!
//! TranslationPath provides a way to locate and modify values in nested JSON structures.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// A segment in a translation path
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathSegment {
    /// Object key access (e.g., "events")
    Key(String),
    /// Array index access (e.g., [5])
    Index(usize),
}

impl PathSegment {
    /// Create a key segment
    pub fn key(k: impl Into<String>) -> Self {
        Self::Key(k.into())
    }

    /// Create an index segment
    pub fn index(i: usize) -> Self {
        Self::Index(i)
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(k) => write!(f, "{}", k),
            Self::Index(i) => write!(f, "{}", i),
        }
    }
}

/// A structured path that can locate any value in JSON
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TranslationPath {
    segments: Vec<PathSegment>,
}

impl TranslationPath {
    /// Create an empty path (root)
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a path from segments
    pub fn from_segments(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }

    /// Parse a path from a string
    /// Format: "events.5.pages.0.list.12.parameters.0"
    pub fn parse(s: &str) -> Result<Self, PathParseError> {
        if s.is_empty() {
            return Ok(Self::new());
        }

        let segments: Result<Vec<PathSegment>, PathParseError> = s
            .split('.')
            .map(|part| {
                if part.is_empty() {
                    Err(PathParseError::EmptySegment)
                } else if let Ok(index) = part.parse::<usize>() {
                    Ok(PathSegment::Index(index))
                } else {
                    Ok(PathSegment::Key(part.to_string()))
                }
            })
            .collect();

        Ok(Self {
            segments: segments?,
        })
    }

    /// Get the number of segments in the path
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if the path is empty (root)
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the segments
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Append a key segment
    pub fn append_key(&self, key: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Key(key.into()));
        Self { segments }
    }

    /// Append an index segment
    pub fn append_index(&self, index: usize) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Index(index));
        Self { segments }
    }

    /// Append another path
    pub fn append(&self, other: &TranslationPath) -> Self {
        let mut segments = self.segments.clone();
        segments.extend(other.segments.iter().cloned());
        Self { segments }
    }

    /// Get the parent path (all segments except the last)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            None
        } else {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }

    /// Get the last segment
    pub fn last(&self) -> Option<&PathSegment> {
        self.segments.last()
    }

    /// Get a value at this path from JSON
    pub fn get<'a>(&self, json: &'a Value) -> Option<&'a Value> {
        let mut current = json;
        for segment in &self.segments {
            current = match segment {
                PathSegment::Key(key) => current.get(key)?,
                PathSegment::Index(idx) => current.get(*idx)?,
            };
        }
        Some(current)
    }

    /// Get a mutable value at this path from JSON
    pub fn get_mut<'a>(&self, json: &'a mut Value) -> Option<&'a mut Value> {
        let mut current = json;
        for segment in &self.segments {
            current = match segment {
                PathSegment::Key(key) => current.get_mut(key)?,
                PathSegment::Index(idx) => current.get_mut(*idx)?,
            };
        }
        Some(current)
    }

    /// Set a value at this path in JSON
    pub fn set(&self, json: &mut Value, value: Value) -> Result<(), PathSetError> {
        if self.segments.is_empty() {
            *json = value;
            return Ok(());
        }

        // Navigate to parent
        let parent_path = self.parent().ok_or(PathSetError::InvalidPath)?;
        let parent = parent_path
            .get_mut(json)
            .ok_or(PathSetError::ParentNotFound)?;

        // Set the value at the last segment
        match self.last().ok_or(PathSetError::InvalidPath)? {
            PathSegment::Key(key) => {
                if let Some(obj) = parent.as_object_mut() {
                    obj.insert(key.clone(), value);
                    Ok(())
                } else {
                    Err(PathSetError::NotAnObject)
                }
            }
            PathSegment::Index(idx) => {
                if let Some(arr) = parent.as_array_mut() {
                    if *idx < arr.len() {
                        arr[*idx] = value;
                        Ok(())
                    } else {
                        Err(PathSetError::IndexOutOfBounds(*idx))
                    }
                } else {
                    Err(PathSetError::NotAnArray)
                }
            }
        }
    }

    /// Convert to string representation
    pub fn to_path_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Create a unique ID for translation units based on this path
    pub fn to_unit_id(&self, suffix: &str) -> String {
        let path_str = self.to_path_string();
        if suffix.is_empty() {
            path_str
        } else {
            format!("{}_{}", path_str, suffix)
        }
    }
}

impl fmt::Display for TranslationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path_string())
    }
}

/// Error parsing a path string
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathParseError {
    #[error("Empty segment in path")]
    EmptySegment,
}

/// Error setting a value at a path
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathSetError {
    #[error("Invalid path")]
    InvalidPath,
    #[error("Parent path not found")]
    ParentNotFound,
    #[error("Path target is not an object")]
    NotAnObject,
    #[error("Path target is not an array")]
    NotAnArray,
    #[error("Array index {0} out of bounds")]
    IndexOutOfBounds(usize),
}

/// A pattern for matching paths (supports wildcards)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPattern {
    /// Original pattern string
    pattern: String,
    /// Compiled regex for matching
    #[serde(skip)]
    regex: Option<regex::Regex>,
}

impl PathPattern {
    /// Create a new path pattern
    /// Supports |ARY| for array indices and |OBJ| for object keys
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let regex = Self::compile_pattern(&pattern);
        Self { pattern, regex }
    }

    /// Compile pattern to regex
    fn compile_pattern(pattern: &str) -> Option<regex::Regex> {
        let regex_str = format!(
            "^{}$",
            regex::escape(pattern)
                .replace(r"\|ARY\|", r"\d+")
                .replace(r"\|OBJ\|", r"\w+")
        );
        regex::Regex::new(&regex_str).ok()
    }

    /// Check if a concrete path matches this pattern
    pub fn matches(&self, path: &str) -> bool {
        match &self.regex {
            Some(re) => re.is_match(path),
            None => self.pattern == path,
        }
    }

    /// Check if a TranslationPath matches this pattern
    pub fn matches_path(&self, path: &TranslationPath) -> bool {
        self.matches(&path.to_path_string())
    }

    /// Get the pattern string
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl PartialEq for PathPattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for PathPattern {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_path_parse() {
        let path = TranslationPath::parse("events.5.pages.0.list.12").unwrap();
        assert_eq!(path.len(), 6);
        assert_eq!(path.segments[0], PathSegment::Key("events".to_string()));
        assert_eq!(path.segments[1], PathSegment::Index(5));
        assert_eq!(path.segments[2], PathSegment::Key("pages".to_string()));
        assert_eq!(path.segments[3], PathSegment::Index(0));
    }

    #[test]
    fn test_path_empty() {
        let path = TranslationPath::parse("").unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn test_path_append() {
        let path = TranslationPath::new();
        let path = path.append_key("events");
        let path = path.append_index(5);
        assert_eq!(path.to_path_string(), "events.5");
    }

    #[test]
    fn test_path_get() {
        let json = json!({
            "events": [
                null,
                {"name": "Event 1", "pages": [{"list": [{"code": 401}]}]}
            ]
        });
        
        let path = TranslationPath::parse("events.1.name").unwrap();
        assert_eq!(path.get(&json), Some(&json!("Event 1")));
        
        let path = TranslationPath::parse("events.1.pages.0.list.0.code").unwrap();
        assert_eq!(path.get(&json), Some(&json!(401)));
        
        let path = TranslationPath::parse("events.99").unwrap();
        assert_eq!(path.get(&json), None);
    }

    #[test]
    fn test_path_set() {
        let mut json = json!({
            "events": [
                {"name": "Old Name"}
            ]
        });
        
        let path = TranslationPath::parse("events.0.name").unwrap();
        path.set(&mut json, json!("New Name")).unwrap();
        
        assert_eq!(json["events"][0]["name"], "New Name");
    }

    #[test]
    fn test_path_pattern() {
        let pattern = PathPattern::new("events.|ARY|.pages.|ARY|.list.|ARY|.parameters.|ARY|");
        assert!(pattern.matches("events.5.pages.0.list.12.parameters.0"));
        assert!(pattern.matches("events.0.pages.2.list.100.parameters.3"));
        assert!(!pattern.matches("events.5.pages.0.list.12"));
        assert!(!pattern.matches("other.5.pages.0.list.12.parameters.0"));
    }

    #[test]
    fn test_path_pattern_obj() {
        let pattern = PathPattern::new("plugins.|OBJ|.parameters");
        assert!(pattern.matches("plugins.QuestSystem.parameters"));
        assert!(pattern.matches("plugins.NUUN_EnemyBook.parameters"));
        assert!(!pattern.matches("plugins.parameters"));
    }

    #[test]
    fn test_to_unit_id() {
        let path = TranslationPath::parse("events.1.pages.0.list.5").unwrap();
        assert_eq!(path.to_unit_id("dialogue"), "events.1.pages.0.list.5_dialogue");
        assert_eq!(path.to_unit_id(""), "events.1.pages.0.list.5");
    }
}
