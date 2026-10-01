use crate::output::print_json_success;
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::DocumentMutator;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct SplitOutput {
    pub input_file: String,
    pub output_files: Vec<String>,
    pub output_count: usize,
}

pub fn execute_split(
    file: &Path,
    pages_str: &str,
    output_dir: Option<&Path>,
    json: bool,
) -> Result<(), PdfxError> {
    if !file.exists() {
        return Err(PdfxError::IoError(format!(
            "Input file not found: {}",
            file.display()
        )));
    }

    let out_dir = output_dir.unwrap_or_else(|| Path::new("."));
    let parsed = PageRange::parse(pages_str)?;
    let ranges: Vec<PageRange> = match parsed {
        PageRange::Composite(sub) => sub,
        single => vec![single],
    };

    let cancel_token = CancellationToken::new();
    let engine = LopdfEngine::new();
    let created_files = engine.split(file, &ranges, out_dir, &cancel_token)?;

    if json {
        let resp = SplitOutput {
            input_file: file.to_string_lossy().to_string(),
            output_files: created_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            output_count: created_files.len(),
        };
        print_json_success("split", &resp);
    } else {
        println!(
            "✓ Successfully split '{}' into {} files:",
            file.display(),
            created_files.len()
        );
        for f in &created_files {
            println!("  - {}", f.display());
        }
    }

    Ok(())
}
