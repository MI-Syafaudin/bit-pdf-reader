use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use pdfx_core::render::RenderedBitmap;
use pdfx_ocr::config::{OcrConfig, OcrFormat, PageSegmentationMode};
use pdfx_ocr::pipeline::OcrPipeline;
use pdfx_ocr::preprocess::ImagePreprocessor;
use pdfx_ocr::tesseract::TesseractRunner;
use pdfx_ocr::worker::OcrWorker;
use std::path::Path;
use tempfile::tempdir;

fn create_sample_pdf(path: &Path, pages_count: usize) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut page_ids = Vec::new();
    for i in 1..=pages_count {
        let text = format!("BT /F1 14 Tf 72 700 Td (OCR Scanned Page {}) Tj ET", i);
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            text.into_bytes(),
        )));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        page_ids.push(page_id);
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => pages_count as u32,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(path)?;
    Ok(())
}

#[test]
fn test_ocr_config_and_formats() {
    let mut config = OcrConfig::default();
    assert_eq!(config.language, "eng");
    assert_eq!(config.format, OcrFormat::SearchablePdf);
    assert_eq!(config.format.file_extension(), "pdf");

    config.format = OcrFormat::PlainText;
    assert_eq!(config.format.file_extension(), "txt");

    config.format = OcrFormat::Hocr;
    assert_eq!(config.format.file_extension(), "hocr");

    config.format = OcrFormat::Tsv;
    assert_eq!(config.format.file_extension(), "tsv");

    assert_eq!(PageSegmentationMode::Auto.psm_number(), "3");
    assert_eq!(PageSegmentationMode::SingleBlock.psm_number(), "6");
    assert_eq!(PageSegmentationMode::SingleLine.psm_number(), "7");
    assert_eq!(PageSegmentationMode::SparseText.psm_number(), "11");
}

#[test]
fn test_image_preprocessing_pipeline() {
    let width = 20;
    let height = 20;
    let mut data = vec![255; (width * height * 4) as usize];

    // Dark square in the center
    for y in 5..15 {
        for x in 5..15 {
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = 10;
            data[idx + 1] = 10;
            data[idx + 2] = 10;
        }
    }

    let bmp = RenderedBitmap::new(width, height, data);
    let gray = ImagePreprocessor::preprocess_for_ocr(&bmp, true);

    assert_eq!(gray.width(), 20);
    assert_eq!(gray.height(), 20);

    // Center pixel should be black (0)
    assert_eq!(gray.get_pixel(10, 10)[0], 0);
    // Outer corner should be white (255)
    assert_eq!(gray.get_pixel(0, 0)[0], 255);
}

#[test]
fn test_tesseract_runner_detection_or_fallback() {
    match TesseractRunner::detect() {
        Ok(runner) => {
            let langs = runner.available_languages();
            assert!(langs.is_ok());
        }
        Err(err) => match err {
            PdfxError::Ocr(msg) => {
                assert!(msg.contains("Tesseract OCR is not installed"));
            }
            _ => panic!("Unexpected error type: {:?}", err),
        },
    }
}

#[test]
fn test_ocr_cancellation_token() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("test.pdf");
    create_sample_pdf(&pdf, 2).unwrap();

    let cancel = CancellationToken::new();
    cancel.cancel(); // Pre-cancel

    let config = OcrConfig::default();
    let res = OcrPipeline::process_document(&pdf, &[1], &config, None, &cancel, None);

    assert_eq!(res.unwrap_err(), PdfxError::Cancelled);
}

#[test]
fn test_ocr_pipeline_validation() {
    let dir = tempdir().unwrap();
    let missing_pdf = dir.path().join("non_existent.pdf");
    let cancel = CancellationToken::new();
    let config = OcrConfig::default();

    // 1. Missing file error
    let res = OcrPipeline::process_document(&missing_pdf, &[1], &config, None, &cancel, None);
    assert!(matches!(res.unwrap_err(), PdfxError::IoError(_)));

    // 2. Empty pages error
    let valid_pdf = dir.path().join("valid.pdf");
    create_sample_pdf(&valid_pdf, 2).unwrap();
    let res = OcrPipeline::process_document(&valid_pdf, &[], &config, None, &cancel, None);
    assert!(matches!(
        res.unwrap_err(),
        PdfxError::InvalidOperation { .. }
    ));

    // 3. Out of bounds page error (only if tesseract installed, or tested directly)
    if TesseractRunner::detect().is_ok() {
        let res = OcrPipeline::process_document(&valid_pdf, &[10], &config, None, &cancel, None);
        assert!(matches!(
            res.unwrap_err(),
            PdfxError::PageOutOfBounds { .. }
        ));
    }
}

#[test]
fn test_ocr_worker_job_spawn_and_cancel() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("worker_test.pdf");
    create_sample_pdf(&pdf, 2).unwrap();

    let config = OcrConfig::default();
    let handle = OcrWorker::spawn_job(pdf, vec![1, 2], config, None);

    // Immediately cancel
    handle.cancel();

    // Verify result is received and reflects cancellation or error
    let res = handle.result_rx.recv().unwrap();
    assert!(res.is_err());
}
