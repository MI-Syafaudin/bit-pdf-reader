pub use crate::metadata::{DocumentMetadata, DocumentPermissions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentSummary {
    pub path: PathBuf,
    pub filename: String,
    pub page_count: usize,
    pub file_size_bytes: u64,
    pub is_encrypted: bool,
    pub title: Option<String>,
    pub author: Option<String>,
}

impl DocumentSummary {
    pub fn new(path: PathBuf, metadata: &DocumentMetadata) -> Self {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document.pdf".to_string());

        Self {
            path,
            filename,
            page_count: metadata.page_count,
            file_size_bytes: metadata.file_size_bytes,
            is_encrypted: metadata.is_encrypted,
            title: metadata.title.clone(),
            author: metadata.author.clone(),
        }
    }
}

/// Outline / Bookmark item in the document's table of contents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutlineItem {
    pub title: String,
    pub page_number: usize,
    pub children: Vec<OutlineItem>,
}

impl OutlineItem {
    pub fn new(title: impl Into<String>, page_number: usize) -> Self {
        Self {
            title: title.into(),
            page_number,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: OutlineItem) {
        self.children.push(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outline_item_hierarchy() {
        let mut chapter1 = OutlineItem::new("Chapter 1", 1);
        chapter1.add_child(OutlineItem::new("Section 1.1", 2));
        chapter1.add_child(OutlineItem::new("Section 1.2", 4));

        assert_eq!(chapter1.children.len(), 2);
        assert_eq!(chapter1.children[0].title, "Section 1.1");
        assert_eq!(chapter1.children[0].page_number, 2);
    }
}
