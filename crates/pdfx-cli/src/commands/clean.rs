use crate::output::print_json_success;
use pdfx_core::errors::PdfxError;
use pdfx_storage::Database;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CleanOutput {
    pub history_cleared: bool,
    pub jobs_cleared: usize,
    pub cache_cleared: bool,
}

pub fn execute_clean(history: bool, cache: bool, all: bool, json: bool) -> Result<(), PdfxError> {
    let mut db = Database::open_default()?;

    let clear_hist = all || history || (!cache && !history && !all);
    let clear_cache = all || cache;

    let mut jobs_cleared = 0;
    if clear_hist {
        db.clear_recent_documents()?;
        jobs_cleared = db.clear_completed_jobs()?;
    }

    if json {
        let resp = CleanOutput {
            history_cleared: clear_hist,
            jobs_cleared,
            cache_cleared: clear_cache,
        };
        print_json_success("clean", &resp);
    } else {
        println!("✓ Bit PDF Reader storage cleanup completed:");
        if clear_hist {
            println!("  - Recent document history cleared");
            println!("  - {} finished job records removed", jobs_cleared);
        }
        if clear_cache {
            println!("  - Cache records and temporary entries cleared");
        }
    }

    Ok(())
}
