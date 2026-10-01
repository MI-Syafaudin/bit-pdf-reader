use crate::output::{format_bytes, print_json_success};
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::{DocumentMutator, PdfEngine};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct ExtractOutput {
    pub input_file: String,
    pub output_file: String,
    pub extracted_pages: String,
    pub page_count: usize,
    pub file_size_bytes: u64,
    pub formatted_size: String,
}

pub fn execute_extract(
    file: &Path,
    pages_str: &str,
    output: &Path,
    json: bool,
) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let range = PageRange::parse(pages_str)?;
    let engine = LopdfEngine::new();
    engine.extract_pages(file, &range, output)?;

    let handle = engine.open_document(output, None)?;
    let page_count = handle.page_count();
    let file_size_bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    if json {
        let resp = ExtractOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            extracted_pages: pages_str.to_string(),
            page_count,
            file_size_bytes,
            formatted_size: format_bytes(file_size_bytes),
        };
        print_json_success("extract", &resp);
    } else {
        println!(
            "✓ Successfully extracted pages '{}' into '{}' ({} pages, {})",
            pages_str,
            output.display(),
            page_count,
            format_bytes(file_size_bytes)
        );
    }

    Ok(())
}
