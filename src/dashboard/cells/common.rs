use serde::{Deserialize, Serialize};

use crate::tool_ui::ToolUiData;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssistantActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
    /// Full assistant message body. Used for width-aware per-cell rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_body: Option<String>,
    /// Rich (markdown) vs Raw (plain text) display mode.
    #[serde(default = "default_rich_mode")]
    pub rich_mode: bool,
}

fn default_rich_mode() -> bool {
    true
}

/// Controls animation behaviour in the TUI dashboard.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ReducedMotion {
    /// Full animations enabled (spinners, transitions).
    #[default]
    Full,
    /// Animations mostly disabled; static indicators preferred.
    Reduced,
}

/// Thinking / reasoning content produced by the model.
/// Rendered truncated by default; press Enter to expand.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThinkingActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
    /// Full reasoning text. Rendered when `expanded` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_body: Option<String>,
    /// Whether the cell is expanded (toggle via Enter key).
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub image_attachments: Vec<MessageImageAttachment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageImageAttachment {
    pub label: String,
    pub uri: String,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenericAppActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalWaitActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorActivityCell {
    pub title: String,
    pub body_lines: Vec<String>,
}

pub fn assistant_cell_with_body(
    title: impl Into<String>,
    body_lines: Vec<String>,
    full_body: Option<String>,
) -> AssistantActivityCell {
    AssistantActivityCell {
        title: title.into(),
        body_lines,
        full_body,
        rich_mode: true,
    }
}

pub fn user_cell(title: impl Into<String>, body_lines: Vec<String>) -> UserActivityCell {
    UserActivityCell {
        title: title.into(),
        body_lines,
        image_attachments: Vec::new(),
    }
}

pub fn generic_app_cell(
    title: impl Into<String>,
    body_lines: Vec<String>,
) -> GenericAppActivityCell {
    GenericAppActivityCell {
        title: title.into(),
        body_lines,
    }
}

pub fn terminal_wait_cell(
    title: impl Into<String>,
    body_lines: Vec<String>,
) -> TerminalWaitActivityCell {
    TerminalWaitActivityCell {
        title: title.into(),
        body_lines,
    }
}

pub fn error_cell(title: impl Into<String>, body_lines: Vec<String>) -> ErrorActivityCell {
    ErrorActivityCell {
        title: title.into(),
        body_lines,
    }
}

pub fn thinking_cell(
    title: impl Into<String>,
    body_lines: Vec<String>,
    full_body: Option<String>,
) -> ThinkingActivityCell {
    ThinkingActivityCell {
        title: title.into(),
        body_lines,
        full_body,
        expanded: false,
    }
}

impl From<ToolUiData> for GenericAppActivityCell {
    fn from(data: ToolUiData) -> Self {
        generic_app_cell(data.title, data.body_lines)
    }
}

impl From<ToolUiData> for ErrorActivityCell {
    fn from(data: ToolUiData) -> Self {
        error_cell(data.title, data.body_lines)
    }
}
