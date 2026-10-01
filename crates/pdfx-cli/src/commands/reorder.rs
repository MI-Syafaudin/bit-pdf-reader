use crate::output::print_json_success;
use pdfx_core::errors::PdfxError;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::DocumentMutator;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct ReorderOutput {
    pub input_file: String,
    pub output_file: String,
    pub new_order: Vec<usize>,
}

pub fn execute_reorder(
    file: &Path,
    order_str: &str,
    output: &Path,
    json: bool,
) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let order: Result<Vec<usize>, _> = order_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<usize>()
                .map_err(|_| PdfxError::InvalidOperation {
                    reason: format!("Invalid page number '{}' in order sequence", s.trim()),
                })
        })
        .collect();
    let order = order?;

    if order.is_empty() {
        return Err(PdfxError::InvalidOperation {
            reason: "Empty page order specified".to_string(),
        });
    }

    let engine = LopdfEngine::new();
    engine.reorder_pages(file, &order, output)?;

    if json {
        let resp = ReorderOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            new_order: order,
        };
        print_json_success("reorder", &resp);
    } else {
        println!(
            "✓ Successfully reordered pages in '{}' to '{}' ({})",
            file.display(),
            output.display(),
            order_str
        );
    }

    Ok(())
}
