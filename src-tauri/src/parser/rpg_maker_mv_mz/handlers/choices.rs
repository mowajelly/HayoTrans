//! Choice handlers for Show Choices (102) and When [Choice] (402) commands

use super::{generate_unit_id, CommandHandler};
use crate::parser::rpg_maker_mv_mz::command::EventCommand;
use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, ExtractionResult, InjectionOptions,
    InjectionResult, TranslationPath, TranslationUnit,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Handler for Show Choices command (102)
/// Extracts all choice options from the choices array
#[derive(Debug, Clone)]
pub struct ChoicesHandler;

impl CommandHandler for ChoicesHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::ShowChoices]
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

        // Get choices array
        let choices = match cmd.get_choices() {
            Some(c) => c,
            None => return ExtractionResult::empty(),
        };

        let mut units = Vec::new();
        let base_path = path_prefix.append_index(index);

        for (i, choice_text) in choices.iter().enumerate() {
            // Skip empty choices unless configured to include them
            if choice_text.trim().is_empty() && !options.include_empty {
                continue;
            }

            let text = if options.trim_whitespace {
                choice_text.trim().to_string()
            } else {
                choice_text.to_string()
            };

            let unit_id = format!("{}_choice_{}", base_path.to_unit_id(""), i);
            let unit_path = base_path.append_key("parameters").append_index(0).append_index(i);

            let unit = TranslationUnit::new(
                unit_id,
                unit_path,
                EventCode::ShowChoices,
                text,
            )
            .with_speaker(context.current_speaker.clone())
            .with_context(context.to_translation_context());

            units.push(unit);
        }

        ExtractionResult::multiple(units, 1)
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
        let cmd = &mut commands[index];

        // Get mutable access to choices array
        if cmd.parameters.is_empty() {
            return result;
        }

        let base_path = path_prefix.append_index(index);

        if let Some(choices_array) = cmd.parameters[0].as_array_mut() {
            for (i, choice) in choices_array.iter_mut().enumerate() {
                let unit_id = format!("{}_choice_{}", base_path.to_unit_id(""), i);
                
                if let Some(translated) = translations.get(&unit_id) {
                    *choice = json!(translated);
                    result.applied += 1;
                }
            }
            result.commands_modified += 1;
        }

        result
    }
}

/// Handler for When [Choice] command (402)
/// This handles choice branches which also contain the choice text
#[derive(Debug, Clone)]
pub struct ChoiceBranchHandler;

impl CommandHandler for ChoiceBranchHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::WhenChoice]
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

        // Get choice text (parameter index 1)
        let choice_text = match cmd.get_choice_text() {
            Some(t) => t,
            None => return ExtractionResult::empty(),
        };

        // Skip empty choices
        if choice_text.trim().is_empty() && !options.include_empty {
            return ExtractionResult::empty();
        }

        let text = if options.trim_whitespace {
            choice_text.trim().to_string()
        } else {
            choice_text.to_string()
        };

        let unit_id = generate_unit_id(path_prefix, index, "choice_branch");

        let unit = TranslationUnit::new(
            unit_id,
            path_prefix.append_index(index),
            EventCode::WhenChoice,
            text,
        )
        .with_speaker(context.current_speaker.clone())
        .with_context(context.to_translation_context());

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
        let unit_id = generate_unit_id(path_prefix, index, "choice_branch");

        if let Some(translated) = translations.get(&unit_id) {
            let cmd = &mut commands[index];
            if cmd.parameters.len() >= 2 {
                cmd.parameters[1] = Value::String(translated.clone());
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

    fn make_choices(choices: &[&str]) -> EventCommand {
        EventCommand {
            code: 102,
            indent: 0,
            parameters: vec![
                json!(choices),
                json!(0),  // default choice
                json!(1),  // cancel type
                json!(2),  // background
                json!(0),  // position
            ],
        }
    }

    fn make_choice_branch(choice_index: i32, text: &str) -> EventCommand {
        EventCommand {
            code: 402,
            indent: 1,
            parameters: vec![json!(choice_index), json!(text)],
        }
    }

    #[test]
    fn test_choices_extraction() {
        let handler = ChoicesHandler;
        let commands = vec![make_choices(&["はい", "いいえ", "考え中"])];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.units.len(), 3);
        assert_eq!(result.units[0].original, "はい");
        assert_eq!(result.units[1].original, "いいえ");
        assert_eq!(result.units[2].original, "考え中");
    }

    #[test]
    fn test_choices_skip_empty() {
        let handler = ChoicesHandler;
        let commands = vec![make_choices(&["はい", "", "いいえ"])];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 2);
        assert_eq!(result.units[0].original, "はい");
        assert_eq!(result.units[1].original, "いいえ");
    }

    #[test]
    fn test_choices_injection() {
        let handler = ChoicesHandler;
        let mut commands = vec![make_choices(&["はい", "いいえ"])];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert("0_choice_0".to_string(), "Yes".to_string());
        translations.insert("0_choice_1".to_string(), "No".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 2);
        let choices = commands[0].get_choices().unwrap();
        assert_eq!(choices[0], "Yes");
        assert_eq!(choices[1], "No");
    }

    #[test]
    fn test_choice_branch_extraction() {
        let handler = ChoiceBranchHandler;
        let commands = vec![make_choice_branch(0, "はい")];
        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.consumed, 1);
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "はい");
    }

    #[test]
    fn test_choice_branch_injection() {
        let handler = ChoiceBranchHandler;
        let mut commands = vec![make_choice_branch(0, "はい")];
        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert("0_choice_branch".to_string(), "Yes".to_string());

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        assert_eq!(commands[0].get_choice_text(), Some("Yes"));
    }

    #[test]
    fn test_full_choice_flow() {
        // Simulate a complete choice structure
        let choices_handler = ChoicesHandler;
        let branch_handler = ChoiceBranchHandler;

        let commands = vec![
            make_choices(&["はい", "いいえ"]),
            make_choice_branch(0, "はい"),
            // More commands would be here
            make_choice_branch(1, "いいえ"),
        ];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        // Extract from Show Choices (102)
        let result1 = choices_handler.extract(&commands, 0, &path, &mut context, &options);
        assert_eq!(result1.units.len(), 2);

        // Extract from When [Choice] (402) - both branches
        let result2 = branch_handler.extract(&commands, 1, &path, &mut context, &options);
        let result3 = branch_handler.extract(&commands, 2, &path, &mut context, &options);

        assert_eq!(result2.units.len(), 1);
        assert_eq!(result3.units.len(), 1);
    }
}
