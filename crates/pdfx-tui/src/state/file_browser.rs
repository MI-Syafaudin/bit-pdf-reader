use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct FileBrowserState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub error_message: Option<String>,
}

impl FileBrowserState {
    pub fn new(initial_dir: Option<&Path>) -> Self {
        let current_dir = initial_dir
            .map(|p| {
                if p.is_dir() {
                    p.to_path_buf()
                } else {
                    p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
                }
            })
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut state = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            error_message: None,
        };
        state.reload();
        state
    }

    pub fn reload(&mut self) {
        self.entries.clear();
        self.selected_index = 0;
        self.error_message = None;

        // If not at root, add ".." parent directory entry
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
                size_bytes: 0,
            });
        }

        match fs::read_dir(&self.current_dir) {
            Ok(read_dir) => {
                let mut dirs = Vec::new();
                let mut files = Vec::new();

                for entry in read_dir.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Hide hidden files (starting with dot)
                    if name.starts_with('.') {
                        continue;
                    }

                    if let Ok(meta) = entry.metadata() {
                        if meta.is_dir() {
                            dirs.push(FileEntry {
                                path,
                                name,
                                is_dir: true,
                                size_bytes: 0,
                            });
                        } else if name.to_lowercase().ends_with(".pdf") {
                            files.push(FileEntry {
                                path,
                                name,
                                is_dir: false,
                                size_bytes: meta.len(),
                            });
                        }
                    }
                }

                dirs.sort_by_key(|a| a.name.to_lowercase());
                files.sort_by_key(|a| a.name.to_lowercase());

                self.entries.extend(dirs);
                self.entries.extend(files);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to read directory: {}", e));
            }
        }
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.entries.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.entries.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.entries.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.selected_entry().cloned() {
            if entry.is_dir {
                self.current_dir = entry.path;
                self.reload();
                None
            } else {
                Some(entry.path)
            }
        } else {
            None
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.reload();
        }
    }
}
