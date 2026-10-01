use pdfx_storage::models::RecentDocumentRecord;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_home(
    f: &mut Frame,
    recent_docs: &[RecentDocumentRecord],
    selected_idx: usize,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Header
    let header_line = Line::from(vec![
        Span::styled(
            " PDFX ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " High-Performance Terminal PDF Engine for Linux ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let header = Paragraph::new(header_line)
        .block(Block::default().borders(Borders::ALL).title(" Overview "));
    f.render_widget(header, chunks[0]);

    // Split main content into Left (Recent Docs) and Right (Quick Actions & Details)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    // Recent Documents List
    let items: Vec<ListItem> = if recent_docs.is_empty() {
        vec![ListItem::new(
            "  No recent documents found. Press [O] to browse and open a PDF.",
        )]
    } else {
        recent_docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let is_selected = i == selected_idx;
                let prefix = if is_selected { "▶ " } else { "  " };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(&doc.filename, style),
                    Span::styled(
                        format!(
                            " ({} pgs, {})",
                            doc.page_count,
                            format_size(doc.file_size_bytes)
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Recent Documents ({}) ", recent_docs.len())),
    );
    f.render_widget(list, main_chunks[0]);

    // Right panel: Quick Actions & Document info
    let selected_doc = recent_docs.get(selected_idx);
    let mut right_lines = vec![
        Line::from(vec![Span::styled(
            "Quick Actions:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [O] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Open File Browser"),
        ]),
        Line::from(vec![
            Span::styled(
                " [Enter] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Open Selected PDF"),
        ]),
        Line::from(vec![
            Span::styled(
                " [D] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Remove From Recent"),
        ]),
        Line::from(vec![
            Span::styled(
                " [?] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Help Cheatsheet"),
        ]),
        Line::from(vec![
            Span::styled(
                " [Q] ",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
            Span::raw(" Exit Application"),
        ]),
        Line::from(""),
        Line::from("───────────────────────────────────"),
        Line::from(""),
    ];

    if let Some(doc) = selected_doc {
        right_lines.push(Line::from(vec![Span::styled(
            "Selected Document:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]));
        right_lines.push(Line::from(vec![
            Span::styled("Name:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(&doc.filename, Style::default().fg(Color::White)),
        ]));
        right_lines.push(Line::from(vec![
            Span::styled("Path:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                doc.path.to_string_lossy().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        right_lines.push(Line::from(vec![
            Span::styled("Pages: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                doc.page_count.to_string(),
                Style::default().fg(Color::White),
            ),
        ]));
        right_lines.push(Line::from(vec![
            Span::styled("Size:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_size(doc.file_size_bytes),
                Style::default().fg(Color::White),
            ),
        ]));
        right_lines.push(Line::from(vec![
            Span::styled("Last:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                doc.last_opened_at.as_str(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let actions = Paragraph::new(right_lines)
        .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
    f.render_widget(actions, main_chunks[1]);

    // Footer
    let footer_line = Line::from(vec![
        Span::styled(" Navigation: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[↑/↓/j/k] Select   [Enter] Open   [O] File Browser   [?] Help   [Q] Quit",
            Style::default().fg(Color::Gray),
        ),
    ]);
    let footer = Paragraph::new(footer_line);
    f.render_widget(footer, chunks[2]);
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
