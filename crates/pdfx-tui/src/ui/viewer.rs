use crate::state::document_view::{DocumentViewState, SidebarTab, ViewMode};
use crate::state::search::SearchState;
use crate::ui::widgets::{HalfBlockWidget, StatusBarWidget};
use pdfx_core::render::RenderedBitmap;
use pdfx_storage::models::BookmarkRecord;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub struct ViewerData<'a> {
    pub view_state: &'a DocumentViewState,
    pub page_text: &'a str,
    pub bitmap: Option<&'a RenderedBitmap>,
    pub bookmarks: &'a [BookmarkRecord],
    pub search_state: &'a SearchState,
    pub filename: &'a str,
}

pub fn render_viewer(f: &mut Frame, data: &ViewerData, area: Rect) {
    let main_constraints = if data.search_state.is_searching {
        vec![
            Constraint::Min(0),    // Viewer content
            Constraint::Length(3), // Search bar
            Constraint::Length(1), // Status bar
        ]
    } else {
        vec![
            Constraint::Min(0),    // Viewer content
            Constraint::Length(1), // Status bar
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(main_constraints)
        .split(area);

    let content_area = chunks[0];

    // Split content area between Sidebar (if enabled) and Page Viewer
    let (sidebar_area, viewer_area) = if data.view_state.sidebar_open {
        let sub = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Min(0)])
            .split(content_area);
        (Some(sub[0]), sub[1])
    } else {
        (None, content_area)
    };

    // Render Sidebar if open
    if let Some(sb_area) = sidebar_area {
        render_sidebar(f, data.view_state, data.bookmarks, sb_area);
    }

    // Render Main Page Area (Text or Visual)
    match data.view_state.view_mode {
        ViewMode::Text => {
            render_text_page(
                f,
                data.page_text,
                data.view_state,
                data.search_state,
                viewer_area,
            );
        }
        ViewMode::Visual => {
            if let Some(bmp) = data.bitmap {
                let block = Block::default().borders(Borders::ALL).title(format!(
                    " Page {} Preview (Visual) ",
                    data.view_state.current_page
                ));
                let inner = block.inner(viewer_area);
                f.render_widget(block, viewer_area);
                let visual_widget = HalfBlockWidget::new(bmp);
                f.render_widget(visual_widget, inner);
            } else {
                let msg = Paragraph::new(
                    "Rendering page preview...\nPress [v] or [t] to toggle Text mode.",
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Page {} Preview ", data.view_state.current_page)),
                );
                f.render_widget(msg, viewer_area);
            }
        }
    }

    // Search bar if searching
    if data.search_state.is_searching {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(" Find in Document ");
        let count_info = if !data.search_state.matches.is_empty() {
            format!(
                " [{}/{}] ",
                data.search_state.active_match_index + 1,
                data.search_state.matches.len()
            )
        } else if !data.search_state.query.is_empty() {
            " [No matches] ".to_string()
        } else {
            String::new()
        };

        let search_text = format!("Search: {}{}", data.search_state.query, count_info);
        let search_p = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Yellow))
            .block(search_block);
        f.render_widget(search_p, chunks[1]);
        let status_rect = chunks[2];
        let status_bar = StatusBarWidget::new(data.view_state, data.filename);
        f.render_widget(status_bar, status_rect);
    } else {
        let status_rect = chunks[1];
        let status_bar = StatusBarWidget::new(data.view_state, data.filename);
        f.render_widget(status_bar, status_rect);
    }
}

fn render_sidebar(
    f: &mut Frame,
    view_state: &DocumentViewState,
    bookmarks: &[BookmarkRecord],
    area: Rect,
) {
    let title = match view_state.sidebar_tab {
        SidebarTab::Pages => " Pages [Tab] ",
        SidebarTab::Bookmarks => " Bookmarks [Tab] ",
    };

    let block = Block::default().borders(Borders::ALL).title(title);

    match view_state.sidebar_tab {
        SidebarTab::Pages => {
            let mut items = Vec::new();
            for p in 1..=view_state.total_pages {
                let is_current = p == view_state.current_page;
                let is_bookmarked = bookmarks.iter().any(|b| b.page_number == p);

                let marker = if is_current { "▶ " } else { "  " };
                let star = if is_bookmarked { "★ " } else { "  " };

                let style = if is_current {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(marker, style),
                    Span::styled(star, Style::default().fg(Color::Yellow)),
                    Span::styled(format!("Page {:>3}", p), style),
                ]);

                items.push(ListItem::new(line));
            }

            let list = List::new(items).block(block);
            f.render_widget(list, area);
        }
        SidebarTab::Bookmarks => {
            let items: Vec<ListItem> = if bookmarks.is_empty() {
                vec![ListItem::new("  No bookmarks yet.\n  Press [m] to add.")]
            } else {
                bookmarks
                    .iter()
                    .map(|b| {
                        let is_current = b.page_number == view_state.current_page;
                        let style = if is_current {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        let line = Line::from(vec![
                            Span::styled("★ ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("p.{} - {}", b.page_number, b.title), style),
                        ]);
                        ListItem::new(line)
                    })
                    .collect()
            };

            let list = List::new(items).block(block);
            f.render_widget(list, area);
        }
    }
}

fn render_text_page(
    f: &mut Frame,
    text: &str,
    view_state: &DocumentViewState,
    search_state: &SearchState,
    area: Rect,
) {
    let block = Block::default().borders(Borders::ALL).title(format!(
        " Page {}/{} (Text View) ",
        view_state.current_page, view_state.total_pages
    ));

    let query_lower = search_state.query.to_lowercase();
    let is_query_active = !query_lower.is_empty();

    let lines: Vec<Line> = text
        .lines()
        .map(|line| {
            if is_query_active && line.to_lowercase().contains(&query_lower) {
                // Highlight line containing search match
                Line::from(vec![
                    Span::styled("▌ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(Span::raw(line))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((view_state.scroll_y as u16, view_state.scroll_x as u16));

    f.render_widget(paragraph, area);
}
