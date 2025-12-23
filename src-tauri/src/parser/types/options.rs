//! Options for extraction and injection operations
//!
//! These options allow users to customize how text is extracted and injected.

use serde::{Deserialize, Serialize};

/// Options for text extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionOptions {
    /// Whether to trim whitespace from extracted text
    pub trim_whitespace: bool,
    /// Whether to merge consecutive dialogue lines (401 blocks)
    pub merge_dialogue_lines: bool,
    /// Separator to use when merging dialogue lines
    pub dialogue_line_separator: String,
    /// Whether to extract comment blocks (408)
    pub extract_comments: bool,
    /// Whether to skip comments starting with specific prefixes
    pub skip_comment_prefixes: Vec<String>,
    /// Whether to include empty text blocks
    pub include_empty: bool,
    /// Maximum preceding lines to keep for context
    pub max_preceding_lines: usize,
    /// Whether to extract plugin command text (357)
    pub extract_plugins: bool,
    /// Whether to extract script special text (657)
    pub extract_script_text: bool,
    /// Script text prefix pattern to match (e.g., "テキスト = ")
    pub script_text_prefix: Option<String>,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            trim_whitespace: false,
            merge_dialogue_lines: true,
            dialogue_line_separator: "\n".to_string(),
            extract_comments: true,
            skip_comment_prefixes: vec![";".to_string()],
            include_empty: false,
            max_preceding_lines: 5,
            extract_plugins: true,
            extract_script_text: true,
            script_text_prefix: Some("テキスト = ".to_string()),
        }
    }
}

impl ExtractionOptions {
    /// Create options with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options optimized for machine translation
    pub fn for_machine_translation() -> Self {
        Self {
            trim_whitespace: true,
            merge_dialogue_lines: true,
            dialogue_line_separator: " ".to_string(),
            extract_comments: false,
            skip_comment_prefixes: vec![";".to_string()],
            include_empty: false,
            max_preceding_lines: 3,
            extract_plugins: true,
            extract_script_text: true,
            script_text_prefix: Some("テキスト = ".to_string()),
        }
    }

    /// Should this comment be skipped?
    pub fn should_skip_comment(&self, text: &str) -> bool {
        self.skip_comment_prefixes
            .iter()
            .any(|prefix| text.starts_with(prefix))
    }
}

/// Options for text injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionOptions {
    /// Maximum line length for dialogue (split into multiple 401 commands)
    pub max_line_length: Option<usize>,
    /// Whether to use word-aware line splitting
    pub word_aware_split: bool,
    /// Whether to preserve original line breaks in translation
    pub preserve_line_breaks: bool,
    /// Whether to backup the original file before injection
    pub create_backup: bool,
    /// Whether to validate translations before injection
    pub validate_before_inject: bool,
    /// Whether to update commands that are missing translations
    pub skip_missing_translations: bool,
}

impl Default for InjectionOptions {
    fn default() -> Self {
        Self {
            max_line_length: None,
            word_aware_split: true,
            preserve_line_breaks: true,
            create_backup: true,
            validate_before_inject: true,
            skip_missing_translations: true,
        }
    }
}

impl InjectionOptions {
    /// Create options with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options with a specific max line length
    pub fn with_max_line_length(mut self, length: usize) -> Self {
        self.max_line_length = Some(length);
        self
    }

    /// Split text into lines respecting max length
    pub fn split_text(&self, text: &str) -> Vec<String> {
        match self.max_line_length {
            Some(max_len) if max_len > 0 => {
                if self.preserve_line_breaks {
                    // Split on existing line breaks first, then on max length
                    text.lines()
                        .flat_map(|line| self.split_line(line, max_len))
                        .collect()
                } else {
                    // Treat as single text and split only on max length
                    let text = text.replace('\n', " ");
                    self.split_line(&text, max_len)
                }
            }
            _ => {
                // No max length, just split on line breaks
                text.lines().map(|s| s.to_string()).collect()
            }
        }
    }

    /// Split a single line into chunks respecting max length
    fn split_line(&self, line: &str, max_len: usize) -> Vec<String> {
        if line.len() <= max_len {
            return vec![line.to_string()];
        }

        if self.word_aware_split {
            self.split_at_words(line, max_len)
        } else {
            self.split_at_chars(line, max_len)
        }
    }

    /// Split text at word boundaries
    fn split_at_words(&self, text: &str, max_len: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                if word.len() > max_len {
                    // Word itself is too long, need to split it
                    result.extend(self.split_at_chars(word, max_len));
                } else {
                    current_line = word.to_string();
                }
            } else if current_line.len() + 1 + word.len() <= max_len {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                result.push(current_line);
                if word.len() > max_len {
                    result.extend(self.split_at_chars(word, max_len));
                    current_line = String::new();
                } else {
                    current_line = word.to_string();
                }
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }

        result
    }

    /// Split text at character boundaries (for CJK or when word split fails)
    fn split_at_chars(&self, text: &str, max_len: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut current_len = 0;

        for c in text.chars() {
            // For CJK characters, count as 2 for width estimation
            let char_width = if c.is_ascii() { 1 } else { 2 };

            if current_len + char_width > max_len && !current.is_empty() {
                result.push(current);
                current = String::new();
                current_len = 0;
            }

            current.push(c);
            current_len += char_width;
        }

        if !current.is_empty() {
            result.push(current);
        }

        result
    }
}

/// Combined options for parsing operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParserOptions {
    /// Extraction options
    pub extraction: ExtractionOptions,
    /// Injection options
    pub injection: InjectionOptions,
}

impl ParserOptions {
    /// Create with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom extraction options
    pub fn with_extraction(mut self, options: ExtractionOptions) -> Self {
        self.extraction = options;
        self
    }

    /// Create with custom injection options
    pub fn with_injection(mut self, options: InjectionOptions) -> Self {
        self.injection = options;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_options_default() {
        let opts = ExtractionOptions::default();
        assert!(opts.merge_dialogue_lines);
        assert!(opts.extract_comments);
        assert!(!opts.include_empty);
    }

    #[test]
    fn test_should_skip_comment() {
        let opts = ExtractionOptions::default();
        assert!(opts.should_skip_comment("; This is a comment"));
        assert!(!opts.should_skip_comment("This is not a comment"));
    }

    #[test]
    fn test_injection_split_no_limit() {
        let opts = InjectionOptions::default();
        let result = opts.split_text("Line 1\nLine 2\nLine 3");
        assert_eq!(result, vec!["Line 1", "Line 2", "Line 3"]);
    }

    #[test]
    fn test_injection_split_with_limit() {
        let opts = InjectionOptions::default().with_max_line_length(20);
        let result = opts.split_text("This is a very long line that needs splitting");
        assert!(result.iter().all(|line| line.len() <= 20));
        assert!(result.len() > 1);
    }

    #[test]
    fn test_injection_split_cjk() {
        let opts = InjectionOptions {
            max_line_length: Some(10),
            word_aware_split: false,
            ..Default::default()
        };
        let result = opts.split_text("これは日本語のテキストです");
        assert!(result.len() > 1);
    }

    #[test]
    fn test_injection_split_preserve_breaks() {
        let opts = InjectionOptions {
            max_line_length: Some(50),
            preserve_line_breaks: true,
            ..Default::default()
        };
        let result = opts.split_text("Line 1\nLine 2");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_injection_split_no_preserve_breaks() {
        let opts = InjectionOptions {
            max_line_length: Some(50),
            preserve_line_breaks: false,
            ..Default::default()
        };
        let result = opts.split_text("Line 1\nLine 2");
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("Line 1"));
        assert!(result[0].contains("Line 2"));
    }
}
