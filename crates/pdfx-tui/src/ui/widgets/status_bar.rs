use crate::state::document_view::{DocumentViewState, ViewMode};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub struct StatusBarWidget<'a> {
    pub view_state: &'a DocumentViewState,
    pub title: &'a str,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(view_state: &'a DocumentViewState, title: &'a str) -> Self {
        Self { view_state, title }
    }
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mode_label = match self.view_state.view_mode {
            ViewMode::Text => "TEXT",
            ViewMode::Visual => "VISUAL",
        };

        let mode_color = match self.view_state.view_mode {
            ViewMode::Text => Color::Cyan,
            ViewMode::Visual => Color::Green,
        };

        let page_info = format!(
            " Page {}/{} ",
            self.view_state.current_page, self.view_state.total_pages
        );

        let rot_info = if self.view_state.rotation_offset > 0 {
            format!(" {}° ", self.view_state.rotation_offset)
        } else {
            String::new()
        };

        let left_spans = vec![
            Span::styled(
                format!(" {} ", mode_label),
                Style::default()
                    .bg(mode_color)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                page_info,
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                rot_info,
                Style::default().bg(Color::DarkGray).fg(Color::Yellow),
            ),
            Span::raw(" "),
            Span::styled(
                self.title,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::ITALIC),
            ),
        ];

        let right_spans = vec![
            Span::styled("[?]", Style::default().fg(Color::Yellow)),
            Span::raw("Help "),
            Span::styled("[/]", Style::default().fg(Color::Yellow)),
            Span::raw("Search "),
            Span::styled("[b]", Style::default().fg(Color::Yellow)),
            Span::raw("Sidebar "),
            Span::styled("[v]", Style::default().fg(Color::Yellow)),
            Span::raw("Mode "),
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw("Back "),
        ];

        let left_line = Line::from(left_spans);
        let right_line = Line::from(right_spans);

        // Render background bar
        let bar_style = Style::default().bg(Color::Rgb(30, 30, 35)).fg(Color::Gray);
        buf.set_style(area, bar_style);

        // Render left part
        Paragraph::new(left_line).render(area, buf);

        // Render right part right-aligned
        let right_width = right_line.width() as u16;
        if area.width > right_width {
            let right_rect = Rect {
                x: area.x + area.width - right_width,
                y: area.y,
                width: right_width,
                height: 1,
            };
            Paragraph::new(right_line).render(right_rect, buf);
        }
    }
}
