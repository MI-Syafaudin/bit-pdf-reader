use crate::state::document_view::DocumentViewState;
use crate::state::file_browser::FileBrowserState;
use crate::state::modals::{ErrorModalState, JumpToPageState};
use crate::state::search::SearchState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use pdfx_core::document::DocumentMetadata;
use pdfx_core::render::RenderOptions;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::{DocumentHandle, PdfEngine};
use pdfx_storage::cache::{MemoryLruCache, PageCacheKey};
use pdfx_storage::models::{BookmarkRecord, RecentDocumentRecord};
use pdfx_storage::Database;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Home,
    Viewer,
    FileBrowser,
    Help,
    JumpToPage,
    Info,
    Error,
}

pub struct App {
    pub mode: AppMode,
    pub prev_mode: AppMode,
    pub db: Option<Database>,
    pub recent_documents: Vec<RecentDocumentRecord>,
    pub home_selected_idx: usize,

    pub current_file_path: Option<PathBuf>,
    pub current_doc_title: String,
    pub doc_handle: Option<Box<dyn DocumentHandle>>,
    pub doc_metadata: Option<DocumentMetadata>,
    pub view_state: Option<DocumentViewState>,
    pub bookmarks: Vec<BookmarkRecord>,
    pub current_doc_id: Option<i64>,

    // Caches
    pub text_cache: HashMap<usize, String>,
    pub bitmap_cache: MemoryLruCache,

    // Sub-states
    pub search_state: SearchState,
    pub file_browser: FileBrowserState,
    pub jump_modal: JumpToPageState,
    pub error_modal: ErrorModalState,

    pub should_quit: bool,
}

impl App {
    pub fn new(initial_file: Option<PathBuf>) -> Self {
        let db = Database::open_default().ok();
        let mut app = Self {
            mode: AppMode::Home,
            prev_mode: AppMode::Home,
            db,
            recent_documents: Vec::new(),
            home_selected_idx: 0,
            current_file_path: None,
            current_doc_title: "Bit PDF Reader".to_string(),
            doc_handle: None,
            doc_metadata: None,
            view_state: None,
            bookmarks: Vec::new(),
            current_doc_id: None,
            text_cache: HashMap::new(),
            bitmap_cache: MemoryLruCache::new(256), // 256MB LRU bitmap cache
            search_state: SearchState::new(),
            file_browser: FileBrowserState::new(None),
            jump_modal: JumpToPageState::default(),
            error_modal: ErrorModalState::default(),
            should_quit: false,
        };

        app.reload_recent_documents();

        if let Some(path) = initial_file {
            if path.exists() {
                app.open_document(&path);
            }
        }

        app
    }

    pub fn reload_recent_documents(&mut self) {
        if let Some(ref db) = self.db {
            if let Ok(docs) = db.get_recent_documents(50) {
                self.recent_documents = docs;
                if self.home_selected_idx >= self.recent_documents.len() {
                    self.home_selected_idx = 0;
                }
            }
        }
    }

    pub fn open_document(&mut self, path: &Path) {
        let engine = LopdfEngine::new();
        match engine.open_document(path, None) {
            Ok(handle) => {
                let count = handle.page_count();
                let meta = handle.metadata().unwrap_or_default();
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "document.pdf".to_string());

                let title = meta.title.clone().unwrap_or_else(|| filename.clone());

                // Record in database
                let mut doc_id = None;
                if let Some(ref mut db) = self.db {
                    if let Ok(id) = db.record_opened_document(
                        path,
                        &filename,
                        meta.file_size_bytes,
                        count,
                        meta.title.as_deref(),
                        meta.author.as_deref(),
                    ) {
                        doc_id = Some(id);
                    }
                }

                let mut bookmarks = Vec::new();
                if let (Some(ref db), Some(id)) = (&self.db, doc_id) {
                    if let Ok(b) = db.get_bookmarks_for_document(id) {
                        bookmarks = b;
                    }
                }

                self.current_file_path = Some(path.to_path_buf());
                self.current_doc_title = title;
                self.doc_handle = Some(handle);
                self.doc_metadata = Some(meta);
                self.view_state = Some(DocumentViewState::new(count));
                self.bookmarks = bookmarks;
                self.current_doc_id = doc_id;
                self.text_cache.clear();
                self.bitmap_cache.clear();
                self.search_state.clear();

                self.mode = AppMode::Viewer;
                self.reload_recent_documents();
                info!("Opened document: {:?}", path);
            }
            Err(e) => {
                error!("Failed to open document {:?}: {}", path, e);
                self.show_error("Open Error", &e.to_string());
            }
        }
    }

    pub fn show_error(&mut self, title: &str, message: &str) {
        self.error_modal = ErrorModalState {
            title: title.to_string(),
            message: message.to_string(),
        };
        self.prev_mode = self.mode;
        self.mode = AppMode::Error;
    }

    pub fn get_current_page_text(&mut self) -> String {
        let page = match self.view_state {
            Some(ref vs) => vs.current_page,
            None => return String::new(),
        };

        if let Some(text) = self.text_cache.get(&page) {
            return text.clone();
        }

        if let Some(ref handle) = self.doc_handle {
            match handle.extract_text(page) {
                Ok(text) => {
                    self.text_cache.insert(page, text.clone());
                    text
                }
                Err(e) => format!("[Text Extraction Failed: {}]", e),
            }
        } else {
            String::new()
        }
    }

    pub fn get_current_page_bitmap(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<pdfx_core::render::RenderedBitmap> {
        let vs = self.view_state.as_ref()?;
        let (page, rot) = (vs.current_page, vs.rotation_offset);
        let path_str = self
            .current_file_path
            .as_ref()?
            .to_string_lossy()
            .to_string();

        let key = PageCacheKey {
            document_path: path_str,
            page_number: page,
            width,
            height,
        };

        if let Some(bmp) = self.bitmap_cache.get(&key) {
            return Some(bmp);
        }

        if let Some(ref handle) = self.doc_handle {
            let opts = RenderOptions {
                target_width: width.max(100),
                target_height: height.max(100),
                rotate_degrees: rot,
                ..Default::default()
            };

            if let Ok(bmp) = handle.render_page(page, &opts) {
                self.bitmap_cache.put(key, bmp.clone());
                return Some(bmp);
            }
        }

        None
    }

    pub fn add_bookmark_for_current_page(&mut self) {
        let page = match self.view_state {
            Some(ref vs) => vs.current_page,
            None => return,
        };

        if let (Some(ref mut db), Some(doc_id)) = (&mut self.db, self.current_doc_id) {
            let title = format!("Page {}", page);
            if let Ok(_id) = db.add_bookmark(doc_id, page, &title) {
                if let Ok(b) = db.get_bookmarks_for_document(doc_id) {
                    self.bookmarks = b;
                    debug!("Added bookmark for page {}", page);
                }
            }
        }
    }

    pub fn perform_search(&mut self) {
        let query = self.search_state.query.trim().to_string();
        if query.is_empty() {
            self.search_state.matches.clear();
            return;
        }

        let total = match self.view_state {
            Some(ref vs) => vs.total_pages,
            None => return,
        };

        let mut all_matches = Vec::new();
        if let Some(ref handle) = self.doc_handle {
            for p in 1..=total {
                if let Ok(matches) = handle.search_text(p, &query, self.search_state.case_sensitive)
                {
                    all_matches.extend(matches);
                }
            }
        }

        let first_page = all_matches.first().map(|m| m.page_number);
        self.search_state.set_results(all_matches);

        if let (Some(ref mut vs), Some(target_p)) = (&mut self.view_state, first_page) {
            vs.goto_page(target_p);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::Home => self.handle_home_key(key),
            AppMode::Viewer => self.handle_viewer_key(key),
            AppMode::FileBrowser => self.handle_file_browser_key(key),
            AppMode::JumpToPage => self.handle_jump_key(key),
            AppMode::Help => self.handle_help_key(key),
            AppMode::Info => self.handle_info_key(key),
            AppMode::Error => self.handle_error_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.recent_documents.is_empty() {
                    self.home_selected_idx =
                        (self.home_selected_idx + 1) % self.recent_documents.len();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.recent_documents.is_empty() {
                    if self.home_selected_idx == 0 {
                        self.home_selected_idx = self.recent_documents.len() - 1;
                    } else {
                        self.home_selected_idx -= 1;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char('r') => {
                if let Some(doc) = self.recent_documents.get(self.home_selected_idx).cloned() {
                    self.open_document(&doc.path);
                }
            }
            KeyCode::Char('o') => {
                self.file_browser = FileBrowserState::new(None);
                self.prev_mode = AppMode::Home;
                self.mode = AppMode::FileBrowser;
            }
            KeyCode::Char('d') => {
                if let Some(doc) = self.recent_documents.get(self.home_selected_idx).cloned() {
                    if let Some(ref mut db) = self.db {
                        let _ = db.remove_recent_document(doc.id);
                        self.reload_recent_documents();
                    }
                }
            }
            KeyCode::Char('?') => {
                self.prev_mode = AppMode::Home;
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }

    fn handle_viewer_key(&mut self, key: KeyEvent) {
        // If typing in search bar
        if self.search_state.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.search_state.is_searching = false;
                }
                KeyCode::Enter => {
                    self.perform_search();
                }
                KeyCode::Backspace => {
                    self.search_state.query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_state.query.push(c);
                }
                _ => {}
            }
            return;
        }

        // Standard viewer keys
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.mode = AppMode::Home;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref mut vs) = self.view_state {
                    vs.scroll_down(1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref mut vs) = self.view_state {
                    vs.scroll_up(1);
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(ref mut vs) = self.view_state {
                    vs.scroll_left(2);
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(ref mut vs) = self.view_state {
                    vs.scroll_right(2);
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut vs) = self.view_state {
                    vs.next_page();
                }
            }
            KeyCode::PageDown | KeyCode::Char(' ') => {
                if let Some(ref mut vs) = self.view_state {
                    vs.next_page();
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut vs) = self.view_state {
                    vs.prev_page();
                }
            }
            KeyCode::PageUp => {
                if let Some(ref mut vs) = self.view_state {
                    vs.prev_page();
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                if let Some(ref mut vs) = self.view_state {
                    vs.first_page();
                }
            }
            KeyCode::Char('G') | KeyCode::End => {
                if let Some(ref mut vs) = self.view_state {
                    vs.last_page();
                }
            }
            KeyCode::Char(':') => {
                self.jump_modal = JumpToPageState::default();
                self.prev_mode = AppMode::Viewer;
                self.mode = AppMode::JumpToPage;
            }
            KeyCode::Char('/') => {
                self.search_state.is_searching = true;
                self.search_state.query.clear();
            }
            KeyCode::Char('n') => {
                if let Some(m) = self.search_state.next_match() {
                    let target_page = m.page_number;
                    if let Some(ref mut vs) = self.view_state {
                        vs.goto_page(target_page);
                    }
                }
            }
            KeyCode::Char('N') => {
                if let Some(m) = self.search_state.prev_match() {
                    let target_page = m.page_number;
                    if let Some(ref mut vs) = self.view_state {
                        vs.goto_page(target_page);
                    }
                }
            }
            KeyCode::Char('v') | KeyCode::Char('t') => {
                if let Some(ref mut vs) = self.view_state {
                    vs.toggle_view_mode();
                }
            }
            KeyCode::Char('b') | KeyCode::Tab => {
                if let Some(ref mut vs) = self.view_state {
                    vs.toggle_sidebar();
                }
            }
            KeyCode::Char('T') => {
                if let Some(ref mut vs) = self.view_state {
                    vs.toggle_sidebar_tab();
                }
            }
            KeyCode::Char('r') => {
                if let Some(ref mut vs) = self.view_state {
                    vs.rotate_clockwise();
                }
            }
            KeyCode::Char('m') => {
                self.add_bookmark_for_current_page();
            }
            KeyCode::Char('i') => {
                self.prev_mode = AppMode::Viewer;
                self.mode = AppMode::Info;
            }
            KeyCode::Char('o') => {
                self.file_browser = FileBrowserState::new(None);
                self.prev_mode = AppMode::Viewer;
                self.mode = AppMode::FileBrowser;
            }
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.prev_mode = AppMode::Viewer;
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }

    fn handle_file_browser_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = self.prev_mode;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.file_browser.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.file_browser.select_prev();
            }
            KeyCode::Backspace => {
                self.file_browser.go_up();
            }
            KeyCode::Enter => {
                if let Some(selected_path) = self.file_browser.enter_selected() {
                    self.open_document(&selected_path);
                }
            }
            _ => {}
        }
    }

    fn handle_jump_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = self.prev_mode;
            }
            KeyCode::Backspace => {
                self.jump_modal.input.pop();
                self.jump_modal.error = None;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.jump_modal.input.push(c);
                self.jump_modal.error = None;
            }
            KeyCode::Enter => {
                if let Ok(p) = self.jump_modal.input.trim().parse::<usize>() {
                    let total = self
                        .view_state
                        .as_ref()
                        .map(|vs| vs.total_pages)
                        .unwrap_or(0);
                    if p >= 1 && p <= total {
                        if let Some(ref mut vs) = self.view_state {
                            vs.goto_page(p);
                        }
                        self.mode = self.prev_mode;
                    } else {
                        self.jump_modal.error =
                            Some(format!("Page must be between 1 and {}", total));
                    }
                } else {
                    self.jump_modal.error = Some("Invalid number".to_string());
                }
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char('?') => {
                self.mode = self.prev_mode;
            }
            _ => {}
        }
    }

    fn handle_info_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char('i') => {
                self.mode = self.prev_mode;
            }
            _ => {}
        }
    }

    fn handle_error_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = self.prev_mode;
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.mode == AppMode::Viewer {
            match mouse.kind {
                MouseEventKind::ScrollDown => {
                    if let Some(ref mut vs) = self.view_state {
                        vs.scroll_down(3);
                    }
                }
                MouseEventKind::ScrollUp => {
                    if let Some(ref mut vs) = self.view_state {
                        vs.scroll_up(3);
                    }
                }
                _ => {}
            }
        }
    }
}
