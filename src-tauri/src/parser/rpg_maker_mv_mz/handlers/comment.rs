//! Comment handler for Comment Body (408) commands

use super::{generate_unit_id, CommandHandler};
use crate::parser::rpg_maker_mv_mz::command::EventCommand;
use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, ExtractionResult, InjectionOptions,
    InjectionResult, TranslationPath, TranslationUnit,
};
use serde_json::Value;
use std::collections::HashMap;

/// Handler for Comment Body command (408)
/// This handles comment text that might need translation
#[derive(Debug, Clone)]
pub struct CommentHandler;

impl CommandHandler for CommentHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::CommentBody]
    }

    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> ExtractionResult {
        // Check if comment extraction is enabled
        if !options.extract_comments {
            return ExtractionResult::empty();
        }

        let cmd = &commands[index];

        // Get comment text
        let comment_text = match cmd.get_comment_text() {
            Some(t) => t,
            None => return ExtractionResult::empty(),
        };

        // Skip comments with specific prefixes
        if options.should_skip_comment(comment_text) {
            return ExtractionResult::empty();
        }

        // Skip empty comments
        if comment_text.trim().is_empty() && !options.include_empty {
            return ExtractionResult::empty();
        }

        let text = if options.trim_whitespace {
            comment_text.trim().to_string()
        } else {
            comment_text.to_string()
        };

        let unit_id = generate_unit_id(path_prefix, index, "comment");

        // Add a tag to indicate this is a comment
        let mut trans_context = context.to_translation_context();
        trans_context.add_tag("comment".to_string());

        let unit = TranslationUnit::new(
            unit_id,
            path_prefix.append_index(index),
            EventCode::CommentBody,
            text,
        )
        .with_context(trans_context);

        ExtractionResult::single(unit, 1)
    }

    fn inject(
        &self,
        commands: &mut Vec<EventCommand>,
        index: usize,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        _context: &ExtractionContext,
        _options: &InjectionOptions,
    ) -> InjectionResult {
        let mut result = InjectionResult::new();
        let unit_id = generate_unit_id(path_prefix, index, "comment");

        if let Some(translated) = translations.get(&unit_id) {
            let cmd = &mut commands[index];
            if !cmd.parameters.is_empty() {
                cmd.parameters[0] = Value::String(translated.clone());
                result.applied += 1;
                result.commands_modified += 1;
            }
        }

        result
    }
}

/// Handler for Script Special Text command (657)
/// This handles special script text patterns like "テキスト = ..."
#[derive(Debug, Clone)]
pub struct ScriptTextHandler {
    /// Prefix pattern to match
    prefix: String,
}

impl ScriptTextHandler {
    /// Create a new script text handler with the given prefix
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Create with the default Japanese prefix
    pub fn default_prefix() -> Self {
        Self::new("テキスト = ")
    }
}

impl CommandHandler for ScriptTextHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::ScriptBodyAlt]
    }

    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> ExtractionResult {
        // Check if script text extraction is enabled
        if !options.extract_script_text {
            return ExtractionResult::empty();
        }

        let cmd = &commands[index];

        // Try to extract script special text with configured prefix
        let prefix = options.script_text_prefix.as_ref()
            .map(|s| s.as_str())
            .unwrap_or(&self.prefix);

        let text = match cmd.get_script_special_text(prefix) {
            Some(t) => t,
            None => return ExtractionResult::empty(),
        };

        // Skip empty text
        if text.trim().is_empty() && !options.include_empty {
            return ExtractionResult::empty();
        }

        let processed_text = if options.trim_whitespace {
            text.trim().to_string()
        } else {
            text
        };

        let unit_id = generate_unit_id(path_prefix, index, "script_text");

        // Add a tag to indicate this is script text
        let mut trans_context = context.to_translation_context();
        trans_context.add_tag("script_text".to_string());

        let unit = TranslationUnit::new(
            unit_id,
            path_prefix.append_index(index),
            EventCode::ScriptBodyAlt,
            processed_text,
        )
        .with_context(trans_context);

        ExtractionResult::single(unit, 1)
    }

    fn inject(
        &self,
        commands: &mut Vec<EventCommand>,
        index: usize,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        _context: &ExtractionContext,
        options: &InjectionOptions,
    ) -> InjectionResult {
        let mut result = InjectionResult::new();
        let unit_id = generate_unit_id(path_prefix, index, "script_text");

        if let Some(translated) = translations.get(&unit_id) {
            let cmd = &mut commands[index];
            if !cmd.parameters.is_empty() {
                // Reconstruct the script text with the prefix
                let prefix = match &options {
                    _ => "テキスト = ", // Use default prefix for injection
                };
                let new_text = format!("{}{}", prefix, translated);
                cmd.parameters[0] = Value::String(new_text);
                result.applied += 1;
                result.commands_modified += 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_comment(text: &str) -> EventCommand {
        EventCommand {
            code: 408,
            indent: 0,
            parameters: vec![json!(text)],
        }
    }

    fn make_script_text(text: &str) -> EventCommand {
        EventCommand {
            code: 657,
            indent: 0,
            parameters: vec![json!(format!("テキスト = {}", text))],
        }
    }

    #[test]
    fn test_comment_extraction() {
        let handler = CommentHandler;
        let commands = vec![make_comment("これはコメントです")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "これはコメントです");
        assert!(result.units[0].context.tags.contains(&"comment".to_string()));
    }

    #[test]
    fn test_comment_skip_semicolon() {
        let handler = CommentHandler;
        let commands = vec![make_comment("; This is a code comment")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn test_comment_extraction_disabled() {
        let handler = CommentHandler;
        let commands = vec![make_comment("これはコメントです")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let mut options = ExtractionOptions::default();
        options.extract_comments = false;

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn test_comment_injection() {
        let handler = CommentHandler;
        let mut commands = vec![make_comment("これはコメントです")];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert("0_comment".to_string(), "This is a comment".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        assert_eq!(commands[0].get_comment_text(), Some("This is a comment"));
    }

    #[test]
    fn test_script_text_extraction() {
        let handler = ScriptTextHandler::default_prefix();
        let commands = vec![make_script_text("特別なテキスト")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "特別なテキスト");
        assert!(result.units[0].context.tags.contains(&"script_text".to_string()));
    }

    #[test]
    fn test_script_text_no_match() {
        let handler = ScriptTextHandler::default_prefix();
        let commands = vec![EventCommand {
            code: 657,
            indent: 0,
            parameters: vec![json!("some_variable = 100")],
        }];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn test_script_text_injection() {
        let handler = ScriptTextHandler::default_prefix();
        let mut commands = vec![make_script_text("特別なテキスト")];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert("0_script_text".to_string(), "Special Text".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        let text = commands[0].get_string_param(0).unwrap();
        assert!(text.starts_with("テキスト = "));
        assert!(text.ends_with("Special Text"));
    }
}
