use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocumentPermissions {
    pub can_print: bool,
    pub can_modify: bool,
    pub can_copy: bool,
    pub can_annotate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub pdf_version: Option<String>,
    pub is_encrypted: bool,
    pub page_count: usize,
    pub file_size_bytes: u64,
    pub permissions: DocumentPermissions,
}

impl DocumentMetadata {
    pub fn formatted_file_size(&self) -> String {
        format_bytes(self.file_size_bytes)
    }

    pub fn display_title(&self, fallback_filename: &str) -> String {
        self.title
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback_filename)
            .to_string()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(10 * 1024 * 1024), "10.00 MB");
        assert_eq!(
            format_bytes(2 * 1024 * 1024 * 1024 + 500 * 1024 * 1024),
            "2.49 GB"
        );
    }

    #[test]
    fn test_display_title() {
        let meta = DocumentMetadata {
            title: Some(" My Document ".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.display_title("doc.pdf"), "My Document");

        let empty_meta = DocumentMetadata {
            title: Some("   ".to_string()),
            ..Default::default()
        };
        assert_eq!(empty_meta.display_title("doc.pdf"), "doc.pdf");
    }
}
