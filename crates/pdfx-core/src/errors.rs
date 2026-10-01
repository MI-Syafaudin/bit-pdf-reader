use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PdfxError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Corrupted PDF document at '{path}': {reason}")]
    CorruptedDocument { path: PathBuf, reason: String },

    #[error("Password required for encrypted document at '{0}'")]
    PasswordRequired(PathBuf),

    #[error("Invalid password provided for encrypted document")]
    InvalidPassword,

    #[error("Invalid page range expression '{input}': {reason}")]
    InvalidPageRange { input: String, reason: String },

    #[error("Page index {index} is out of bounds (document has {total} pages)")]
    PageOutOfBounds { index: usize, total: usize },

    #[error("Invalid rotation angle {degrees}°. Allowed angles are 90°, 180°, and 270°")]
    InvalidRotation { degrees: u16 },

    #[error("Invalid operation: {reason}")]
    InvalidOperation { reason: String },

    #[error("PDF engine error: {0}")]
    Engine(String),

    #[error("OCR engine error: {0}")]
    Ocr(String),

    #[error("Database persistence error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Operation was cancelled by user")]
    Cancelled,
}

impl From<std::io::Error> for PdfxError {
    fn from(err: std::io::Error) -> Self {
        PdfxError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let err = PdfxError::InvalidRotation { degrees: 45 };
        assert_eq!(
            err.to_string(),
            "Invalid rotation angle 45°. Allowed angles are 90°, 180°, and 270°"
        );

        let err = PdfxError::PageOutOfBounds {
            index: 15,
            total: 10,
        };
        assert_eq!(
            err.to_string(),
            "Page index 15 is out of bounds (document has 10 pages)"
        );
    }
}
