//! Dialogue handlers for Show Text (101) and Text Body (401) commands

use super::{generate_unit_id, CommandHandler};
use crate::parser::rpg_maker_mv_mz::command::EventCommand;
use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, ExtractionResult, InjectionOptions,
    InjectionResult, TranslationPath, TranslationUnit,
};
use std::collections::HashMap;

/// Handler for Show Text command (101)
/// This command sets up the message window and speaker name
#[derive(Debug, Clone)]
pub struct ShowTextHandler;

impl CommandHandler for ShowTextHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::ShowText]
    }

    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        _path_prefix: &TranslationPath,
        _context: &mut ExtractionContext,
        _options: &ExtractionOptions,
    ) -> ExtractionResult {
        let cmd = &commands[index];
        
        // Extract and update speaker name
        let speaker = cmd.get_speaker_name();
        
        ExtractionResult::empty().with_speaker_update(speaker)
    }

    fn inject(
        &self,
        _commands: &mut Vec<EventCommand>,
        _index: usize,
        _translations: &HashMap<String, String>,
        _path_prefix: &TranslationPath,
        _context: &ExtractionContext,
        _options: &InjectionOptions,
    ) -> InjectionResult {
        // ShowText (101) itself doesn't usually need translation for speaker names
        // in the basic case. Actor name translation would be handled separately.
        InjectionResult::new()
    }
}

/// Handler for Text Body command (401)
/// This handles the actual dialogue text, including consecutive blocks
#[derive(Debug, Clone)]
pub struct DialogueHandler;

impl CommandHandler for DialogueHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::ShowTextBody]
    }

    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> ExtractionResult {
        let cmd = &commands[index];
        let indent = cmd.indent;

        // Collect consecutive 401 commands with the same indent
        let mut lines = Vec::new();
        let mut consumed = 0;

        while index + consumed < commands.len() {
            let current = &commands[index + consumed];
            
            // Stop if code is not 401 or indent differs
            if current.code != 401 || current.indent != indent {
                break;
            }

            if let Some(text) = current.get_dialogue_text() {
                let text = if options.trim_whitespace {
                    text.trim().to_string()
                } else {
                    text.to_string()
                };
                lines.push(text);
            }
            consumed += 1;
        }

        // Skip if no lines or all empty
        if lines.is_empty() || (lines.iter().all(|l| l.trim().is_empty()) && !options.include_empty) {
            return ExtractionResult::skip(consumed.max(1));
        }

        // Merge lines if option is set
        let merged_text = if options.merge_dialogue_lines {
            lines.join(&options.dialogue_line_separator)
        } else {
            lines.join("\n")
        };

        // Generate unit ID
        let unit_id = generate_unit_id(path_prefix, index, "dialogue");

        // Create translation unit
        let unit = TranslationUnit::new(
            unit_id,
            path_prefix.append_index(index),
            EventCode::ShowTextBody,
            merged_text.clone(),
        )
        .with_speaker(context.current_speaker.clone())
        .with_context(context.to_translation_context());

        ExtractionResult::single(unit, consumed)
            .with_preceding(merged_text)
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
        
        // Get the original indent
        let indent = commands[index].indent;

        // Count consecutive 401 commands
        let mut old_count = 0;
        while index + old_count < commands.len() {
            let cmd = &commands[index + old_count];
            if cmd.code != 401 || cmd.indent != indent {
                break;
            }
            old_count += 1;
        }

        // Look up translation
        let unit_id = generate_unit_id(path_prefix, index, "dialogue");
        
        match translations.get(&unit_id) {
            Some(translated) => {
                // Split translated text into lines
                let new_lines = options.split_text(translated);

                // Create new 401 commands
                let new_commands: Vec<EventCommand> = new_lines
                    .iter()
                    .map(|line| EventCommand::dialogue(indent, line))
                    .collect();

                let _new_count = new_commands.len();

                // Replace old commands with new ones
                let _ = commands.splice(index..index + old_count, new_commands);

                result.applied += 1;
                result.commands_modified += old_count;
            }
            None => {
                if !options.skip_missing_translations {
                    result.not_found += 1;
                    result.add_warning(format!("Translation not found for: {}", unit_id));
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_show_text(speaker: &str) -> EventCommand {
        EventCommand {
            code: 101,
            indent: 0,
            parameters: vec![
                json!("Actor1"),
                json!(0),
                json!(0),
                json!(2),
                json!(speaker),
            ],
        }
    }

    fn make_dialogue(text: &str, indent: i32) -> EventCommand {
        EventCommand {
            code: 401,
            indent,
            parameters: vec![json!(text)],
        }
    }

    #[test]
    fn test_show_text_extracts_speaker() {
        let handler = ShowTextHandler;
        let commands = vec![make_show_text("村人A")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.speaker_update, Some(Some("村人A".to_string())));
    }

    #[test]
    fn test_dialogue_extracts_single_line() {
        let handler = DialogueHandler;
        let commands = vec![make_dialogue("Hello!", 0)];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "Hello!");
    }

    #[test]
    fn test_dialogue_merges_consecutive_lines() {
        let handler = DialogueHandler;
        let commands = vec![
            make_dialogue("Line 1", 0),
            make_dialogue("Line 2", 0),
            make_dialogue("Line 3", 0),
        ];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 3);
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_dialogue_respects_indent() {
        let handler = DialogueHandler;
        let commands = vec![
            make_dialogue("Line 1", 0),
            make_dialogue("Line 2", 0),
            make_dialogue("Nested", 1), // Different indent
        ];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 2); // Only first two
        assert_eq!(result.units[0].original, "Line 1\nLine 2");
    }

    #[test]
    fn test_dialogue_injection() {
        let handler = DialogueHandler;
        let mut commands = vec![
            make_dialogue("Line 1", 0),
            make_dialogue("Line 2", 0),
        ];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert("0_dialogue".to_string(), "Translated Line 1\nTranslated Line 2".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].get_dialogue_text(), Some("Translated Line 1"));
        assert_eq!(commands[1].get_dialogue_text(), Some("Translated Line 2"));
    }

    #[test]
    fn test_dialogue_injection_with_max_length() {
        let handler = DialogueHandler;
        let mut commands = vec![make_dialogue("Short", 0)];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default().with_max_line_length(20);

        let mut translations = HashMap::new();
        translations.insert("0_dialogue".to_string(), "This is a very long translated text that should be split".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        // Should have been split into multiple commands
        assert!(commands.len() > 1);
        // Each line should be <= 20 characters
        for cmd in &commands {
            assert!(cmd.get_dialogue_text().unwrap().len() <= 20);
        }
    }

    #[test]
    fn test_dialogue_with_speaker_context() {
        let show_text_handler = ShowTextHandler;
        let dialogue_handler = DialogueHandler;
        
        let commands = vec![
            make_show_text("村人A"),
            make_dialogue("こんにちは！", 0),
        ];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        // First, extract from show text to update speaker
        let result1 = show_text_handler.extract(&commands, 0, &path, &mut context, &options);
        if let Some(speaker) = result1.speaker_update {
            context.set_speaker(speaker);
        }

        // Then extract dialogue
        let result2 = dialogue_handler.extract(&commands, 1, &path, &mut context, &options);

        assert_eq!(result2.units[0].speaker, Some("村人A".to_string()));
    }
}
