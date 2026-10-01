use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::PdfEngine;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct PageTextOutput {
    pub page: usize,
    pub text: String,
}

#[derive(Serialize, Debug)]
pub struct DocumentTextOutput {
    pub file: String,
    pub pages_extracted: usize,
    pub pages: Vec<PageTextOutput>,
}

pub fn execute_text(
    file: &Path,
    pages_str: &str,
    format: &str,
    output: Option<&Path>,
    _json: bool,
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

    let mut extracted_pages = Vec::with_capacity(pages.len());
    for p in pages {
        let text = handle.extract_text(p)?;
        extracted_pages.push(PageTextOutput { page: p, text });
    }

    let output_content = match format.to_lowercase().as_str() {
        "json" => {
            let doc_text = DocumentTextOutput {
                file: file.to_string_lossy().to_string(),
                pages_extracted: extracted_pages.len(),
                pages: extracted_pages,
            };
            serde_json::to_string_pretty(&doc_text)
                .map_err(|e| PdfxError::Engine(format!("JSON serialization error: {}", e)))?
        }
        "jsonl" => {
            let mut lines = Vec::new();
            for item in extracted_pages {
                let line = serde_json::to_string(&item)
                    .map_err(|e| PdfxError::Engine(format!("JSON serialization error: {}", e)))?;
                lines.push(line);
            }
            lines.join("\n")
        }
        _ => {
            // Plain text
            let mut full_text = String::new();
            for item in extracted_pages {
                if !full_text.is_empty() {
                    full_text.push('\x0C'); // Form feed page break
                }
                full_text.push_str(&item.text);
            }
            full_text
        }
    };

    match output {
        Some(out_path) => {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut f = File::create(out_path)?;
            f.write_all(output_content.as_bytes())?;
            println!(
                "✓ Extracted text from '{}' saved to '{}'",
                file.display(),
                out_path.display()
            );
        }
        None => {
            println!("{}", output_content);
        }
    }

    Ok(())
}
