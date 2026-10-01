use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use pdfx_cli::commands::*;
use pdfx_cli::exit_code_for_error;
use pdfx_core::errors::PdfxError;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::PdfEngine;
use std::path::Path;
use tempfile::tempdir;

fn create_test_pdf(path: &Path, pages_count: usize, text_prefix: &str) -> std::io::Result<()> {
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
        let content_text = format!("BT /F1 14 Tf 72 700 Td ({}: Page {}) Tj ET", text_prefix, i);
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            content_text.into_bytes(),
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

    let info_id = doc.add_object(dictionary! {
        "Title" => "PDFX Test Document",
        "Author" => "PDFX Test Suite",
    });
    doc.trailer.set("Info", info_id);

    doc.save(path)?;
    Ok(())
}

#[test]
fn test_cli_info_success_and_json() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("test.pdf");
    create_test_pdf(&pdf, 3, "InfoTest").unwrap();

    // Standard output
    assert!(execute_info(&pdf, false).is_ok());

    // JSON output
    assert!(execute_info(&pdf, true).is_ok());

    // Non-existent file error
    let missing = dir.path().join("non_existent.pdf");
    let err = execute_info(&missing, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err), 3);
}

#[test]
fn test_cli_merge() {
    let dir = tempdir().unwrap();
    let pdf1 = dir.path().join("part1.pdf");
    let pdf2 = dir.path().join("part2.pdf");
    let merged = dir.path().join("merged.pdf");

    create_test_pdf(&pdf1, 2, "P1").unwrap();
    create_test_pdf(&pdf2, 3, "P2").unwrap();

    assert!(execute_merge(&[pdf1, pdf2], &merged, false).is_ok());
    assert!(merged.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&merged, None).unwrap();
    assert_eq!(handle.page_count(), 5);
}

#[test]
fn test_cli_split() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    create_test_pdf(&pdf, 4, "SplitTest").unwrap();

    let split_dir = dir.path().join("splits");
    assert!(execute_split(&pdf, "1-2,3-4", Some(&split_dir), false).is_ok());

    let part1 = split_dir.join("source_part_1.pdf");
    let part2 = split_dir.join("source_part_2.pdf");
    assert!(part1.exists());
    assert!(part2.exists());

    let engine = LopdfEngine::new();
    let h1 = engine.open_document(&part1, None).unwrap();
    let h2 = engine.open_document(&part2, None).unwrap();
    assert_eq!(h1.page_count(), 2);
    assert_eq!(h2.page_count(), 2);
}

#[test]
fn test_cli_extract() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let extracted = dir.path().join("extracted.pdf");
    create_test_pdf(&pdf, 5, "ExtractTest").unwrap();

    assert!(execute_extract(&pdf, "2-4", &extracted, false).is_ok());
    assert!(extracted.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&extracted, None).unwrap();
    assert_eq!(handle.page_count(), 3);
}

#[test]
fn test_cli_rotate() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let rotated = dir.path().join("rotated.pdf");
    create_test_pdf(&pdf, 2, "RotateTest").unwrap();

    // Valid rotation
    assert!(execute_rotate(&pdf, "1", 90, &rotated, false).is_ok());
    assert!(rotated.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&rotated, None).unwrap();
    let dim1 = handle.page_dimensions(1).unwrap();
    assert_eq!(dim1.rotation, 90);

    // Invalid rotation degrees
    let err = execute_rotate(&pdf, "1", 45, &rotated, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err), 2);
}

#[test]
fn test_cli_delete() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let deleted = dir.path().join("deleted.pdf");
    create_test_pdf(&pdf, 4, "DeleteTest").unwrap();

    // With yes=true to bypass interactive prompt
    assert!(execute_delete(&pdf, "2", &deleted, true, false).is_ok());
    assert!(deleted.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&deleted, None).unwrap();
    assert_eq!(handle.page_count(), 3);
}

#[test]
fn test_cli_reorder() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let reordered = dir.path().join("reordered.pdf");
    create_test_pdf(&pdf, 3, "ReorderTest").unwrap();

    assert!(execute_reorder(&pdf, "3,1,2", &reordered, false).is_ok());
    assert!(reordered.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&reordered, None).unwrap();
    assert_eq!(handle.page_count(), 3);

    // Invalid reorder (out of bounds)
    let err = execute_reorder(&pdf, "1,2,5", &reordered, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err), 2);
}

#[test]
fn test_cli_text() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let text_out = dir.path().join("extracted.txt");
    create_test_pdf(&pdf, 2, "TextHello").unwrap();

    assert!(execute_text(&pdf, "all", "plain", Some(&text_out), false).is_ok());
    assert!(text_out.exists());
    let content = std::fs::read_to_string(&text_out).unwrap();
    assert!(content.contains("TextHello: Page 1"));
    assert!(content.contains("TextHello: Page 2"));

    // JSON format
    let json_out = dir.path().join("extracted.json");
    assert!(execute_text(&pdf, "1", "json", Some(&json_out), false).is_ok());
    assert!(json_out.exists());
    let json_content = std::fs::read_to_string(&json_out).unwrap();
    assert!(json_content.contains("\"page\": 1"));
}

#[test]
fn test_cli_compress() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("source.pdf");
    let compressed = dir.path().join("compressed.pdf");
    create_test_pdf(&pdf, 3, "CompressTest").unwrap();

    assert!(execute_compress(&pdf, &compressed, false).is_ok());
    assert!(compressed.exists());

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&compressed, None).unwrap();
    assert_eq!(handle.page_count(), 3);
}

#[test]
fn test_cli_clean() {
    // Tests cleaning of DB records
    assert!(execute_clean(true, true, false, false).is_ok());
    assert!(execute_clean(false, false, true, true).is_ok());
}

#[test]
fn test_exit_codes_mapping() {
    let err_arg = PdfxError::InvalidRotation { degrees: 45 };
    assert_eq!(exit_code_for_error(&err_arg), 2);

    let err_input = PdfxError::IoError("File not found: /path/foo".to_string());
    assert_eq!(exit_code_for_error(&err_input), 3);

    let err_output = PdfxError::IoError("Permission denied: cannot write to /root".to_string());
    assert_eq!(exit_code_for_error(&err_output), 4);

    let err_pdf = PdfxError::Engine("Corrupted trailer".to_string());
    assert_eq!(exit_code_for_error(&err_pdf), 5);

    let err_ocr = PdfxError::Ocr("Tesseract not installed".to_string());
    assert_eq!(exit_code_for_error(&err_ocr), 6);

    let err_cancel = PdfxError::Cancelled;
    assert_eq!(exit_code_for_error(&err_cancel), 7);
}
