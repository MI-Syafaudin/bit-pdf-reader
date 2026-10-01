use crate::models::{AppSettings, BookmarkRecord, RecentDocumentRecord};
use directories::ProjectDirs;
use pdfx_core::errors::PdfxError;
use pdfx_core::jobs::{JobInfo, JobProgress, JobStatus};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the default database at XDG data dir (~/.local/share/pdfx/pdfx.db)
    pub fn open_default() -> Result<Self, PdfxError> {
        let path = Self::default_db_path()?;
        Self::open(&path)
    }

    /// Open an in-memory database for testing
    pub fn open_in_memory() -> Result<Self, PdfxError> {
        let conn = Connection::open_in_memory().map_err(|e| PdfxError::Database(e.to_string()))?;
        let db = Self { conn };
        db.initialize_pragmas()?;
        db.initialize_schema()?;
        Ok(db)
    }

    pub fn open(path: &Path) -> Result<Self, PdfxError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).map_err(|e| PdfxError::Database(e.to_string()))?;
        let db = Self { conn };
        db.initialize_pragmas()?;
        db.initialize_schema()?;
        Ok(db)
    }

    pub fn default_db_path() -> Result<PathBuf, PdfxError> {
        let proj = ProjectDirs::from("org", "pdfx", "pdfx").ok_or_else(|| {
            PdfxError::Config("Unable to resolve project data directories".to_string())
        })?;
        Ok(proj.data_dir().join("pdfx.db"))
    }

    fn initialize_pragmas(&self) -> Result<(), PdfxError> {
        self.conn
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA busy_timeout = 5000;
                 PRAGMA foreign_keys = ON;
                 PRAGMA temp_store = MEMORY;",
            )
            .map_err(|e| {
                PdfxError::Database(format!("Failed to configure SQLite pragmas: {}", e))
            })?;
        Ok(())
    }

    fn initialize_schema(&self) -> Result<(), PdfxError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS documents (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL UNIQUE,
                    filename TEXT NOT NULL,
                    size_bytes INTEGER NOT NULL,
                    page_count INTEGER NOT NULL,
                    title TEXT,
                    author TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    last_opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path);

                CREATE TABLE IF NOT EXISTS recent_files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_id INTEGER NOT NULL UNIQUE,
                    opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    last_page_viewed INTEGER NOT NULL DEFAULT 1,
                    zoom_level REAL NOT NULL DEFAULT 100.0,
                    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_recent_files_opened ON recent_files(opened_at DESC);

                CREATE TABLE IF NOT EXISTS bookmarks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_id INTEGER NOT NULL,
                    page_number INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
                    UNIQUE(document_id, page_number)
                );
                CREATE INDEX IF NOT EXISTS idx_bookmarks_doc ON bookmarks(document_id);

                CREATE TABLE IF NOT EXISTS jobs (
                    id TEXT PRIMARY KEY,
                    job_type TEXT NOT NULL,
                    status TEXT NOT NULL,
                    input_path TEXT NOT NULL,
                    output_path TEXT,
                    progress REAL NOT NULL DEFAULT 0.0,
                    error_message TEXT,
                    created_at_unix INTEGER NOT NULL,
                    completed_at_unix INTEGER
                );
                CREATE INDEX IF NOT EXISTS idx_jobs_created ON jobs(created_at_unix DESC);

                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );",
            )
            .map_err(|e| {
                PdfxError::Database(format!("Failed to initialize database schema: {}", e))
            })?;
        debug!("Database schema verified");
        Ok(())
    }

    // ==========================================
    // DOCUMENTS & RECENT FILES
    // ==========================================

    pub fn record_opened_document(
        &mut self,
        path: &Path,
        filename: &str,
        size_bytes: u64,
        page_count: usize,
        title: Option<&str>,
        author: Option<&str>,
    ) -> Result<i64, PdfxError> {
        let path_str = path.to_string_lossy();
        self.conn
            .execute(
                "INSERT INTO documents (path, filename, size_bytes, page_count, title, author, last_opened_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
             ON CONFLICT(path) DO UPDATE SET
                last_opened_at = CURRENT_TIMESTAMP,
                size_bytes = excluded.size_bytes,
                page_count = excluded.page_count,
                title = excluded.title,
                author = excluded.author;",
                params![path_str, filename, size_bytes as i64, page_count as i64, title, author],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let doc_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM documents WHERE path = ?1",
                params![path_str],
                |row| row.get(0),
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO recent_files (document_id, opened_at)
             VALUES (?1, CURRENT_TIMESTAMP)
             ON CONFLICT(document_id) DO UPDATE SET
                opened_at = CURRENT_TIMESTAMP;",
                params![doc_id],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        info!("Recorded opened document id {} at {}", doc_id, path_str);
        Ok(doc_id)
    }

    pub fn update_last_page_and_zoom(
        &mut self,
        doc_id: i64,
        page: usize,
        zoom: f32,
    ) -> Result<(), PdfxError> {
        self.conn
            .execute(
                "UPDATE recent_files SET last_page_viewed = ?1, zoom_level = ?2 WHERE document_id = ?3",
                params![page as i64, zoom as f64, doc_id],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_recent_documents(
        &self,
        limit: usize,
    ) -> Result<Vec<RecentDocumentRecord>, PdfxError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT d.id, d.path, d.filename, d.page_count, d.size_bytes, r.opened_at, r.last_page_viewed, r.zoom_level
             FROM recent_files r
             JOIN documents d ON r.document_id = d.id
             ORDER BY r.opened_at DESC
             LIMIT ?1",
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let path_str: String = row.get(1)?;
                Ok(RecentDocumentRecord {
                    id: row.get(0)?,
                    path: PathBuf::from(path_str),
                    filename: row.get(2)?,
                    page_count: row.get::<_, i64>(3)? as usize,
                    file_size_bytes: row.get::<_, i64>(4)? as u64,
                    last_opened_at: row.get(5)?,
                    last_page_viewed: row.get::<_, i64>(6)? as usize,
                    zoom_level: row.get::<_, f64>(7)? as f32,
                })
            })
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| PdfxError::Database(e.to_string()))?);
        }
        Ok(results)
    }

    pub fn remove_recent_document(&mut self, doc_id: i64) -> Result<(), PdfxError> {
        self.conn
            .execute(
                "DELETE FROM recent_files WHERE document_id = ?1",
                params![doc_id],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn clear_recent_documents(&mut self) -> Result<(), PdfxError> {
        self.conn
            .execute("DELETE FROM recent_files", [])
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    // ==========================================
    // BOOKMARKS
    // ==========================================

    pub fn add_bookmark(
        &mut self,
        doc_id: i64,
        page_number: usize,
        title: &str,
    ) -> Result<i64, PdfxError> {
        self.conn
            .execute(
                "INSERT INTO bookmarks (document_id, page_number, title)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(document_id, page_number) DO UPDATE SET title = excluded.title;",
                params![doc_id, page_number as i64, title],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    pub fn get_bookmarks_for_document(
        &self,
        doc_id: i64,
    ) -> Result<Vec<BookmarkRecord>, PdfxError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, document_id, page_number, title, created_at
                 FROM bookmarks
                 WHERE document_id = ?1
                 ORDER BY page_number ASC",
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![doc_id], |row| {
                Ok(BookmarkRecord {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    page_number: row.get::<_, i64>(2)? as usize,
                    title: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| PdfxError::Database(e.to_string()))?);
        }
        Ok(results)
    }

    pub fn remove_bookmark(&mut self, bookmark_id: i64) -> Result<(), PdfxError> {
        self.conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![bookmark_id])
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn rename_bookmark(&mut self, bookmark_id: i64, new_title: &str) -> Result<(), PdfxError> {
        self.conn
            .execute(
                "UPDATE bookmarks SET title = ?1 WHERE id = ?2",
                params![new_title, bookmark_id],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    // ==========================================
    // JOBS HISTORY
    // ==========================================

    pub fn create_job(&mut self, job: &JobInfo) -> Result<(), PdfxError> {
        let status_str = match job.status {
            JobStatus::Queued => "Queued",
            JobStatus::Running => "Running",
            JobStatus::Completed => "Completed",
            JobStatus::Failed => "Failed",
            JobStatus::Cancelled => "Cancelled",
        };

        self.conn
            .execute(
                "INSERT INTO jobs (id, job_type, status, input_path, output_path, progress, error_message, created_at_unix, completed_at_unix)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    job.id,
                    job.job_type,
                    status_str,
                    job.input_path,
                    job.output_path,
                    job.progress.percentage as f64,
                    job.error_message,
                    job.created_at_unix as i64,
                    job.completed_at_unix.map(|v| v as i64),
                ],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn update_job_progress(
        &mut self,
        job_id: &str,
        progress: f32,
        status: JobStatus,
        error: Option<&str>,
        output_path: Option<&str>,
    ) -> Result<(), PdfxError> {
        let status_str = match status {
            JobStatus::Queued => "Queued",
            JobStatus::Running => "Running",
            JobStatus::Completed => "Completed",
            JobStatus::Failed => "Failed",
            JobStatus::Cancelled => "Cancelled",
        };

        let now = if status.is_finished() {
            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            )
        } else {
            None
        };

        self.conn
            .execute(
                "UPDATE jobs SET
                    progress = ?1,
                    status = ?2,
                    error_message = COALESCE(?3, error_message),
                    output_path = COALESCE(?4, output_path),
                    completed_at_unix = COALESCE(?5, completed_at_unix)
                 WHERE id = ?6",
                params![progress as f64, status_str, error, output_path, now, job_id],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_job(&self, job_id: &str) -> Result<Option<JobInfo>, PdfxError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, job_type, status, input_path, output_path, progress, error_message, created_at_unix, completed_at_unix
                 FROM jobs WHERE id = ?1",
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        match stmt.query_row(params![job_id], |row| {
            let status_str: String = row.get(2)?;
            let status = match status_str.as_str() {
                "Running" => JobStatus::Running,
                "Completed" => JobStatus::Completed,
                "Failed" => JobStatus::Failed,
                "Cancelled" => JobStatus::Cancelled,
                _ => JobStatus::Queued,
            };

            let pct: f64 = row.get(5)?;
            let progress = JobProgress {
                percentage: pct as f32,
                ..Default::default()
            };

            Ok(JobInfo {
                id: row.get(0)?,
                job_type: row.get(1)?,
                status,
                input_path: row.get(3)?,
                output_path: row.get(4)?,
                progress,
                error_message: row.get(6)?,
                created_at_unix: row.get::<_, i64>(7)? as u64,
                completed_at_unix: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
            })
        }) {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PdfxError::Database(e.to_string())),
        }
    }

    pub fn get_recent_jobs(&self, limit: usize) -> Result<Vec<JobInfo>, PdfxError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, job_type, status, input_path, output_path, progress, error_message, created_at_unix, completed_at_unix
                 FROM jobs ORDER BY created_at_unix DESC LIMIT ?1",
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let status_str: String = row.get(2)?;
                let status = match status_str.as_str() {
                    "Running" => JobStatus::Running,
                    "Completed" => JobStatus::Completed,
                    "Failed" => JobStatus::Failed,
                    "Cancelled" => JobStatus::Cancelled,
                    _ => JobStatus::Queued,
                };

                let pct: f64 = row.get(5)?;
                let progress = JobProgress {
                    percentage: pct as f32,
                    ..Default::default()
                };

                Ok(JobInfo {
                    id: row.get(0)?,
                    job_type: row.get(1)?,
                    status,
                    input_path: row.get(3)?,
                    output_path: row.get(4)?,
                    progress,
                    error_message: row.get(6)?,
                    created_at_unix: row.get::<_, i64>(7)? as u64,
                    completed_at_unix: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                })
            })
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| PdfxError::Database(e.to_string()))?);
        }
        Ok(results)
    }

    pub fn clear_completed_jobs(&mut self) -> Result<usize, PdfxError> {
        let count = self
            .conn
            .execute(
                "DELETE FROM jobs WHERE status IN ('Completed', 'Failed', 'Cancelled')",
                [],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(count)
    }

    // ==========================================
    // SETTINGS
    // ==========================================

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, PdfxError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| PdfxError::Database(e.to_string()))?;

        match stmt.query_row(params![key], |row| row.get::<_, String>(0)) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PdfxError::Database(e.to_string())),
        }
    }

    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<(), PdfxError> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value, updated_at)
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
                params![key, value],
            )
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete_setting(&mut self, key: &str) -> Result<(), PdfxError> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", params![key])
            .map_err(|e| PdfxError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn load_app_settings(&self) -> Result<AppSettings, PdfxError> {
        let mut settings = AppSettings::default();

        if let Some(theme) = self.get_setting("theme")? {
            settings.theme = theme;
        }
        if let Some(confirm) = self.get_setting("confirm_destructive")? {
            settings.confirm_destructive = confirm.parse().unwrap_or(true);
        }
        if let Some(cache_mb) = self.get_setting("max_cache_mb")? {
            settings.max_cache_mb = cache_mb.parse().unwrap_or(256);
        }
        if let Some(jobs) = self.get_setting("max_parallel_jobs")? {
            settings.max_parallel_jobs = jobs.parse().unwrap_or(4);
        }
        if let Some(dpi) = self.get_setting("default_dpi")? {
            settings.default_dpi = dpi.parse().unwrap_or(150.0);
        }
        if let Some(lang) = self.get_setting("ocr_language")? {
            settings.ocr_language = lang;
        }

        Ok(settings)
    }

    pub fn save_app_settings(&mut self, settings: &AppSettings) -> Result<(), PdfxError> {
        self.set_setting("theme", &settings.theme)?;
        self.set_setting(
            "confirm_destructive",
            &settings.confirm_destructive.to_string(),
        )?;
        self.set_setting("max_cache_mb", &settings.max_cache_mb.to_string())?;
        self.set_setting("max_parallel_jobs", &settings.max_parallel_jobs.to_string())?;
        self.set_setting("default_dpi", &settings.default_dpi.to_string())?;
        self.set_setting("ocr_language", &settings.ocr_language)?;
        Ok(())
    }
}
