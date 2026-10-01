use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use pdfx_cli::commands::*;
use pdfx_cli::exit_code_for_error;
use pdfx_cli::output::CliJsonError;
use pdfx_core::errors::PdfxError;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn create_valid_test_pdf(path: &Path, pages_count: usize) -> std::io::Result<()> {
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
        let content_text = format!("BT /F1 14 Tf 72 700 Td (Page {}) Tj ET", i);
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
    doc.save(path)?;
    Ok(())
}

#[test]
fn test_corrupted_and_empty_files_handling() {
    let dir = tempdir().unwrap();

    // 1. Zero-byte file
    let empty_file = dir.path().join("empty.pdf");
    File::create(&empty_file).unwrap();

    let err_empty = execute_info(&empty_file, false).unwrap_err();
    assert!(matches!(
        err_empty,
        PdfxError::CorruptedDocument { .. } | PdfxError::Engine(_)
    ));
    assert_eq!(exit_code_for_error(&err_empty), 5);

    // 2. Truncated corrupted PDF
    let truncated_file = dir.path().join("truncated.pdf");
    let mut f = File::create(&truncated_file).unwrap();
    f.write_all(b"%PDF-1.7\n%corrupted garbage bytes without trailer\n")
        .unwrap();

    let err_trunc = execute_info(&truncated_file, false).unwrap_err();
    assert!(matches!(
        err_trunc,
        PdfxError::CorruptedDocument { .. } | PdfxError::Engine(_)
    ));
    assert_eq!(exit_code_for_error(&err_trunc), 5);

    // 3. Non-PDF binary file (random bytes)
    let random_file = dir.path().join("random.bin");
    let mut f = File::create(&random_file).unwrap();
    f.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33])
        .unwrap();

    let err_random = execute_info(&random_file, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_random), 5);
}

#[test]
fn test_invalid_page_range_and_out_of_bounds() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("three_pages.pdf");
    create_valid_test_pdf(&pdf, 3).unwrap();

    let dest = dir.path().join("out.pdf");

    // 1. Page out of bounds (requesting page 15 on a 3-page document)
    let err_oob = execute_extract(&pdf, "15", &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_oob), 2);

    // 2. Invalid syntax expression
    let err_syntax = execute_extract(&pdf, "invalid_range", &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_syntax), 2);

    // 3. Inverted range "5-2"
    let err_inverted = execute_extract(&pdf, "5-2", &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_inverted), 2);
}

#[test]
fn test_rotation_degree_restrictions() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("sample.pdf");
    create_valid_test_pdf(&pdf, 2).unwrap();
    let dest = dir.path().join("rotated.pdf");

    // 45 degrees is not allowed
    let err_45 = execute_rotate(&pdf, "1", 45, &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_45), 2);

    // 0 degrees is not allowed
    let err_0 = execute_rotate(&pdf, "1", 0, &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_0), 2);

    // 360 degrees is not allowed
    let err_360 = execute_rotate(&pdf, "1", 360, &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_360), 2);
}

#[test]
fn test_delete_all_pages_rejection() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("two_pages.pdf");
    create_valid_test_pdf(&pdf, 2).unwrap();
    let dest = dir.path().join("deleted.pdf");

    // Attempting to delete all pages (1-2) must be rejected
    let err = execute_delete(&pdf, "1-2", &dest, true, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err), 2);
    assert!(!dest.exists());
}

#[test]
fn test_reorder_count_and_value_validation() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("three_pages.pdf");
    create_valid_test_pdf(&pdf, 3).unwrap();
    let dest = dir.path().join("reordered.pdf");

    // Count mismatch: provided 2 indices for 3 pages
    let err_count = execute_reorder(&pdf, "1,2", &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_count), 2);

    // Page number out of bounds: 4 on a 3-page PDF
    let err_bounds = execute_reorder(&pdf, "1,2,4", &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err_bounds), 2);
}

#[test]
fn test_merge_validation_minimum_files() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("single.pdf");
    create_valid_test_pdf(&pdf, 1).unwrap();
    let dest = dir.path().join("merged.pdf");

    // Merging fewer than 2 files must fail
    let err = execute_merge(&[pdf], &dest, false).unwrap_err();
    assert_eq!(exit_code_for_error(&err), 2);
}

#[test]
fn test_cli_json_error_serialization() {
    let err_obj = CliJsonError {
        status: "error",
        command: "info",
        error: "Corrupted document structure".to_string(),
    };

    let serialized = serde_json::to_string(&err_obj).unwrap();
    assert!(serialized.contains("\"status\":\"error\""));
    assert!(serialized.contains("\"command\":\"info\""));
    assert!(serialized.contains("\"error\":\"Corrupted document structure\""));
}
