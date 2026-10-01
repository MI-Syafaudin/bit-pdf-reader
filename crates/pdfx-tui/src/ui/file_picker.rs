use crate::state::file_browser::FileBrowserState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_file_picker(f: &mut Frame, state: &FileBrowserState, area: Rect) {
    let popup_area = centered_rect(70, 70, area);
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Path header
            Constraint::Min(0),    // File list
            Constraint::Length(2), // Help footer
        ])
        .split(popup_area);

    let path_str = state.current_dir.to_string_lossy();
    let header = Paragraph::new(format!(" Directory: {}", path_str))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Open PDF File "),
        );
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = state
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == state.selected_index;
            let icon = if entry.is_dir { "📁 " } else { "📄 " };
            let prefix = if is_selected { "▶ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let size_str = if !entry.is_dir {
                format!(" ({:.1} KB)", entry.size_bytes as f64 / 1024.0)
            } else {
                String::new()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::raw(icon),
                Span::styled(&entry.name, style),
                Span::styled(size_str, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));
    f.render_widget(list, chunks[1]);

    let footer = Paragraph::new(
        " [Enter] Open / Enter dir   [Backspace] Parent Dir   [↑/↓/j/k] Navigate   [Esc] Cancel",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
