use crate::output::{format_bytes, print_banner, print_footer, print_json_success, print_kv};
use pdfx_core::document::DocumentMetadata;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageDimensions;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::PdfEngine;
use pdfx_storage::Database;
use serde::Serialize;
use std::path::Path;
use tracing::debug;

#[derive(Serialize, Debug)]
pub struct InfoOutput {
    pub path: String,
    pub filename: String,
    pub file_size_bytes: u64,
    pub formatted_size: String,
    pub page_count: usize,
    pub pdf_version: Option<String>,
    pub is_encrypted: bool,
    pub metadata: DocumentMetadata,
    pub first_page_dimensions: Option<PageDimensions>,
}

pub fn execute_info(file: &Path, json: bool) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let engine = LopdfEngine::new();
    let handle = engine.open_document(file, None)?;
    let metadata = handle.metadata()?;
    let page_count = handle.page_count();
    let dimensions = if page_count > 0 {
        handle.page_dimensions(1).ok()
    } else {
        None
    };

    let filename = file
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "document.pdf".to_string());

    // Record in local history database
    if let Ok(mut db) = Database::open_default() {
        let _ = db.record_opened_document(
            file,
            &filename,
            metadata.file_size_bytes,
            page_count,
            metadata.title.as_deref(),
            metadata.author.as_deref(),
        );
    } else {
        debug!("Database not accessible, skipping recent files update");
    }

    if json {
        let output = InfoOutput {
            path: file.to_string_lossy().to_string(),
            filename,
            file_size_bytes: metadata.file_size_bytes,
            formatted_size: format_bytes(metadata.file_size_bytes),
            page_count,
            pdf_version: metadata.pdf_version.clone(),
            is_encrypted: metadata.is_encrypted,
            metadata: metadata.clone(),
            first_page_dimensions: dimensions,
        };
        print_json_success("info", &output);
    } else {
        print_banner("PDFX Document Information");
        print_kv("File:", file.display());
        print_kv("Page Count:", page_count);
        print_kv(
            "File Size:",
            format!(
                "{} ({} bytes)",
                format_bytes(metadata.file_size_bytes),
                metadata.file_size_bytes
            ),
        );
        print_kv(
            "PDF Version:",
            metadata.pdf_version.as_deref().unwrap_or("Unknown"),
        );
        print_kv(
            "Encrypted:",
            if metadata.is_encrypted { "Yes" } else { "No" },
        );

        if let Some(ref title) = metadata.title {
            if !title.is_empty() {
                print_kv("Title:", title);
            }
        }
        if let Some(ref author) = metadata.author {
            if !author.is_empty() {
                print_kv("Author:", author);
            }
        }
        if let Some(ref subject) = metadata.subject {
            if !subject.is_empty() {
                print_kv("Subject:", subject);
            }
        }
        if let Some(ref creator) = metadata.creator {
            if !creator.is_empty() {
                print_kv("Creator:", creator);
            }
        }
        if let Some(ref producer) = metadata.producer {
            if !producer.is_empty() {
                print_kv("Producer:", producer);
            }
        }
        if let Some(ref created) = metadata.creation_date {
            if !created.is_empty() {
                print_kv("Created:", created);
            }
        }
        if let Some(ref modified) = metadata.modification_date {
            if !modified.is_empty() {
                print_kv("Modified:", modified);
            }
        }

        if let Some(ref dim) = dimensions {
            print_kv(
                "Dimensions:",
                format!(
                    "{:.2} x {:.2} pt ({:?}, {}°)",
                    dim.width,
                    dim.height,
                    dim.orientation(),
                    dim.rotation
                ),
            );
        }

        print_footer();
    }

    Ok(())
}
