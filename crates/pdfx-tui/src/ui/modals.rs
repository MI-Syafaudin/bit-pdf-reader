use crate::state::modals::{ErrorModalState, JumpToPageState};
use crate::ui::file_picker::centered_rect;
use pdfx_core::document::DocumentMetadata;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render_help_modal(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(65, 75, area);
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Noble PDF Reader — Navigation & Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("──────────────────────────────────────────────────"),
        Line::from(vec![
            Span::styled(
                "Navigation:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(""),
        ]),
        Line::from("  j / ↓ / Scroll Down    : Scroll down one line"),
        Line::from("  k / ↑ / Scroll Up      : Scroll up one line"),
        Line::from("  Ctrl+d / PageDown      : Next page"),
        Line::from("  Ctrl+u / PageUp        : Previous page"),
        Line::from("  n / Next Match         : Jump to next search match"),
        Line::from("  N / Prev Match         : Jump to previous search match"),
        Line::from("  g / Home               : First page of document"),
        Line::from("  G / End                : Last page of document"),
        Line::from("  : / g                  : Jump to page modal"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Document & Views:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(""),
        ]),
        Line::from("  v / t                  : Toggle Text view / Visual preview"),
        Line::from("  b / Tab                : Toggle Sidebar (Pages / Bookmarks)"),
        Line::from("  r                      : Rotate page view clockwise 90°"),
        Line::from("  m                      : Bookmark current page"),
        Line::from("  /                      : Search text in document"),
        Line::from("  i                      : Document information & metadata"),
        Line::from("  o                      : Open new file browser"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "General:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(""),
        ]),
        Line::from("  ? / F1                 : Toggle this help popup"),
        Line::from("  Esc / q                : Close popup / Return to Home / Quit"),
        Line::from("──────────────────────────────────────────────────"),
        Line::from(vec![
            Span::styled(
                "Noble PDF Reader v0.1.0",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | Created by "),
            Span::styled(
                "Syafudin",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Powered by: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Ratatui, Crossterm, lopdf, Tesseract, SQLite",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Press [Esc] or [q] to close this help window.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help Cheatsheet ");
    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, popup_area);
}

pub fn render_jump_to_page_modal(
    f: &mut Frame,
    state: &JumpToPageState,
    total_pages: usize,
    area: Rect,
) {
    let popup_area = centered_rect(40, 25, area);
    f.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(format!("Enter target page number (1 - {}):", total_pages)),
        Line::from(""),
        Line::from(vec![
            Span::styled("Page: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                &state.input,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]),
    ];

    if let Some(ref err) = state.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            err,
            Style::default().fg(Color::Red),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Jump   [Esc] Cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Jump To Page ");
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

pub fn render_info_modal(f: &mut Frame, meta: &DocumentMetadata, path: &str, area: Rect) {
    let popup_area = centered_rect(60, 60, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(vec![Span::styled(
            "Document Details",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("──────────────────────────────────────────────────"),
        Line::from(format!("Path:         {}", path)),
        Line::from(format!("Page Count:   {}", meta.page_count)),
        Line::from(format!(
            "File Size:    {:.2} MB ({} bytes)",
            meta.file_size_bytes as f64 / (1024.0 * 1024.0),
            meta.file_size_bytes
        )),
        Line::from(format!(
            "PDF Version:  {}",
            meta.pdf_version.as_deref().unwrap_or("Unknown")
        )),
        Line::from(format!(
            "Encrypted:    {}",
            if meta.is_encrypted { "Yes" } else { "No" }
        )),
        Line::from(format!(
            "Title:        {}",
            meta.title.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Author:       {}",
            meta.author.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Subject:      {}",
            meta.subject.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Creator:      {}",
            meta.creator.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Producer:     {}",
            meta.producer.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Created:      {}",
            meta.creation_date.as_deref().unwrap_or("—")
        )),
        Line::from(format!(
            "Modified:     {}",
            meta.modification_date.as_deref().unwrap_or("—")
        )),
        Line::from("──────────────────────────────────────────────────"),
        Line::from(Span::styled(
            "Press [Esc] or [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Document Info ");
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

pub fn render_error_modal(f: &mut Frame, state: &ErrorModalState, area: Rect) {
    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(vec![Span::styled(
            &state.title,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(state.message.as_str()),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Esc] or [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Alert ")
        .border_style(Style::default().fg(Color::Red));
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}
