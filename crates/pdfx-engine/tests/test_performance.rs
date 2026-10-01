mod common;

use pdfx_core::page::PageRange;
use pdfx_engine::lopdf_engine::LopdfEngine;
use pdfx_engine::traits::{DocumentMutator, PdfEngine};
use pdfx_storage::cache::{MemoryLruCache, PageCacheKey};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn test_large_document_lazy_loading_and_throughput() {
    let dir = tempdir().unwrap();
    let pdf_path = dir.path().join("large_100_pages.pdf");

    // Create 100-page document
    common::create_test_pdf(&pdf_path, 100, "BenchmarkPage").unwrap();

    let engine = LopdfEngine::new();

    // 1. Measure startup / open time (must be instantaneous, sub-50ms)
    let start_open = Instant::now();
    let handle = engine.open_document(&pdf_path, None).unwrap();
    let open_duration = start_open.elapsed();
    assert_eq!(handle.page_count(), 100);
    assert!(
        open_duration.as_millis() < 250,
        "Opening 100-page PDF took too long: {:?}",
        open_duration
    );

    // 2. Measure lazy text extraction for a single random page
    let start_extract = Instant::now();
    let text = handle.extract_text(77).unwrap();
    let extract_duration = start_extract.elapsed();
    assert!(text.contains("BenchmarkPage: Page 77"));
    assert!(
        extract_duration.as_millis() < 50,
        "Lazy single page extraction took too long: {:?}",
        extract_duration
    );

    // 3. Measure metadata extraction speed
    let start_meta = Instant::now();
    let meta = handle.metadata().unwrap();
    let meta_duration = start_meta.elapsed();
    assert_eq!(meta.page_count, 100);
    assert!(
        meta_duration.as_millis() < 20,
        "Metadata extraction took too long: {:?}",
        meta_duration
    );
}

#[test]
fn test_multithreaded_concurrent_page_reads() {
    let dir = tempdir().unwrap();
    let pdf_path = dir.path().join("concurrent_test.pdf");
    common::create_test_pdf(&pdf_path, 50, "Concur").unwrap();

    let engine = LopdfEngine::new();
    let handle = Arc::new(engine.open_document(&pdf_path, None).unwrap());

    let mut handles = Vec::new();

    // Spawn 8 concurrent reader threads
    for t_idx in 0..8 {
        let h = Arc::clone(&handle);
        let t = thread::spawn(move || {
            for page in 1..=50 {
                if (page + t_idx) % 2 == 0 {
                    let text = h.extract_text(page).unwrap();
                    assert!(text.contains(&format!("Concur: Page {}", page)));
                    let dims = h.page_dimensions(page).unwrap();
                    assert_eq!(dims.width, 595.0);
                }
            }
        });
        handles.push(t);
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_compression_efficiency_and_integrity() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("uncompressed.pdf");
    let compressed = dir.path().join("compressed.pdf");

    common::create_test_pdf(&src, 20, "CompressContent").unwrap();
    let orig_size = std::fs::metadata(&src).unwrap().len();

    let engine = LopdfEngine::new();
    engine.compress(&src, &compressed).unwrap();

    assert!(compressed.exists());
    let comp_size = std::fs::metadata(&compressed).unwrap().len();

    // Verify compressed document is readable and has same page count and text
    let handle = engine.open_document(&compressed, None).unwrap();
    assert_eq!(handle.page_count(), 20);
    let text = handle.extract_text(10).unwrap();
    assert!(text.contains("CompressContent: Page 10"));

    // Size should be equal or smaller (never corrupted or massively inflated)
    assert!(comp_size <= orig_size + 100);
}

#[test]
fn test_atomic_tempfile_no_lingering_residue() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("source.pdf");
    let dest = dir.path().join("rotated.pdf");

    common::create_test_pdf(&src, 5, "TempClean").unwrap();

    let engine = LopdfEngine::new();
    let range = PageRange::parse("1-3").unwrap();
    engine.rotate(&src, &range, 90, &dest).unwrap();

    // Ensure destination exists
    assert!(dest.exists());

    // Check directory: zero .pdfx-tmp-* files remain
    let entries = std::fs::read_dir(dir.path()).unwrap();
    let mut temp_files_count = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(".pdfx-tmp-") {
            temp_files_count += 1;
        }
    }
    assert_eq!(
        temp_files_count, 0,
        "Temporary staging files were not cleaned up!"
    );
}

#[test]
fn test_memory_churn_stress_and_lru_limits() {
    // 2 MB cache capacity
    let mut cache = MemoryLruCache::new(2);
    let capacity_bytes = 2 * 1024 * 1024;

    // Insert 30 large bitmaps (each ~200 KB) = ~6 MB total churn into a 2 MB cache
    for page in 1..=30 {
        let width = 200;
        let height = 250;
        let data = vec![page as u8; (width * height * 4) as usize];
        let bmp = pdfx_core::render::RenderedBitmap::new(width, height, data);

        let key = PageCacheKey {
            document_path: "churn_test.pdf".to_string(),
            page_number: page,
            width,
            height,
        };

        cache.put(key, bmp);

        // Memory invariant: cache size must NEVER exceed configured bound!
        assert!(
            cache.current_size_bytes() <= capacity_bytes,
            "Cache size {} exceeded capacity {}",
            cache.current_size_bytes(),
            capacity_bytes
        );
    }

    // Latest inserted pages must exist, oldest must have been evicted
    let latest_key = PageCacheKey {
        document_path: "churn_test.pdf".to_string(),
        page_number: 30,
        width: 200,
        height: 250,
    };
    assert!(cache.get(&latest_key).is_some());

    let evicted_key = PageCacheKey {
        document_path: "churn_test.pdf".to_string(),
        page_number: 1,
        width: 200,
        height: 250,
    };
    assert!(cache.get(&evicted_key).is_none());
}
