use crate::output::{format_bytes, print_json_success};
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::{DocumentMutator, PdfEngine};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug)]
pub struct MergeOutput {
    pub output_file: String,
    pub input_files_count: usize,
    pub input_files: Vec<String>,
    pub total_pages: usize,
    pub file_size_bytes: u64,
    pub formatted_size: String,
}

pub fn execute_merge(files: &[PathBuf], output: &Path, json: bool) -> Result<(), PdfxError> {
    if files.len() < 2 {
        return Err(PdfxError::InvalidOperation {
            reason: "At least 2 PDF documents are required to merge".to_string(),
        });
    }

    for file in files {
        if !file.exists() {
            return Err(PdfxError::IoError(format!(
                "Input file not found: {}",
                file.display()
            )));
        }
    }

    let cancel_token = CancellationToken::new();
    let engine = LopdfEngine::new();
    engine.merge(files, output, &cancel_token)?;

    let handle = engine.open_document(output, None)?;
    let total_pages = handle.page_count();
    let file_size_bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    if json {
        let resp = MergeOutput {
            output_file: output.to_string_lossy().to_string(),
            input_files_count: files.len(),
            input_files: files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            total_pages,
            file_size_bytes,
            formatted_size: format_bytes(file_size_bytes),
        };
        print_json_success("merge", &resp);
    } else {
        println!(
            "✓ Successfully merged {} files into '{}' ({} pages, {})",
            files.len(),
            output.display(),
            total_pages,
            format_bytes(file_size_bytes)
        );
    }

    Ok(())
}
