use crate::output::{format_bytes, print_json_success};
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::PdfEngine;
use pdfx_ocr::{OcrConfig, OcrFormat, OcrPipeline, OcrProgress};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct OcrOutput {
    pub input_file: String,
    pub output_file: String,
    pub language: String,
    pub pages_processed: usize,
    pub file_size_bytes: u64,
    pub formatted_size: String,
}

pub fn execute_ocr(
    file: &Path,
    lang: &str,
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

    let engine = LopdfEngine::new();
    let handle = engine.open_document(file, None)?;
    let total_pages = handle.page_count();

    let range = PageRange::parse(pages_str)?;
    let pages = range.resolve(total_pages)?;

    if pages.is_empty() {
        return Err(PdfxError::InvalidOperation {
            reason: "No pages specified for OCR".to_string(),
        });
    }

    let cancel_token = CancellationToken::new();
    let config = OcrConfig {
        language: lang.to_string(),
        format: OcrFormat::SearchablePdf,
        dpi: 300,
        ..Default::default()
    };

    let progress_cb = |p: OcrProgress| {
        if !json {
            println!("  [{:.0}%] {}", p.percentage, p.stage);
        }
    };

    let result = OcrPipeline::process_document(
        file,
        &pages,
        &config,
        Some(output),
        &cancel_token,
        Some(&progress_cb),
    )?;

    let file_size_bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    if json {
        let resp = OcrOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            language: lang.to_string(),
            pages_processed: result.pages_processed,
            file_size_bytes,
            formatted_size: format_bytes(file_size_bytes),
        };
        print_json_success("ocr", &resp);
    } else {
        println!(
            "✓ Successfully performed OCR on {} pages in '{}' (lang: {}) -> '{}' ({})",
            result.pages_processed,
            file.display(),
            lang,
            output.display(),
            format_bytes(file_size_bytes)
        );
    }

    Ok(())
}
