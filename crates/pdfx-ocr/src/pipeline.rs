use crate::config::{OcrConfig, OcrFormat};
use crate::preprocess::ImagePreprocessor;
use crate::tesseract::TesseractRunner;
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_core::render::RenderOptions;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::{DocumentMutator, PdfEngine};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone)]
pub struct OcrProgress {
    pub current_page: usize,
    pub total_pages: usize,
    pub percentage: f32,
    pub stage: String,
}

#[derive(Debug, Clone)]
pub struct OcrResult {
    pub pages_processed: usize,
    pub output_file: Option<PathBuf>,
    pub extracted_text: Option<String>,
}

pub struct OcrPipeline;

impl OcrPipeline {
    pub fn process_document(
        document_path: &Path,
        pages: &[usize],
        config: &OcrConfig,
        output_path: Option<&Path>,
        cancel_token: &CancellationToken,
        progress_cb: Option<&dyn Fn(OcrProgress)>,
    ) -> Result<OcrResult, PdfxError> {
        if cancel_token.is_cancelled() {
            return Err(PdfxError::Cancelled);
        }

        if !document_path.exists() {
            return Err(PdfxError::IoError(format!(
                "Document file not found: {}",
                document_path.display()
            )));
        }

        if pages.is_empty() {
            return Err(PdfxError::InvalidOperation {
                reason: "No pages specified for OCR processing".to_string(),
            });
        }

        let runner = TesseractRunner::detect()?;

        // Verify language availability
        if let Ok(has_lang) = runner.has_language(&config.language) {
            if !has_lang {
                let available = runner.available_languages().unwrap_or_default();
                return Err(PdfxError::Ocr(format!(
                    "Language '{}' is not installed in Tesseract. Available languages: {:?}",
                    config.language, available
                )));
            }
        }

        let engine = LopdfEngine::new();
        let handle = engine.open_document(document_path, None)?;
        let total_doc_pages = handle.page_count();

        for &p in pages {
            if p == 0 || p > total_doc_pages {
                return Err(PdfxError::PageOutOfBounds {
                    index: p,
                    total: total_doc_pages,
                });
            }
        }

        let temp_dir = tempfile::tempdir().map_err(|e| PdfxError::IoError(e.to_string()))?;
        let total_pages = pages.len();
        let mut per_page_outputs: Vec<PathBuf> = Vec::with_capacity(total_pages);

        let render_opts = RenderOptions {
            dpi: config.dpi as f32,
            ..Default::default()
        };

        for (idx, &page_num) in pages.iter().enumerate() {
            if cancel_token.is_cancelled() {
                return Err(PdfxError::Cancelled);
            }

            let pct = (idx as f32 / total_pages as f32) * 100.0;
            if let Some(ref cb) = progress_cb {
                cb(OcrProgress {
                    current_page: idx + 1,
                    total_pages,
                    percentage: pct,
                    stage: format!("Rasterizing page {}/{}...", idx + 1, total_pages),
                });
            }

            // 1. Render page to bitmap
            let bitmap = handle.render_page(page_num, &render_opts)?;

            // 2. Preprocess / Binarize
            let preprocessed = ImagePreprocessor::preprocess_for_ocr(&bitmap, config.binarize);
            let img_file = temp_dir.path().join(format!("page_{}.png", page_num));
            preprocessed.save(&img_file).map_err(|e| {
                PdfxError::IoError(format!("Failed to save preprocessed image: {}", e))
            })?;

            if let Some(ref cb) = progress_cb {
                cb(OcrProgress {
                    current_page: idx + 1,
                    total_pages,
                    percentage: pct + (50.0 / total_pages as f32),
                    stage: format!("Running OCR on page {}/{}...", idx + 1, total_pages),
                });
            }

            // 3. Run Tesseract
            let out_base = temp_dir.path().join(format!("ocr_p_{}", page_num));
            let final_out = runner.run_ocr(&img_file, &out_base, config, cancel_token)?;
            per_page_outputs.push(final_out);
        }

        if cancel_token.is_cancelled() {
            return Err(PdfxError::Cancelled);
        }

        let mut result = OcrResult {
            pages_processed: total_pages,
            output_file: None,
            extracted_text: None,
        };

        match config.format {
            OcrFormat::SearchablePdf => {
                let out = output_path.ok_or_else(|| PdfxError::InvalidOperation {
                    reason: "Output path required for SearchablePdf OCR format".to_string(),
                })?;

                if let Some(ref cb) = progress_cb {
                    cb(OcrProgress {
                        current_page: total_pages,
                        total_pages,
                        percentage: 95.0,
                        stage: "Combining pages into final searchable PDF...".to_string(),
                    });
                }

                if per_page_outputs.len() == 1 {
                    std::fs::copy(&per_page_outputs[0], out)?;
                } else {
                    engine.merge(&per_page_outputs, out, cancel_token)?;
                }

                info!("Searchable PDF generated at: {:?}", out);
                result.output_file = Some(out.to_path_buf());
            }
            OcrFormat::PlainText => {
                let mut combined_text = String::new();
                for f in &per_page_outputs {
                    let page_text = std::fs::read_to_string(f)?;
                    if !combined_text.is_empty() {
                        combined_text.push('\x0C'); // Form feed page delimiter
                    }
                    combined_text.push_str(&page_text);
                }

                if let Some(out) = output_path {
                    if let Some(parent) = out.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut file = File::create(out)?;
                    file.write_all(combined_text.as_bytes())?;
                    result.output_file = Some(out.to_path_buf());
                }

                result.extracted_text = Some(combined_text);
            }
            OcrFormat::Hocr | OcrFormat::Tsv => {
                // If single page, copy; if multiple pages, copy primary or write to output
                if let Some(out) = output_path {
                    if let Some(first) = per_page_outputs.first() {
                        std::fs::copy(first, out)?;
                        result.output_file = Some(out.to_path_buf());
                    }
                }
            }
        }

        if let Some(ref cb) = progress_cb {
            cb(OcrProgress {
                current_page: total_pages,
                total_pages,
                percentage: 100.0,
                stage: "OCR Completed successfully.".to_string(),
            });
        }

        Ok(result)
    }
}
