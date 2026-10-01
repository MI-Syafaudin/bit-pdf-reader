use pdfx_core::jobs::{JobInfo, JobStatus};
use pdfx_core::render::RenderedBitmap;
use pdfx_storage::{AppSettings, Database, MemoryLruCache, PageCacheKey};
use std::path::Path;

#[test]
fn test_database_recent_documents_workflow() {
    let mut db = Database::open_in_memory().unwrap();

    let doc_id = db
        .record_opened_document(
            Path::new("/home/user/doc1.pdf"),
            "doc1.pdf",
            1024 * 1024,
            12,
            Some("Document One"),
            Some("Author A"),
        )
        .unwrap();

    assert!(doc_id > 0);

    let recent = db.get_recent_documents(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].filename, "doc1.pdf");
    assert_eq!(recent[0].page_count, 12);
    assert_eq!(recent[0].last_page_viewed, 1);
    assert_eq!(recent[0].zoom_level, 100.0);

    // Update viewing state
    db.update_last_page_and_zoom(doc_id, 5, 125.0).unwrap();
    let updated_recent = db.get_recent_documents(10).unwrap();
    assert_eq!(updated_recent[0].last_page_viewed, 5);
    assert_eq!(updated_recent[0].zoom_level, 125.0);

    // Remove single recent
    db.remove_recent_document(doc_id).unwrap();
    assert_eq!(db.get_recent_documents(10).unwrap().len(), 0);
}

#[test]
fn test_database_bookmarks_workflow() {
    let mut db = Database::open_in_memory().unwrap();

    let doc_id = db
        .record_opened_document(
            Path::new("/docs/report.pdf"),
            "report.pdf",
            50000,
            20,
            None,
            None,
        )
        .unwrap();

    let bm1 = db.add_bookmark(doc_id, 3, "Chapter 1: Intro").unwrap();
    let _bm2 = db.add_bookmark(doc_id, 10, "Chapter 2: Methods").unwrap();

    let bookmarks = db.get_bookmarks_for_document(doc_id).unwrap();
    assert_eq!(bookmarks.len(), 2);
    assert_eq!(bookmarks[0].page_number, 3);
    assert_eq!(bookmarks[0].title, "Chapter 1: Intro");
    assert_eq!(bookmarks[1].page_number, 10);

    // Rename bookmark
    db.rename_bookmark(bm1, "Chapter 1: Introduction").unwrap();
    let renamed = db.get_bookmarks_for_document(doc_id).unwrap();
    assert_eq!(renamed[0].title, "Chapter 1: Introduction");

    // Remove bookmark
    db.remove_bookmark(bm1).unwrap();
    let remaining = db.get_bookmarks_for_document(doc_id).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].page_number, 10);
}

#[test]
fn test_database_jobs_workflow() {
    let mut db = Database::open_in_memory().unwrap();

    let job = JobInfo::new("job-101", "compress", "/tmp/large.pdf");
    db.create_job(&job).unwrap();

    let loaded = db.get_job("job-101").unwrap().unwrap();
    assert_eq!(loaded.id, "job-101");
    assert_eq!(loaded.status, JobStatus::Queued);

    // Update progress
    db.update_job_progress("job-101", 65.0, JobStatus::Running, None, None)
        .unwrap();
    let running = db.get_job("job-101").unwrap().unwrap();
    assert_eq!(running.status, JobStatus::Running);
    assert_eq!(running.progress.percentage, 65.0);

    // Complete job
    db.update_job_progress(
        "job-101",
        100.0,
        JobStatus::Completed,
        None,
        Some("/tmp/large_compressed.pdf"),
    )
    .unwrap();

    let completed = db.get_job("job-101").unwrap().unwrap();
    assert_eq!(completed.status, JobStatus::Completed);
    assert_eq!(
        completed.output_path.as_deref(),
        Some("/tmp/large_compressed.pdf")
    );

    // Clear completed jobs
    let cleared = db.clear_completed_jobs().unwrap();
    assert_eq!(cleared, 1);
    assert!(db.get_job("job-101").unwrap().is_none());
}

#[test]
fn test_database_settings_workflow() {
    let mut db = Database::open_in_memory().unwrap();

    // Default settings
    let default_settings = db.load_app_settings().unwrap();
    assert_eq!(default_settings.theme, "dark");
    assert_eq!(default_settings.max_cache_mb, 256);

    // Save custom settings
    let custom = AppSettings {
        theme: "light".to_string(),
        confirm_destructive: false,
        max_cache_mb: 512,
        max_parallel_jobs: 8,
        default_dpi: 300.0,
        ocr_language: "ind".to_string(),
    };
    db.save_app_settings(&custom).unwrap();

    let reloaded = db.load_app_settings().unwrap();
    assert_eq!(reloaded, custom);
}

#[test]
fn test_lru_cache_eviction() {
    // 1 MB cache limit
    let mut cache = MemoryLruCache::new(1);

    // A 400 KB bitmap: 100 x 1000 x 4 bytes = 400,000 bytes
    let bmp1 = RenderedBitmap::new(100, 1000, vec![0; 400_000]);
    let key1 = PageCacheKey {
        document_path: "doc.pdf".to_string(),
        page_number: 1,
        width: 100,
        height: 1000,
    };

    let bmp2 = RenderedBitmap::new(100, 1000, vec![1; 400_000]);
    let key2 = PageCacheKey {
        document_path: "doc.pdf".to_string(),
        page_number: 2,
        width: 100,
        height: 1000,
    };

    cache.put(key1.clone(), bmp1);
    cache.put(key2.clone(), bmp2);
    // Access key2 first, then key1 -> key1 becomes MRU, key2 becomes LRU
    assert!(cache.get(&key2).is_some());
    assert!(cache.get(&key1).is_some());

    // Putting a 3rd bitmap (400 KB) pushes total to 1.2 MB > 1 MB limit
    // Key2 is least recently used, so key2 should be evicted!
    let bmp3 = RenderedBitmap::new(100, 1000, vec![2; 400_000]);
    let key3 = PageCacheKey {
        document_path: "doc.pdf".to_string(),
        page_number: 3,
        width: 100,
        height: 1000,
    };
    cache.put(key3.clone(), bmp3);

    assert!(cache.current_size_bytes() <= 1024 * 1024);
    assert!(cache.get(&key3).is_some());
    assert!(cache.get(&key1).is_some());
    assert!(cache.get(&key2).is_none());
}
