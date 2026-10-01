use crate::output::{format_bytes, print_json_success};
use pdfx_core::errors::PdfxError;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::DocumentMutator;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct CompressOutput {
    pub input_file: String,
    pub output_file: String,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub bytes_saved: i64,
    pub compression_percentage: f32,
}

pub fn execute_compress(file: &Path, output: &Path, json: bool) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let orig_size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
    let engine = LopdfEngine::new();
    engine.compress(file, output)?;
    let new_size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    let bytes_saved = (orig_size as i64) - (new_size as i64);
    let percentage = if orig_size > 0 {
        ((bytes_saved as f64 / orig_size as f64) * 100.0) as f32
    } else {
        0.0
    };

    if json {
        let resp = CompressOutput {
            input_file: file.to_string_lossy().to_string(),
            output_file: output.to_string_lossy().to_string(),
            original_size_bytes: orig_size,
            compressed_size_bytes: new_size,
            bytes_saved,
            compression_percentage: percentage,
        };
        print_json_success("compress", &resp);
    } else {
        println!("✓ Successfully compressed '{}':", file.display());
        println!("  Target:     {}", output.display());
        println!(
            "  Original:   {} ({} bytes)",
            format_bytes(orig_size),
            orig_size
        );
        println!(
            "  Compressed: {} ({} bytes)",
            format_bytes(new_size),
            new_size
        );
        if bytes_saved > 0 {
            println!(
                "  Savings:    {} ({:.1}%)",
                format_bytes(bytes_saved as u64),
                percentage
            );
        } else {
            println!("  Savings:    0% (already maximally compressed)");
        }
    }

    Ok(())
}
