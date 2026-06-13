use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crossterm::{cursor::MoveTo, queue, style::Print};
use ratatui::{buffer::Buffer, layout::Rect};
use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TerminalHyperlinkOverlay {
    pub(super) x: u16,
    pub(super) y: u16,
    pub(super) text: String,
    pub(super) target: String,
}

pub(super) fn collect_terminal_hyperlink_overlays(
    buffer: &Buffer,
    area: Rect,
) -> Vec<TerminalHyperlinkOverlay> {
    let mut overlays = Vec::new();
    for y in area.top()..area.bottom() {
        let row = rendered_row(buffer, area, y);
        if row.trim().is_empty() {
            continue;
        }
        let mut occupied = Vec::new();
        for matched in url_regex().find_iter(&row.text) {
            let text = trim_trailing_link_punctuation(matched.as_str());
            if text.is_empty() {
                continue;
            }
            let start = matched.start();
            let end = start + text.len();
            occupied.push((start, end));
            if let Some(overlay) = row_overlay(&row, y, start, text, text) {
                overlays.push(overlay);
            }
        }
        for matched in file_regex().find_iter(&row.text) {
            let text = trim_trailing_link_punctuation(matched.as_str());
            if text.is_empty() {
                continue;
            }
            let start = matched.start();
            let end = start + text.len();
            if occupied.iter().any(|(occupied_start, occupied_end)| {
                ranges_overlap(start, end, *occupied_start, *occupied_end)
            }) {
                continue;
            }
            if let Some(target) = file_uri_for_display_path(text) {
                if let Some(overlay) = row_overlay(&row, y, start, text, &target) {
                    overlays.push(overlay);
                }
            }
        }
    }
    overlays
}

pub(super) fn write_terminal_hyperlink_overlays<W: Write>(
    writer: &mut W,
    overlays: &[TerminalHyperlinkOverlay],
    cursor_pos: Option<(u16, u16)>,
) -> io::Result<()> {
    for overlay in overlays {
        let target = sanitize_osc8_part(&overlay.target);
        let text = sanitize_osc8_part(&overlay.text);
        queue!(
            writer,
            MoveTo(overlay.x, overlay.y),
            Print(format!("\x1b]8;;{target}\x1b\\{text}\x1b]8;;\x1b\\"))
        )?;
    }
    if let Some((x, y)) = cursor_pos {
        queue!(writer, MoveTo(x, y))?;
    }
    writer.flush()
}

struct RenderedRow {
    text: String,
    byte_columns: Vec<(usize, u16)>,
}

impl RenderedRow {
    fn trim(&self) -> &str {
        self.text.trim()
    }

    fn x_for_byte(&self, byte_offset: usize) -> Option<u16> {
        match self
            .byte_columns
            .binary_search_by_key(&byte_offset, |(offset, _)| *offset)
        {
            Ok(index) => Some(self.byte_columns[index].1),
            Err(0) => None,
            Err(index) => Some(self.byte_columns[index.saturating_sub(1)].1),
        }
    }
}

fn rendered_row(buffer: &Buffer, area: Rect, y: u16) -> RenderedRow {
    let mut text = String::new();
    let mut byte_columns = Vec::new();
    for x in area.left()..area.right() {
        if let Some(cell) = buffer.cell((x, y)) {
            byte_columns.push((text.len(), x));
            text.push_str(cell.symbol());
        }
    }
    byte_columns.push((text.len(), area.right()));
    RenderedRow { text, byte_columns }
}

fn row_overlay(
    row: &RenderedRow,
    y: u16,
    byte_start: usize,
    text: &str,
    target: &str,
) -> Option<TerminalHyperlinkOverlay> {
    let x = row.x_for_byte(byte_start)?;
    Some(TerminalHyperlinkOverlay {
        x,
        y,
        text: text.to_string(),
        target: target.to_string(),
    })
}

fn url_regex() -> &'static Regex {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    URL_RE.get_or_init(|| Regex::new(r#"https?://[^\s<>"')\]]+"#).expect("valid URL regex"))
}

fn file_regex() -> &'static Regex {
    static FILE_RE: OnceLock<Regex> = OnceLock::new();
    FILE_RE.get_or_init(|| {
        Regex::new(
            r#"(?x)
            (?:
                [A-Za-z]:[\\/][^\s<>"'|]+
                |
                (?:\.{1,2}[\\/])?[A-Za-z0-9_.@-]+(?:[\\/][A-Za-z0-9_.@-]+)+
            )
            (?::\d+)?
            "#,
        )
        .expect("valid file regex")
    })
}

fn trim_trailing_link_punctuation(text: &str) -> &str {
    text.trim_end_matches(['.', ',', ';', ':', ')', ']', '}'])
}

fn ranges_overlap(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    left_start < right_end && right_start < left_end
}

fn file_uri_for_display_path(text: &str) -> Option<String> {
    let (without_line, line) = split_line_suffix(text);
    if !is_probable_file_reference(without_line) {
        return None;
    }
    let path = Path::new(without_line);
    let absolute = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let mut target = format!("file://{}", uri_path(&absolute));
    if let Some(line) = line {
        target.push_str(&format!("#L{line}"));
    }
    Some(target)
}

fn is_probable_file_reference(text: &str) -> bool {
    let normalized = text.replace('\\', "/");
    if normalized.starts_with("./")
        || normalized.starts_with("../")
        || is_windows_absolute_path(&normalized)
    {
        return true;
    }

    let path = Path::new(text);
    if path.exists() {
        return true;
    }

    normalized
        .rsplit('/')
        .next()
        .is_some_and(|file_name| file_name.contains('.'))
}

fn is_windows_absolute_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

fn split_line_suffix(text: &str) -> (&str, Option<u64>) {
    let Some((path, line)) = text.rsplit_once(':') else {
        return (text, None);
    };
    if !line.is_empty()
        && line.chars().all(|ch| ch.is_ascii_digit())
        && let Ok(line) = line.parse::<u64>()
    {
        (path, Some(line))
    } else {
        (text, None)
    }
}

fn uri_path(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) && !value.starts_with('/') {
        value.insert(0, '/');
    }
    value
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
}

fn sanitize_osc8_part(text: &str) -> String {
    text.replace(['\x1b', '\n', '\r'], "")
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, style::Style};

    use super::*;

    #[test]
    fn collects_url_and_file_overlays_from_rendered_buffer() {
        let area = Rect::new(0, 0, 120, 2);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(
            0,
            0,
            "See https://example.com/docs and src/dashboard/mod.rs:42",
            Style::default(),
        );

        let overlays = collect_terminal_hyperlink_overlays(&buffer, area);

        assert!(
            overlays
                .iter()
                .any(|overlay| overlay.target == "https://example.com/docs")
        );
        assert!(
            overlays
                .iter()
                .any(|overlay| overlay.text == "src/dashboard/mod.rs:42"
                    && overlay.target.starts_with("file://")
                    && overlay.target.ends_with("#L42"))
        );
    }

    #[test]
    fn file_overlay_column_uses_buffer_cells_after_wide_text() {
        let area = Rect::new(0, 0, 120, 1);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(0, 0, "ＷＩＤＥ src/dashboard/mod.rs:42", Style::default());

        let overlays = collect_terminal_hyperlink_overlays(&buffer, area);
        let overlay = overlays
            .iter()
            .find(|overlay| overlay.text == "src/dashboard/mod.rs:42")
            .expect("file path should be linked");

        assert_eq!(
            overlay.x, 9,
            "wide CJK cells before a link must not shift OSC8 overlay placement"
        );
    }

    #[test]
    fn conceptual_slash_terms_are_not_file_links() {
        let area = Rect::new(0, 0, 120, 1);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(
            0,
            0,
            "Keep model constraints in App/Event/Workflow/PendingWork concepts",
            Style::default(),
        );

        let overlays = collect_terminal_hyperlink_overlays(&buffer, area);

        assert!(
            overlays.is_empty(),
            "conceptual slash-separated labels should not become file links: {overlays:?}"
        );
    }

    #[test]
    fn slash_commands_are_not_file_links() {
        let area = Rect::new(0, 0, 120, 1);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(
            0,
            0,
            "Use /command or /status in the dashboard",
            Style::default(),
        );

        let overlays = collect_terminal_hyperlink_overlays(&buffer, area);

        assert!(
            overlays.is_empty(),
            "slash commands should not become file links: {overlays:?}"
        );
    }
}
