use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentDocumentRecord {
    pub id: i64,
    pub path: PathBuf,
    pub filename: String,
    pub page_count: usize,
    pub file_size_bytes: u64,
    pub last_opened_at: String,
    pub last_page_viewed: usize,
    pub zoom_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookmarkRecord {
    pub id: i64,
    pub document_id: i64,
    pub page_number: usize,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub theme: String, // "dark", "light", "monochrome"
    pub confirm_destructive: bool,
    pub max_cache_mb: usize,
    pub max_parallel_jobs: usize,
    pub default_dpi: f32,
    pub ocr_language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            confirm_destructive: true,
            max_cache_mb: 256,
            max_parallel_jobs: 4,
            default_dpi: 150.0,
            ocr_language: "eng".to_string(),
        }
    }
}
