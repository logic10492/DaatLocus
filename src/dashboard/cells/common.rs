use serde::{Deserialize, Serialize};

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::tool_ui::ToolUiData;

use super::markdown::render_markdown_width;

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

impl AssistantActivityCell {
    /// Render this cell into styled lines using width-aware markdown when in rich mode,
    /// or plain text wrapping when in raw mode.
    pub fn display_lines(&self, max_width: u16) -> Vec<Line<'static>> {
        let body = self
            .full_body
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if body.is_empty() {
            return Vec::new();
        }
        if self.rich_mode {
            let md_lines =
                render_markdown_width(&body, Color::Gray, max_width.saturating_sub(3));
            let mut out = Vec::new();
            for md_line in md_lines {
                let mut spans = vec![Span::raw("   ")];
                spans.extend(md_line.spans);
                out.push(Line::from(spans));
            }
            out
        } else {
            // Raw mode: plain text with wrapping
            let content_width = (max_width.saturating_sub(3)).max(20) as usize;
            let mut out = Vec::new();
            for raw_line in body.lines() {
                let wrapped = textwrap::wrap(raw_line, content_width);
                for sub in &wrapped {
                    out.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled(sub.to_string(), Style::default().fg(Color::Gray)),
                    ]));
                }
            }
            out
        }
    }
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

impl ThinkingActivityCell {
    /// Render thinking content into styled lines with width-aware wrapping.
    pub fn display_lines(&self, max_width: u16, bar: &str) -> Vec<Line<'static>> {
        let bar_span = Span::styled(bar.to_string(), Style::default().fg(Color::DarkGray));
        let content_width = (max_width.saturating_sub(2)).max(20) as usize;
        let mut out = Vec::new();
        if self.expanded {
            if let Some(ref full) = self.full_body {
                for body_line in full.lines() {
                    let wrapped = textwrap::wrap(body_line, content_width);
                    for sub in &wrapped {
                        out.push(Line::from(vec![
                            bar_span.clone(),
                            Span::raw(" "),
                            Span::styled(sub.to_string(), Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
            }
        } else {
            for body_line in self.body_lines.iter().take(5) {
                let wrapped = textwrap::wrap(body_line, content_width);
                for sub in &wrapped {
                    out.push(Line::from(vec![
                        bar_span.clone(),
                        Span::raw(" "),
                        Span::styled(sub.to_string(), Style::default().fg(Color::Gray)),
                    ]));
                }
            }
        }
        out
    }
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
