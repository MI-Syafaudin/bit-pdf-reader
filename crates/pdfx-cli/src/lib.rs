pub mod args;
pub mod commands;
pub mod output;

use pdfx_core::errors::PdfxError;

pub fn exit_code_for_error(err: &PdfxError) -> i32 {
    match err {
        PdfxError::InvalidPageRange { .. }
        | PdfxError::PageOutOfBounds { .. }
        | PdfxError::InvalidRotation { .. }
        | PdfxError::InvalidOperation { .. }
        | PdfxError::Config(_) => 2,

        PdfxError::IoError(msg) => {
            let lower = msg.to_lowercase();
            if lower.contains("not found") || lower.contains("no such file") {
                3
            } else if lower.contains("permission")
                || lower.contains("read-only")
                || lower.contains("cannot write")
            {
                4
            } else {
                1
            }
        }

        PdfxError::CorruptedDocument { .. }
        | PdfxError::PasswordRequired(_)
        | PdfxError::InvalidPassword
        | PdfxError::Engine(_) => 5,

        PdfxError::Ocr(_) => 6,
        PdfxError::Cancelled => 7,
        PdfxError::Database(_) => 1,
    }
}
