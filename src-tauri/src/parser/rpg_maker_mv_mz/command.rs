//! Event command structure for RPG Maker MV/MZ
//!
//! Represents a single command in an event's command list.

use crate::parser::types::EventCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single event command from RPG Maker MV/MZ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCommand {
    /// Command code (e.g., 101, 401, 102)
    pub code: i32,
    /// Indent level (for nested commands like conditionals)
    #[serde(default)]
    pub indent: i32,
    /// Command parameters
    #[serde(default)]
    pub parameters: Vec<Value>,
}

impl EventCommand {
    /// Create a new event command
    pub fn new(code: i32, indent: i32, parameters: Vec<Value>) -> Self {
        Self {
            code,
            indent,
            parameters,
        }
    }

    /// Get the event code as enum
    pub fn event_code(&self) -> EventCode {
        EventCode::from(self.code)
    }

    /// Check if this is a dialogue text command (401)
    pub fn is_dialogue(&self) -> bool {
        self.code == 401
    }

    /// Check if this is a show text header (101)
    pub fn is_show_text(&self) -> bool {
        self.code == 101
    }

    /// Check if this is a choice command (102)
    pub fn is_choice(&self) -> bool {
        self.code == 102
    }

    /// Get a string parameter at index
    pub fn get_string_param(&self, index: usize) -> Option<&str> {
        self.parameters.get(index)?.as_str()
    }

    /// Get an integer parameter at index
    pub fn get_int_param(&self, index: usize) -> Option<i64> {
        self.parameters.get(index)?.as_i64()
    }

    /// Get an array parameter at index
    pub fn get_array_param(&self, index: usize) -> Option<&Vec<Value>> {
        self.parameters.get(index)?.as_array()
    }

    /// Get an object parameter at index
    pub fn get_object_param(&self, index: usize) -> Option<&serde_json::Map<String, Value>> {
        self.parameters.get(index)?.as_object()
    }

    /// Set a string parameter at index
    pub fn set_string_param(&mut self, index: usize, value: &str) -> bool {
        if index < self.parameters.len() {
            self.parameters[index] = Value::String(value.to_string());
            true
        } else {
            false
        }
    }

    /// Extract speaker name from ShowText command (101)
    /// Format: ["face_name", face_index, background, position, "speaker_name"]
    pub fn get_speaker_name(&self) -> Option<String> {
        if self.code != 101 {
            return None;
        }
        // MV/MZ format has speaker name at index 4
        if self.parameters.len() >= 5 {
            let speaker = self.get_string_param(4)?;
            if !speaker.is_empty() {
                return Some(speaker.to_string());
            }
        }
        None
    }

    /// Get the dialogue text from a 401 command
    pub fn get_dialogue_text(&self) -> Option<&str> {
        if self.code != 401 {
            return None;
        }
        self.get_string_param(0)
    }

    /// Get choices array from a 102 command
    /// Format: [["choice1", "choice2", ...], default, cancelType, ...]
    pub fn get_choices(&self) -> Option<Vec<&str>> {
        if self.code != 102 {
            return None;
        }
        let choices_array = self.get_array_param(0)?;
        choices_array
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .into()
    }

    /// Get choice text from a 402 (When [Choice]) command
    /// Format: [choice_index, "choice_text"]
    pub fn get_choice_text(&self) -> Option<&str> {
        if self.code != 402 {
            return None;
        }
        self.get_string_param(1)
    }

    /// Get plugin command data (357)
    /// Format: ["plugin_name", "command", "display_name", {args}]
    pub fn get_plugin_data(&self) -> Option<PluginCommandData> {
        if self.code != 357 {
            return None;
        }
        if self.parameters.len() < 4 {
            return None;
        }

        Some(PluginCommandData {
            plugin_name: self.get_string_param(0)?.to_string(),
            command: self.get_string_param(1)?.to_string(),
            display_name: self.get_string_param(2)?.to_string(),
            arguments: self.parameters.get(3)?.clone(),
        })
    }

    /// Get comment text from a 408 command
    pub fn get_comment_text(&self) -> Option<&str> {
        if self.code != 408 {
            return None;
        }
        self.get_string_param(0)
    }

    /// Check if this is a script continuation with special text (657)
    pub fn get_script_special_text(&self, prefix: &str) -> Option<String> {
        if self.code != 657 {
            return None;
        }
        let text = self.get_string_param(0)?;
        if text.starts_with(prefix) {
            Some(text[prefix.len()..].to_string())
        } else {
            None
        }
    }

    /// Create an empty/end command (code 0)
    pub fn empty() -> Self {
        Self {
            code: 0,
            indent: 0,
            parameters: Vec::new(),
        }
    }

    /// Create a dialogue text command (401)
    pub fn dialogue(indent: i32, text: &str) -> Self {
        Self {
            code: 401,
            indent,
            parameters: vec![Value::String(text.to_string())],
        }
    }
}

/// Plugin command data extracted from code 357
#[derive(Debug, Clone)]
pub struct PluginCommandData {
    /// Name of the plugin (e.g., "QuestSystem")
    pub plugin_name: String,
    /// Command identifier
    pub command: String,
    /// Display name for the command
    pub display_name: String,
    /// Arguments object
    pub arguments: Value,
}

impl PluginCommandData {
    /// Get a string argument by key
    pub fn get_string_arg(&self, key: &str) -> Option<&str> {
        self.arguments.get(key)?.as_str()
    }

    /// Get a nested value by path (dot-separated)
    pub fn get_by_path(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.arguments;

        for part in parts {
            if let Ok(index) = part.parse::<usize>() {
                current = current.get(index)?;
            } else {
                current = current.get(part)?;
            }
        }

        Some(current)
    }
}

/// Parse event commands from a JSON array
pub fn parse_commands(list: &Value) -> Vec<EventCommand> {
    list.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Convert commands back to JSON array
pub fn commands_to_json(commands: &[EventCommand]) -> Value {
    Value::Array(
        commands
            .iter()
            .map(|cmd| serde_json::to_value(cmd).unwrap_or(Value::Null))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_command_from_json() {
        let json = json!({
            "code": 401,
            "indent": 0,
            "parameters": ["Hello, world!"]
        });

        let cmd: EventCommand = serde_json::from_value(json).unwrap();
        assert_eq!(cmd.code, 401);
        assert_eq!(cmd.indent, 0);
        assert_eq!(cmd.get_dialogue_text(), Some("Hello, world!"));
    }

    #[test]
    fn test_show_text_speaker() {
        let cmd = EventCommand {
            code: 101,
            indent: 0,
            parameters: vec![
                json!("Actor1"),
                json!(0),
                json!(0),
                json!(2),
                json!("村人A"),
            ],
        };

        assert_eq!(cmd.get_speaker_name(), Some("村人A".to_string()));
    }

    #[test]
    fn test_show_text_no_speaker() {
        let cmd = EventCommand {
            code: 101,
            indent: 0,
            parameters: vec![
                json!("Actor1"),
                json!(0),
                json!(0),
                json!(2),
                json!(""),
            ],
        };

        assert_eq!(cmd.get_speaker_name(), None);
    }

    #[test]
    fn test_choices() {
        let cmd = EventCommand {
            code: 102,
            indent: 0,
            parameters: vec![
                json!(["はい", "いいえ"]),
                json!(0),
                json!(1),
                json!(2),
                json!(0),
            ],
        };

        let choices = cmd.get_choices().unwrap();
        assert_eq!(choices, vec!["はい", "いいえ"]);
    }

    #[test]
    fn test_choice_branch() {
        let cmd = EventCommand {
            code: 402,
            indent: 1,
            parameters: vec![json!(0), json!("はい")],
        };

        assert_eq!(cmd.get_choice_text(), Some("はい"));
    }

    #[test]
    fn test_plugin_command() {
        let cmd = EventCommand {
            code: 357,
            indent: 0,
            parameters: vec![
                json!("TorigoyaMZ_NotifyMessage"),
                json!("notify"),
                json!("通知の表示"),
                json!({
                    "message": "テストメッセージ",
                    "icon": "",
                    "note": ""
                }),
            ],
        };

        let plugin_data = cmd.get_plugin_data().unwrap();
        assert_eq!(plugin_data.plugin_name, "TorigoyaMZ_NotifyMessage");
        assert_eq!(plugin_data.command, "notify");
        assert_eq!(plugin_data.get_string_arg("message"), Some("テストメッセージ"));
    }

    #[test]
    fn test_script_special_text() {
        let cmd = EventCommand {
            code: 657,
            indent: 0,
            parameters: vec![json!("テキスト = これは特別なテキストです")],
        };

        let text = cmd.get_script_special_text("テキスト = ").unwrap();
        assert_eq!(text, "これは特別なテキストです");
    }

    #[test]
    fn test_parse_commands() {
        let json = json!([
            {"code": 101, "indent": 0, "parameters": ["", 0, 0, 2, "NPC"]},
            {"code": 401, "indent": 0, "parameters": ["Hello!"]},
            {"code": 401, "indent": 0, "parameters": ["How are you?"]},
            {"code": 0, "indent": 0, "parameters": []}
        ]);

        let commands = parse_commands(&json);
        assert_eq!(commands.len(), 4);
        assert_eq!(commands[0].code, 101);
        assert_eq!(commands[1].get_dialogue_text(), Some("Hello!"));
    }

    #[test]
    fn test_create_dialogue_command() {
        let cmd = EventCommand::dialogue(0, "New text");
        assert_eq!(cmd.code, 401);
        assert_eq!(cmd.indent, 0);
        assert_eq!(cmd.get_dialogue_text(), Some("New text"));
    }
}
