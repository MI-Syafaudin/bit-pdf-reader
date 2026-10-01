use pdfx_core::search::SearchMatch;

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub case_sensitive: bool,
    pub matches: Vec<SearchMatch>,
    pub active_match_index: usize,
    pub is_searching: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_results(&mut self, matches: Vec<SearchMatch>) {
        self.matches = matches;
        self.active_match_index = 0;
        self.is_searching = false;
    }

    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            None
        } else {
            self.active_match_index = (self.active_match_index + 1) % self.matches.len();
            self.matches.get(self.active_match_index)
        }
    }

    pub fn prev_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            None
        } else {
            if self.active_match_index == 0 {
                self.active_match_index = self.matches.len() - 1;
            } else {
                self.active_match_index -= 1;
            }
            self.matches.get(self.active_match_index)
        }
    }

    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.matches.get(self.active_match_index)
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.active_match_index = 0;
        self.is_searching = false;
    }
}
