mod common;

use pdfx_core::cancellation::CancellationToken;
use pdfx_core::page::PageRange;
use pdfx_core::render::RenderOptions;
use pdfx_engine::{DocumentMutator, LopdfEngine, PdfEngine};
use tempfile::tempdir;

#[test]
fn test_engine_open_metadata_and_dimensions() {
    let dir = tempdir().unwrap();
    let pdf_path = dir.path().join("doc.pdf");
    common::create_test_pdf(&pdf_path, 3, "Sample").unwrap();

    let engine = LopdfEngine::new();
    let handle = engine.open_document(&pdf_path, None).unwrap();

    assert_eq!(handle.page_count(), 3);

    let meta = handle.metadata().unwrap();
    assert_eq!(meta.title.as_deref(), Some("Test Document"));
    assert_eq!(meta.author.as_deref(), Some("PDFX Test Suite"));
    assert_eq!(meta.page_count, 3);
    assert!(!meta.is_encrypted);

    let dims = handle.page_dimensions(1).unwrap();
    assert_eq!(dims.width, 595.0);
    assert_eq!(dims.height, 842.0);
    assert_eq!(dims.rotation, 0);

    // Test text extraction and search
    let text = handle.extract_text(1).unwrap();
    assert!(text.contains("Sample: Page 1"));

    let search_results = handle.search_text(1, "Sample", false).unwrap();
    assert!(!search_results.is_empty());
    assert_eq!(search_results[0].page_number, 1);

    // Test render fallback
    let opts = RenderOptions::default();
    let bitmap = handle.render_page(1, &opts).unwrap();
    assert!(bitmap.width >= 100);
    assert!(bitmap.height >= 100);
    assert_eq!(
        bitmap.byte_size(),
        (bitmap.width * bitmap.height * 4) as usize
    );
}

#[test]
fn test_engine_rotate() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.pdf");
    let rotated = dir.path().join("rotated.pdf");
    common::create_test_pdf(&src, 2, "RotateTest").unwrap();

    let engine = LopdfEngine::new();
    engine
        .rotate(&src, &PageRange::parse("1").unwrap(), 90, &rotated)
        .unwrap();

    let handle = engine.open_document(&rotated, None).unwrap();
    let dims_p1 = handle.page_dimensions(1).unwrap();
    assert_eq!(dims_p1.rotation, 90);

    let dims_p2 = handle.page_dimensions(2).unwrap();
    assert_eq!(dims_p2.rotation, 0);
}

#[test]
fn test_engine_delete_pages() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.pdf");
    let deleted = dir.path().join("deleted.pdf");
    common::create_test_pdf(&src, 4, "DeleteTest").unwrap();

    let engine = LopdfEngine::new();
    engine
        .delete_pages(&src, &PageRange::parse("2,4").unwrap(), &deleted)
        .unwrap();

    let handle = engine.open_document(&deleted, None).unwrap();
    assert_eq!(handle.page_count(), 2);
}

#[test]
fn test_engine_reorder_pages() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.pdf");
    let reordered = dir.path().join("reordered.pdf");
    common::create_test_pdf(&src, 3, "Order").unwrap();

    let engine = LopdfEngine::new();
    engine.reorder_pages(&src, &[3, 1, 2], &reordered).unwrap();

    let handle = engine.open_document(&reordered, None).unwrap();
    assert_eq!(handle.page_count(), 3);
}

#[test]
fn test_engine_merge() {
    let dir = tempdir().unwrap();
    let doc_a = dir.path().join("a.pdf");
    let doc_b = dir.path().join("b.pdf");
    let merged = dir.path().join("merged.pdf");

    common::create_test_pdf(&doc_a, 2, "DocA").unwrap();
    common::create_test_pdf(&doc_b, 3, "DocB").unwrap();

    let engine = LopdfEngine::new();
    let cancel = CancellationToken::new();
    engine.merge(&[doc_a, doc_b], &merged, &cancel).unwrap();

    let handle = engine.open_document(&merged, None).unwrap();
    assert_eq!(handle.page_count(), 5);
}

#[test]
fn test_engine_split() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.pdf");
    let split_dir = dir.path().join("splits");
    common::create_test_pdf(&src, 5, "Split").unwrap();

    let engine = LopdfEngine::new();
    let cancel = CancellationToken::new();
    let ranges = vec![
        PageRange::parse("1-2").unwrap(),
        PageRange::parse("3-5").unwrap(),
    ];

    let output_files = engine.split(&src, &ranges, &split_dir, &cancel).unwrap();
    assert_eq!(output_files.len(), 2);

    let handle1 = engine.open_document(&output_files[0], None).unwrap();
    assert_eq!(handle1.page_count(), 2);

    let handle2 = engine.open_document(&output_files[1], None).unwrap();
    assert_eq!(handle2.page_count(), 3);
}
