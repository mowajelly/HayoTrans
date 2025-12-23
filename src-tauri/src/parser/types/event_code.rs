//! Event command codes for RPG Maker MV/MZ
//!
//! These codes define the type of command in event lists.

use serde::{Deserialize, Serialize};

/// Event command codes used in RPG Maker MV/MZ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCode {
    /// Show Text - Contains speaker/face info (101)
    ShowText,
    /// Show Text Body - Actual dialogue content (401)
    ShowTextBody,
    /// Show Scrolling Text header (105)
    ShowScrollingText,
    /// Show Scrolling Text body (405)
    ScrollingTextBody,
    /// Comment header (108)
    Comment,
    /// Comment body/continuation (408)
    CommentBody,
    /// Show Choices - Multiple choice options (102)
    ShowChoices,
    /// When [Choice] - Choice branch handler (402)
    WhenChoice,
    /// When Cancel - Cancel branch handler (403)
    WhenCancel,
    /// Choices End (404)
    ChoicesEnd,
    /// Input Number (103)
    InputNumber,
    /// Select Item (104)
    SelectItem,
    /// Plugin Command - MV/MZ plugin commands (357)
    PluginCommand,
    /// Script header (355)
    Script,
    /// Script body/continuation (655)
    ScriptBody,
    /// Script body continuation (657) - used by some plugins
    ScriptBodyAlt,
    /// Change Nickname (324)
    ChangeNickname,
    /// Change Profile (325)
    ChangeProfile,
    /// Unknown or custom command code
    Unknown(i32),
}

impl EventCode {
    /// Get the numeric code value
    pub fn code(&self) -> i32 {
        match self {
            Self::ShowText => 101,
            Self::ShowTextBody => 401,
            Self::ShowScrollingText => 105,
            Self::ScrollingTextBody => 405,
            Self::Comment => 108,
            Self::CommentBody => 408,
            Self::ShowChoices => 102,
            Self::WhenChoice => 402,
            Self::WhenCancel => 403,
            Self::ChoicesEnd => 404,
            Self::InputNumber => 103,
            Self::SelectItem => 104,
            Self::PluginCommand => 357,
            Self::Script => 355,
            Self::ScriptBody => 655,
            Self::ScriptBodyAlt => 657,
            Self::ChangeNickname => 324,
            Self::ChangeProfile => 325,
            Self::Unknown(code) => *code,
        }
    }

    /// Check if this is a translatable text command
    pub fn is_translatable(&self) -> bool {
        matches!(
            self,
            Self::ShowText
                | Self::ShowTextBody
                | Self::ScrollingTextBody
                | Self::CommentBody
                | Self::ShowChoices
                | Self::WhenChoice
                | Self::PluginCommand
                | Self::ScriptBodyAlt
                | Self::ChangeNickname
                | Self::ChangeProfile
        )
    }

    /// Check if this command continues from a previous command
    pub fn is_continuation(&self) -> bool {
        matches!(
            self,
            Self::ShowTextBody
                | Self::ScrollingTextBody
                | Self::CommentBody
                | Self::ScriptBody
                | Self::ScriptBodyAlt
        )
    }

    /// Get a human-readable name for the command
    pub fn name(&self) -> &'static str {
        match self {
            Self::ShowText => "ShowText",
            Self::ShowTextBody => "TextBody",
            Self::ShowScrollingText => "ScrollingText",
            Self::ScrollingTextBody => "ScrollingTextBody",
            Self::Comment => "Comment",
            Self::CommentBody => "CommentBody",
            Self::ShowChoices => "Choices",
            Self::WhenChoice => "ChoiceBranch",
            Self::WhenCancel => "CancelBranch",
            Self::ChoicesEnd => "ChoicesEnd",
            Self::InputNumber => "InputNumber",
            Self::SelectItem => "SelectItem",
            Self::PluginCommand => "PluginCommand",
            Self::Script => "Script",
            Self::ScriptBody => "ScriptBody",
            Self::ScriptBodyAlt => "ScriptBodyAlt",
            Self::ChangeNickname => "ChangeNickname",
            Self::ChangeProfile => "ChangeProfile",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl From<i32> for EventCode {
    fn from(code: i32) -> Self {
        match code {
            101 => Self::ShowText,
            401 => Self::ShowTextBody,
            105 => Self::ShowScrollingText,
            405 => Self::ScrollingTextBody,
            108 => Self::Comment,
            408 => Self::CommentBody,
            102 => Self::ShowChoices,
            402 => Self::WhenChoice,
            403 => Self::WhenCancel,
            404 => Self::ChoicesEnd,
            103 => Self::InputNumber,
            104 => Self::SelectItem,
            357 => Self::PluginCommand,
            355 => Self::Script,
            655 => Self::ScriptBody,
            657 => Self::ScriptBodyAlt,
            324 => Self::ChangeNickname,
            325 => Self::ChangeProfile,
            _ => Self::Unknown(code),
        }
    }
}

impl From<EventCode> for i32 {
    fn from(code: EventCode) -> Self {
        code.code()
    }
}

impl std::fmt::Display for EventCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_code_from_i32() {
        assert_eq!(EventCode::from(101), EventCode::ShowText);
        assert_eq!(EventCode::from(401), EventCode::ShowTextBody);
        assert_eq!(EventCode::from(102), EventCode::ShowChoices);
        assert_eq!(EventCode::from(357), EventCode::PluginCommand);
        assert_eq!(EventCode::from(999), EventCode::Unknown(999));
    }

    #[test]
    fn test_event_code_to_i32() {
        assert_eq!(EventCode::ShowText.code(), 101);
        assert_eq!(EventCode::ShowTextBody.code(), 401);
        assert_eq!(EventCode::Unknown(999).code(), 999);
    }

    #[test]
    fn test_is_translatable() {
        assert!(EventCode::ShowTextBody.is_translatable());
        assert!(EventCode::ShowChoices.is_translatable());
        assert!(!EventCode::ChoicesEnd.is_translatable());
    }

    #[test]
    fn test_is_continuation() {
        assert!(EventCode::ShowTextBody.is_continuation());
        assert!(EventCode::CommentBody.is_continuation());
        assert!(!EventCode::ShowText.is_continuation());
    }
}
