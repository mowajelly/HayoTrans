//! Plugin command handler for Plugin Command (357) events
//!
//! This handler supports both predefined and user-configurable plugin extraction.

use super::CommandHandler;
use crate::parser::rpg_maker_mv_mz::command::EventCommand;
use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, ExtractionResult, InjectionOptions,
    InjectionResult, PathPattern, TranslationPath, TranslationUnit,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration for extracting specific fields from a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExtractionConfig {
    /// Plugin name (e.g., "QuestSystem")
    pub plugin_name: String,
    /// List of field paths to extract
    pub extraction_paths: Vec<PluginFieldConfig>,
    /// Whether this config is enabled
    pub enabled: bool,
    /// Description of the plugin
    pub description: Option<String>,
}

impl PluginExtractionConfig {
    /// Create a new plugin config
    pub fn new(plugin_name: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            extraction_paths: Vec::new(),
            enabled: true,
            description: None,
        }
    }

    /// Add an extraction path
    pub fn add_path(mut self, pattern: impl Into<String>, description: Option<String>) -> Self {
        self.extraction_paths.push(PluginFieldConfig {
            pattern: pattern.into(),
            description,
            translatable: true,
        });
        self
    }
}

/// Configuration for a single field in a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginFieldConfig {
    /// Path pattern (e.g., "QuestDatas.|ARY|.Title")
    pub pattern: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Whether this field should be translated
    pub translatable: bool,
}

/// Handler for Plugin Command (357)
#[derive(Debug, Clone)]
pub struct PluginCommandHandler {
    /// Predefined plugin configurations
    predefined_configs: HashMap<String, PluginExtractionConfig>,
    /// User-defined plugin configurations
    user_configs: HashMap<String, PluginExtractionConfig>,
}

impl PluginCommandHandler {
    /// Create a new plugin command handler with predefined configs
    pub fn new() -> Self {
        let mut handler = Self {
            predefined_configs: HashMap::new(),
            user_configs: HashMap::new(),
        };
        handler.load_predefined_configs();
        handler
    }

    /// Load predefined plugin configurations
    fn load_predefined_configs(&mut self) {
        // TorigoyaMZ_NotifyMessage
        self.predefined_configs.insert(
            "TorigoyaMZ_NotifyMessage".to_string(),
            PluginExtractionConfig::new("TorigoyaMZ_NotifyMessage")
                .add_path("message", Some("Notification message".to_string())),
        );

        // NotifyMessage_Battle
        self.predefined_configs.insert(
            "NotifyMessage_Battle".to_string(),
            PluginExtractionConfig::new("NotifyMessage_Battle")
                .add_path("message", Some("Battle notification message".to_string())),
        );

        // BattleLogOutput
        self.predefined_configs.insert(
            "BattleLogOutput".to_string(),
            PluginExtractionConfig::new("BattleLogOutput")
                .add_path("message", Some("Battle log message".to_string())),
        );
    }

    /// Add a user-defined config
    pub fn add_user_config(&mut self, config: PluginExtractionConfig) {
        self.user_configs.insert(config.plugin_name.clone(), config);
    }

    /// Get config for a plugin (user config takes precedence)
    pub fn get_config(&self, plugin_name: &str) -> Option<&PluginExtractionConfig> {
        self.user_configs
            .get(plugin_name)
            .or_else(|| self.predefined_configs.get(plugin_name))
    }

    /// Extract translatable fields from plugin arguments
    fn extract_from_args(
        &self,
        plugin_name: &str,
        args: &Value,
        path_prefix: &TranslationPath,
        index: usize,
        context: &ExtractionContext,
        options: &ExtractionOptions,
    ) -> Vec<TranslationUnit> {
        let config = match self.get_config(plugin_name) {
            Some(c) if c.enabled => c,
            _ => return Vec::new(),
        };

        let mut units = Vec::new();
        let base_path = path_prefix.append_index(index);

        for field_config in &config.extraction_paths {
            if !field_config.translatable {
                continue;
            }

            let pattern = PathPattern::new(&field_config.pattern);
            
            // Walk the arguments to find matching paths
            let matches = self.find_matching_fields(args, &pattern, "");
            
            for (field_path, value) in matches {
                if let Some(text) = value.as_str() {
                    let text = if options.trim_whitespace {
                        text.trim().to_string()
                    } else {
                        text.to_string()
                    };

                    if text.is_empty() && !options.include_empty {
                        continue;
                    }

                    let unit_id = format!(
                        "{}_plugin_{}_{}",
                        base_path.to_unit_id(""),
                        plugin_name.replace('.', "_"),
                        field_path.replace('.', "_")
                    );

                    let mut trans_context = context.to_translation_context();
                    trans_context.add_tag(format!("plugin:{}", plugin_name));
                    trans_context.add_tag(format!("field:{}", field_path));

                    let unit = TranslationUnit::new(
                        unit_id,
                        base_path.append_key("parameters").append_index(3),
                        EventCode::PluginCommand,
                        text,
                    )
                    .with_context(trans_context);

                    units.push(unit);
                }
            }
        }

        units
    }

    /// Find fields matching a pattern in a JSON value
    fn find_matching_fields<'a>(
        &self,
        value: &'a Value,
        pattern: &PathPattern,
        current_path: &str,
    ) -> Vec<(String, &'a Value)> {
        let mut results = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };

                    if pattern.matches(&new_path) {
                        results.push((new_path.clone(), val));
                    }

                    // Recurse into nested structures
                    results.extend(self.find_matching_fields(val, pattern, &new_path));
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_path = if current_path.is_empty() {
                        i.to_string()
                    } else {
                        format!("{}.{}", current_path, i)
                    };

                    if pattern.matches(&new_path) {
                        results.push((new_path.clone(), val));
                    }

                    results.extend(self.find_matching_fields(val, pattern, &new_path));
                }
            }
            _ => {}
        }

        results
    }

    /// Inject translations back into plugin arguments
    fn inject_to_args(
        &self,
        plugin_name: &str,
        args: &mut Value,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        index: usize,
    ) -> InjectionResult {
        let mut result = InjectionResult::new();

        let config = match self.get_config(plugin_name) {
            Some(c) if c.enabled => c,
            _ => return result,
        };

        let base_path = path_prefix.append_index(index);

        for field_config in &config.extraction_paths {
            if !field_config.translatable {
                continue;
            }

            let pattern = PathPattern::new(&field_config.pattern);
            let matches = self.find_matching_field_paths(args, &pattern, "");

            for field_path in matches {
                let unit_id = format!(
                    "{}_plugin_{}_{}",
                    base_path.to_unit_id(""),
                    plugin_name.replace('.', "_"),
                    field_path.replace('.', "_")
                );

                if let Some(translated) = translations.get(&unit_id) {
                    if self.set_field_value(args, &field_path, translated.clone()) {
                        result.applied += 1;
                    }
                }
            }
        }

        if result.applied > 0 {
            result.commands_modified += 1;
        }

        result
    }

    /// Find paths matching a pattern (returns paths, not values)
    fn find_matching_field_paths(
        &self,
        value: &Value,
        pattern: &PathPattern,
        current_path: &str,
    ) -> Vec<String> {
        let mut results = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };

                    if pattern.matches(&new_path) && val.is_string() {
                        results.push(new_path.clone());
                    }

                    results.extend(self.find_matching_field_paths(val, pattern, &new_path));
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_path = if current_path.is_empty() {
                        i.to_string()
                    } else {
                        format!("{}.{}", current_path, i)
                    };

                    if pattern.matches(&new_path) && val.is_string() {
                        results.push(new_path.clone());
                    }

                    results.extend(self.find_matching_field_paths(val, pattern, &new_path));
                }
            }
            _ => {}
        }

        results
    }

    /// Set a field value by path
    fn set_field_value(&self, value: &mut Value, path: &str, new_value: String) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;

            if let Ok(index) = part.parse::<usize>() {
                if is_last {
                    if let Some(arr) = current.as_array_mut() {
                        if index < arr.len() {
                            arr[index] = Value::String(new_value);
                            return true;
                        }
                    }
                } else {
                    current = match current.get_mut(index) {
                        Some(v) => v,
                        None => return false,
                    };
                }
            } else {
                if is_last {
                    if let Some(obj) = current.as_object_mut() {
                        obj.insert(part.to_string(), Value::String(new_value));
                        return true;
                    }
                } else {
                    current = match current.get_mut(*part) {
                        Some(v) => v,
                        None => return false,
                    };
                }
            }
        }

        false
    }
}

impl Default for PluginCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for PluginCommandHandler {
    fn handles(&self) -> Vec<EventCode> {
        vec![EventCode::PluginCommand]
    }

    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> ExtractionResult {
        if !options.extract_plugins {
            return ExtractionResult::empty();
        }

        let cmd = &commands[index];
        let plugin_data = match cmd.get_plugin_data() {
            Some(d) => d,
            None => return ExtractionResult::empty(),
        };

        let units = self.extract_from_args(
            &plugin_data.plugin_name,
            &plugin_data.arguments,
            path_prefix,
            index,
            context,
            options,
        );

        if units.is_empty() {
            ExtractionResult::empty()
        } else {
            ExtractionResult::multiple(units, 1)
        }
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
        let cmd = &commands[index];
        let plugin_data = match cmd.get_plugin_data() {
            Some(d) => d,
            None => return InjectionResult::new(),
        };

        let plugin_name = plugin_data.plugin_name.clone();
        let mut args = plugin_data.arguments.clone();

        let result = self.inject_to_args(
            &plugin_name,
            &mut args,
            translations,
            path_prefix,
            index,
        );

        if result.applied > 0 {
            // Update the command with new arguments
            commands[index].parameters[3] = args;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_plugin_command(plugin_name: &str, args: Value) -> EventCommand {
        EventCommand {
            code: 357,
            indent: 0,
            parameters: vec![
                json!(plugin_name),
                json!("command"),
                json!("Display Name"),
                args,
            ],
        }
    }

    #[test]
    fn test_torigoya_notify_message() {
        let handler = PluginCommandHandler::new();
        let commands = vec![make_plugin_command(
            "TorigoyaMZ_NotifyMessage",
            json!({
                "message": "テストメッセージ",
                "icon": "",
                "note": ""
            }),
        )];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "テストメッセージ");
        assert!(result.units[0].context.tags.contains(&"plugin:TorigoyaMZ_NotifyMessage".to_string()));
    }

    #[test]
    fn test_unknown_plugin() {
        let handler = PluginCommandHandler::new();
        let commands = vec![make_plugin_command(
            "UnknownPlugin",
            json!({
                "someField": "Some Value"
            }),
        )];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        // Unknown plugins should return no units
        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn test_user_defined_config() {
        let mut handler = PluginCommandHandler::new();
        
        // Add user config for a custom plugin
        handler.add_user_config(
            PluginExtractionConfig::new("CustomPlugin")
                .add_path("customField", Some("Custom field".to_string()))
        );

        let commands = vec![make_plugin_command(
            "CustomPlugin",
            json!({
                "customField": "カスタムテキスト"
            }),
        )];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].original, "カスタムテキスト");
    }

    #[test]
    fn test_plugin_extraction_disabled() {
        let handler = PluginCommandHandler::new();
        let commands = vec![make_plugin_command(
            "TorigoyaMZ_NotifyMessage",
            json!({
                "message": "テストメッセージ"
            }),
        )];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let mut options = ExtractionOptions::default();
        options.extract_plugins = false;

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn test_plugin_injection() {
        let handler = PluginCommandHandler::new();
        let mut commands = vec![make_plugin_command(
            "TorigoyaMZ_NotifyMessage",
            json!({
                "message": "テストメッセージ",
                "icon": "",
                "note": ""
            }),
        )];

        let path = TranslationPath::new();
        let context = ExtractionContext::new("test.json");
        let options = InjectionOptions::default();

        let mut translations = HashMap::new();
        translations.insert(
            "0_plugin_TorigoyaMZ_NotifyMessage_message".to_string(),
            "Test Message".to_string(),
        );

        let result = handler.inject(&mut commands, 0, &translations, &path, &context, &options);

        assert_eq!(result.applied, 1);
        let args = &commands[0].parameters[3];
        assert_eq!(args["message"].as_str(), Some("Test Message"));
    }

    #[test]
    fn test_nested_field_extraction() {
        let mut handler = PluginCommandHandler::new();
        
        // Config with nested path pattern
        handler.add_user_config(
            PluginExtractionConfig::new("QuestPlugin")
                .add_path("quests.|ARY|.title", Some("Quest title".to_string()))
        );

        let commands = vec![make_plugin_command(
            "QuestPlugin",
            json!({
                "quests": [
                    {"title": "クエスト1", "completed": false},
                    {"title": "クエスト2", "completed": true}
                ]
            }),
        )];

        let path = TranslationPath::new();
        let mut context = ExtractionContext::new("test.json");
        let options = ExtractionOptions::default();

        let result = handler.extract(&commands, 0, &path, &mut context, &options);

        assert_eq!(result.units.len(), 2);
        assert!(result.units.iter().any(|u| u.original == "クエスト1"));
        assert!(result.units.iter().any(|u| u.original == "クエスト2"));
    }
}
