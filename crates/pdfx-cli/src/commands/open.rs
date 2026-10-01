use pdfx_core::errors::PdfxError;
use std::path::PathBuf;

pub fn execute_open(file: Option<PathBuf>) -> Result<(), PdfxError> {
    if let Some(ref path) = file {
        if !path.exists() {
            return Err(PdfxError::IoError(format!(
                "File not found: {}",
                path.display()
            )));
        }
    }

    pdfx_tui::run_tui(file)
        .map_err(|e| PdfxError::Engine(format!("Failed to run interactive TUI: {}", e)))
}
