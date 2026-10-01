#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Text,
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Pages,
    Bookmarks,
}

#[derive(Debug, Clone)]
pub struct DocumentViewState {
    pub current_page: usize,
    pub total_pages: usize,
    pub zoom: f32,
    pub rotation_offset: u16,
    pub view_mode: ViewMode,
    pub sidebar_open: bool,
    pub sidebar_tab: SidebarTab,
    pub scroll_y: usize,
    pub scroll_x: usize,
}

impl DocumentViewState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: if total_pages > 0 { 1 } else { 0 },
            total_pages,
            zoom: 100.0,
            rotation_offset: 0,
            view_mode: ViewMode::Text,
            sidebar_open: true,
            sidebar_tab: SidebarTab::Pages,
            scroll_y: 0,
            scroll_x: 0,
        }
    }

    pub fn next_page(&mut self) -> bool {
        if self.current_page < self.total_pages {
            self.current_page += 1;
            self.scroll_y = 0;
            self.scroll_x = 0;
            true
        } else {
            false
        }
    }

    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 1 {
            self.current_page -= 1;
            self.scroll_y = 0;
            self.scroll_x = 0;
            true
        } else {
            false
        }
    }

    pub fn goto_page(&mut self, page: usize) {
        if page >= 1 && page <= self.total_pages {
            self.current_page = page;
            self.scroll_y = 0;
            self.scroll_x = 0;
        }
    }

    pub fn first_page(&mut self) {
        self.goto_page(1);
    }

    pub fn last_page(&mut self) {
        self.goto_page(self.total_pages);
    }

    pub fn rotate_clockwise(&mut self) {
        self.rotation_offset = (self.rotation_offset + 90) % 360;
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Text => ViewMode::Visual,
            ViewMode::Visual => ViewMode::Text,
        };
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_open = !self.sidebar_open;
    }

    pub fn toggle_sidebar_tab(&mut self) {
        self.sidebar_tab = match self.sidebar_tab {
            SidebarTab::Pages => SidebarTab::Bookmarks,
            SidebarTab::Bookmarks => SidebarTab::Pages,
        };
    }

    pub fn scroll_down(&mut self, delta: usize) {
        self.scroll_y = self.scroll_y.saturating_add(delta);
    }

    pub fn scroll_up(&mut self, delta: usize) {
        self.scroll_y = self.scroll_y.saturating_sub(delta);
    }

    pub fn scroll_right(&mut self, delta: usize) {
        self.scroll_x = self.scroll_x.saturating_add(delta);
    }

    pub fn scroll_left(&mut self, delta: usize) {
        self.scroll_x = self.scroll_x.saturating_sub(delta);
    }
}
