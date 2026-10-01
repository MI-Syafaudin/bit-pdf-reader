use crate::output::print_json_success;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::DocumentMutator;
use serde::Serialize;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct DeleteOutput {
    pub input_file: String,
    pub output_file: String,
    pub deleted_pages: String,
}

pub fn execute_delete(
    file: &Path,
    pages_str: &str,
    output: &Path,
    yes: bool,
    json: bool,
) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let range = PageRange::parse(pages_str)?;

    // Confirmation if destructive and interactive
    if !yes && stdin().is_terminal() {
        print!(
            "Are you sure you want to delete pages '{}' from '{}'? [y/N]: ",
            pages_str,
            file.display()
        );
        let _ = stdout().flush();
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .map_err(|e| PdfxError::IoError(e.to_string()))?;
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            return Err(PdfxError::Cancelled);
        }
    }

    let engine = LopdfEngine::new();
    engine.delete_pages(file, &range, output)?;

    if json {
        let resp = DeleteOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            deleted_pages: pages_str.to_string(),
        };
        print_json_success("delete", &resp);
    } else {
        println!(
            "✓ Successfully deleted pages '{}' from '{}' into '{}'",
            pages_str,
            file.display(),
            output.display()
        );
    }

    Ok(())
}
