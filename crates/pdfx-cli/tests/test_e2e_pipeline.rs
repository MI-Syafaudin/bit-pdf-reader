use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use pdfx_cli::commands::*;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::PdfEngine;
use std::path::Path;
use tempfile::tempdir;

fn create_source_pdf(path: &Path, pages_count: usize) -> std::io::Result<()> {
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
        let content_text = format!("BT /F1 14 Tf 72 700 Td (E2E Section: Page {}) Tj ET", i);
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
        "Title" => "E2E Master Document",
        "Author" => "PDFX Test Engineer",
    });
    doc.trailer.set("Info", info_id);

    doc.save(path)?;
    Ok(())
}

#[test]
fn test_end_to_end_full_pipeline_chaining() {
    let dir = tempdir().unwrap();
    let original_pdf = dir.path().join("master_doc.pdf");

    // Step 1: Create 6-page source document
    create_source_pdf(&original_pdf, 6).unwrap();
    assert!(original_pdf.exists());

    // Step 2: Info command (inspect metadata)
    assert!(execute_info(&original_pdf, false).is_ok());
    assert!(execute_info(&original_pdf, true).is_ok());

    // Step 3: Split into two 3-page parts: "1-3" and "4-6"
    let split_dir = dir.path().join("split_output");
    assert!(execute_split(&original_pdf, "1-3,4-6", Some(&split_dir), false).is_ok());

    let part1 = split_dir.join("master_doc_part_1.pdf");
    let part2 = split_dir.join("master_doc_part_2.pdf");
    assert!(part1.exists());
    assert!(part2.exists());

    let engine = LopdfEngine::new();
    let h1 = engine.open_document(&part1, None).unwrap();
    let h2 = engine.open_document(&part2, None).unwrap();
    assert_eq!(h1.page_count(), 3);
    assert_eq!(h2.page_count(), 3);

    // Step 4: Rotate all pages in part 1 by 90 degrees
    let rotated_part1 = dir.path().join("part1_rotated.pdf");
    assert!(execute_rotate(&part1, "all", 90, &rotated_part1, false).is_ok());
    assert!(rotated_part1.exists());

    let h_rot = engine.open_document(&rotated_part1, None).unwrap();
    assert_eq!(h_rot.page_dimensions(1).unwrap().rotation, 90);
    assert_eq!(h_rot.page_dimensions(2).unwrap().rotation, 90);
    assert_eq!(h_rot.page_dimensions(3).unwrap().rotation, 90);

    // Step 5: Delete page 2 from part 2
    let part2_edited = dir.path().join("part2_edited.pdf");
    assert!(execute_delete(&part2, "2", &part2_edited, true, false).is_ok());
    assert!(part2_edited.exists());

    let h_part2 = engine.open_document(&part2_edited, None).unwrap();
    assert_eq!(h_part2.page_count(), 2);

    // Step 6: Merge rotated part 1 (3 pages) and edited part 2 (2 pages) -> combined 5 pages
    let combined_pdf = dir.path().join("combined.pdf");
    assert!(execute_merge(&[rotated_part1, part2_edited], &combined_pdf, false).is_ok());
    assert!(combined_pdf.exists());

    let h_comb = engine.open_document(&combined_pdf, None).unwrap();
    assert_eq!(h_comb.page_count(), 5);

    // Step 7: Compress the combined document
    let compressed_pdf = dir.path().join("compressed.pdf");
    assert!(execute_compress(&combined_pdf, &compressed_pdf, false).is_ok());
    assert!(compressed_pdf.exists());

    let h_comp = engine.open_document(&compressed_pdf, None).unwrap();
    assert_eq!(h_comp.page_count(), 5);

    // Step 8: Extract text from compressed PDF to JSON file
    let text_json = dir.path().join("extracted_text.json");
    assert!(execute_text(&compressed_pdf, "all", "json", Some(&text_json), false).is_ok());
    assert!(text_json.exists());

    let json_content = std::fs::read_to_string(&text_json).unwrap();
    assert!(json_content.contains("E2E Section: Page 1"));
    assert!(json_content.contains("pages_extracted\": 5"));

    // Step 9: Reorder pages (reverse order 5,4,3,2,1)
    let reordered_pdf = dir.path().join("reordered.pdf");
    assert!(execute_reorder(&compressed_pdf, "5,4,3,2,1", &reordered_pdf, false).is_ok());
    assert!(reordered_pdf.exists());

    let h_reorder = engine.open_document(&reordered_pdf, None).unwrap();
    assert_eq!(h_reorder.page_count(), 5);

    // Step 10: Clean history database
    assert!(execute_clean(true, true, true, false).is_ok());
}
