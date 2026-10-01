pub mod app;
pub mod events;
pub mod state;
pub mod ui;

use app::{App, AppMode};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{AppEvent, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

pub fn run_tui(target_file: Option<PathBuf>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(target_file);
    let event_handler = EventHandler::new(50);

    let res = run_loop(&mut terminal, &mut app, &event_handler);

    event_handler.stop();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI Error: {err:?}");
    }

    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> io::Result<()> {
    while !app.should_quit {
        // Collect page text and bitmap if in viewer
        let (current_text, current_bitmap) = if app.doc_handle.is_some() && app.view_state.is_some()
        {
            let text = app.get_current_page_text();
            let bmp = if app.view_state.as_ref().map(|vs| vs.view_mode)
                == Some(state::document_view::ViewMode::Visual)
            {
                app.get_current_page_bitmap(800, 1000)
            } else {
                None
            };
            (text, bmp)
        } else {
            (String::new(), None)
        };

        terminal.draw(|f| {
            let area = f.area();

            // Render primary base screen
            match app.mode {
                AppMode::Home => {
                    ui::render_home(f, &app.recent_documents, app.home_selected_idx, area);
                }
                _ => {
                    if let Some(ref vs) = app.view_state {
                        let data = ui::ViewerData {
                            view_state: vs,
                            page_text: &current_text,
                            bitmap: current_bitmap.as_ref(),
                            bookmarks: &app.bookmarks,
                            search_state: &app.search_state,
                            filename: &app.current_doc_title,
                        };
                        ui::render_viewer(f, &data, area);
                    } else {
                        ui::render_home(f, &app.recent_documents, app.home_selected_idx, area);
                    }
                }
            }

            // Render modal overlays on top of base screen
            match app.mode {
                AppMode::FileBrowser => {
                    ui::render_file_picker(f, &app.file_browser, area);
                }
                AppMode::Help => {
                    ui::render_help_modal(f, area);
                }
                AppMode::JumpToPage => {
                    let total = app
                        .view_state
                        .as_ref()
                        .map(|vs| vs.total_pages)
                        .unwrap_or(1);
                    ui::render_jump_to_page_modal(f, &app.jump_modal, total, area);
                }
                AppMode::Info => {
                    if let Some(ref meta) = app.doc_metadata {
                        let path_str = app
                            .current_file_path
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        ui::render_info_modal(f, meta, &path_str, area);
                    }
                }
                AppMode::Error => {
                    ui::render_error_modal(f, &app.error_modal, area);
                }
                _ => {}
            }
        })?;

        match events.receiver.recv() {
            Ok(AppEvent::Key(key)) => app.handle_key(key),
            Ok(AppEvent::Mouse(mouse)) => app.handle_mouse(mouse),
            Ok(AppEvent::Tick) => {}
            Ok(AppEvent::Resize(_, _)) => {}
            Err(_) => break,
        }
    }

    Ok(())
}
