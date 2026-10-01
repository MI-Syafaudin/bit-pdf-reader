use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use pdfx_core::render::RenderedBitmap;
use pdfx_core::search::SearchMatch;
use pdfx_tui::app::{App, AppMode};
use pdfx_tui::state::document_view::{DocumentViewState, SidebarTab, ViewMode};
use pdfx_tui::state::file_browser::FileBrowserState;
use pdfx_tui::state::search::SearchState;
use pdfx_tui::ui::widgets::HalfBlockWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::path::Path;
use tempfile::tempdir;

fn create_sample_pdf(path: &Path, pages_count: usize) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut page_ids = Vec::new();
    for i in 1..=pages_count {
        let text = format!("BT /F1 14 Tf 72 700 Td (Page Text {}) Tj ET", i);
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            text.into_bytes(),
        )));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        page_ids.push(page_id);
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => pages_count as u32,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(path)?;
    Ok(())
}

#[test]
fn test_document_view_state_navigation() {
    let mut vs = DocumentViewState::new(5);
    assert_eq!(vs.current_page, 1);
    assert_eq!(vs.total_pages, 5);

    // Forward navigation
    assert!(vs.next_page());
    assert_eq!(vs.current_page, 2);
    assert!(vs.next_page());
    assert_eq!(vs.current_page, 3);

    // Backward navigation
    assert!(vs.prev_page());
    assert_eq!(vs.current_page, 2);

    // Jump navigation
    vs.last_page();
    assert_eq!(vs.current_page, 5);
    assert!(!vs.next_page()); // At boundary

    vs.first_page();
    assert_eq!(vs.current_page, 1);
    assert!(!vs.prev_page()); // At boundary

    // Direct goto
    vs.goto_page(4);
    assert_eq!(vs.current_page, 4);

    // Rotation
    assert_eq!(vs.rotation_offset, 0);
    vs.rotate_clockwise();
    assert_eq!(vs.rotation_offset, 90);
    vs.rotate_clockwise();
    assert_eq!(vs.rotation_offset, 180);
    vs.rotate_clockwise();
    assert_eq!(vs.rotation_offset, 270);
    vs.rotate_clockwise();
    assert_eq!(vs.rotation_offset, 0);

    // View mode toggle
    assert_eq!(vs.view_mode, ViewMode::Text);
    vs.toggle_view_mode();
    assert_eq!(vs.view_mode, ViewMode::Visual);
    vs.toggle_view_mode();
    assert_eq!(vs.view_mode, ViewMode::Text);

    // Sidebar toggles
    assert!(vs.sidebar_open);
    vs.toggle_sidebar();
    assert!(!vs.sidebar_open);
    vs.toggle_sidebar();
    assert!(vs.sidebar_open);

    assert_eq!(vs.sidebar_tab, SidebarTab::Pages);
    vs.toggle_sidebar_tab();
    assert_eq!(vs.sidebar_tab, SidebarTab::Bookmarks);
}

#[test]
fn test_search_state() {
    let mut ss = SearchState::new();
    assert!(ss.matches.is_empty());

    let matches = vec![
        SearchMatch {
            page_number: 1,
            match_text: "foo".to_string(),
            context_snippet: "test foo".to_string(),
        },
        SearchMatch {
            page_number: 3,
            match_text: "foo".to_string(),
            context_snippet: "another foo".to_string(),
        },
    ];

    ss.set_results(matches);
    assert_eq!(ss.matches.len(), 2);
    assert_eq!(ss.current_match().unwrap().page_number, 1);

    // Next match
    let m = ss.next_match().unwrap();
    assert_eq!(m.page_number, 3);

    // Wrap around
    let m = ss.next_match().unwrap();
    assert_eq!(m.page_number, 1);

    // Prev match
    let m = ss.prev_match().unwrap();
    assert_eq!(m.page_number, 3);

    ss.clear();
    assert!(ss.matches.is_empty());
}

#[test]
fn test_file_browser_state() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    let pdf1 = dir.path().join("doc1.pdf");
    create_sample_pdf(&pdf1, 1).unwrap();

    let mut fb = FileBrowserState::new(Some(dir.path()));
    assert!(!fb.entries.is_empty());

    // Navigation
    let initial_idx = fb.selected_index;
    fb.select_next();
    assert_ne!(initial_idx, fb.selected_index);
    fb.select_prev();
    assert_eq!(initial_idx, fb.selected_index);
}

#[test]
fn test_app_lifecycle_and_keys() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("sample.pdf");
    create_sample_pdf(&pdf, 3).unwrap();

    let mut app = App::new(Some(pdf.clone()));
    assert_eq!(app.mode, AppMode::Viewer);
    assert!(app.doc_handle.is_some());
    assert_eq!(app.view_state.as_ref().unwrap().total_pages, 3);
    assert_eq!(app.view_state.as_ref().unwrap().current_page, 1);

    // Test text extraction and cache
    let text = app.get_current_page_text();
    assert!(text.contains("Page Text 1"));
    assert!(app.text_cache.contains_key(&1));

    // Test bitmap rendering fallback and cache
    let bmp = app.get_current_page_bitmap(100, 100);
    assert!(bmp.is_some());

    // Key events: Next page
    app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
    assert_eq!(app.view_state.as_ref().unwrap().current_page, 2);

    // Key events: Prev page
    app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    assert_eq!(app.view_state.as_ref().unwrap().current_page, 1);

    // Key events: Rotate
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
    assert_eq!(app.view_state.as_ref().unwrap().rotation_offset, 90);

    // Key events: Toggle view mode
    app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE));
    assert_eq!(app.view_state.as_ref().unwrap().view_mode, ViewMode::Visual);

    // Key events: Add bookmark
    app.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));

    // Key events: Jump to page modal
    app.handle_key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
    assert_eq!(app.mode, AppMode::JumpToPage);

    // Type page 3 and enter
    app.handle_key(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.mode, AppMode::Viewer);
    assert_eq!(app.view_state.as_ref().unwrap().current_page, 3);

    // Return to Home
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.mode, AppMode::Home);

    // Quit from Home
    app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert!(app.should_quit);
}

#[test]
fn test_half_block_widget_render() {
    let mut data = vec![255; 100 * 100 * 4];
    data[0] = 200;
    data[1] = 100;
    data[2] = 50;

    let bmp = RenderedBitmap::new(100, 100, data);
    let widget = HalfBlockWidget::new(&bmp);

    let area = Rect::new(0, 0, 20, 10);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);

    let cell = buf.cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), "▀");
}
