use crate::output::print_json_success;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::DocumentMutator;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct RotateOutput {
    pub input_file: String,
    pub output_file: String,
    pub rotated_pages: String,
    pub degrees: u16,
}

pub fn execute_rotate(
    file: &Path,
    pages_str: &str,
    degrees: u16,
    output: &Path,
    json: bool,
) -> Result<(), PdfxError> {
    if degrees != 90 && degrees != 180 && degrees != 270 {
        return Err(PdfxError::InvalidRotation { degrees });
    }

    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let range = PageRange::parse(pages_str)?;
    let engine = LopdfEngine::new();
    engine.rotate(file, &range, degrees, output)?;

    if json {
        let resp = RotateOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            rotated_pages: pages_str.to_string(),
            degrees,
        };
        print_json_success("rotate", &resp);
    } else {
        println!(
            "✓ Successfully rotated pages '{}' by {}° into '{}'",
            pages_str,
            degrees,
            output.display()
        );
    }

    Ok(())
}
