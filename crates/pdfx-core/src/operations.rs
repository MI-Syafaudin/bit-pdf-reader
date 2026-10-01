use crate::errors::PdfxError;
use crate::page::PageRange;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeOperation {
    pub input_files: Vec<PathBuf>,
    pub output_file: PathBuf,
}

impl MergeOperation {
    pub fn new(input_files: Vec<PathBuf>, output_file: PathBuf) -> Self {
        Self {
            input_files,
            output_file,
        }
    }

    pub fn validate(&self) -> Result<(), PdfxError> {
        if self.input_files.len() < 2 {
            return Err(PdfxError::InvalidOperation {
                reason: "At least 2 PDF files are required for merging".to_string(),
            });
        }

        for input in &self.input_files {
            if input == &self.output_file {
                return Err(PdfxError::InvalidOperation {
                    reason: format!(
                        "Output file '{:?}' cannot be the same as an input file",
                        self.output_file
                    ),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SplitOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub output_dir: PathBuf,
    pub filename_prefix: String,
}

impl SplitOperation {
    pub fn new(
        input_file: PathBuf,
        page_range: PageRange,
        output_dir: PathBuf,
        filename_prefix: Option<String>,
    ) -> Self {
        let prefix = filename_prefix.unwrap_or_else(|| "page".to_string());
        Self {
            input_file,
            page_range,
            output_dir,
            filename_prefix: prefix,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub output_file: PathBuf,
}

impl ExtractOperation {
    pub fn new(input_file: PathBuf, page_range: PageRange, output_file: PathBuf) -> Self {
        Self {
            input_file,
            page_range,
            output_file,
        }
    }

    pub fn validate(&self) -> Result<(), PdfxError> {
        if self.input_file == self.output_file {
            return Err(PdfxError::InvalidOperation {
                reason: "Output file cannot be the same as the input file".to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RotateOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub degrees: u16, // 90, 180, 270
    pub output_file: PathBuf,
}

impl RotateOperation {
    pub fn new(
        input_file: PathBuf,
        page_range: PageRange,
        degrees: u16,
        output_file: PathBuf,
    ) -> Result<Self, PdfxError> {
        if degrees != 90 && degrees != 180 && degrees != 270 {
            return Err(PdfxError::InvalidRotation { degrees });
        }
        Ok(Self {
            input_file,
            page_range,
            degrees,
            output_file,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub output_file: PathBuf,
}

impl DeleteOperation {
    pub fn new(input_file: PathBuf, page_range: PageRange, output_file: PathBuf) -> Self {
        Self {
            input_file,
            page_range,
            output_file,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReorderOperation {
    pub input_file: PathBuf,
    pub new_order: Vec<usize>, // 1-indexed list of page positions
    pub output_file: PathBuf,
}

impl ReorderOperation {
    pub fn new(
        input_file: PathBuf,
        new_order: Vec<usize>,
        output_file: PathBuf,
    ) -> Result<Self, PdfxError> {
        if new_order.is_empty() {
            return Err(PdfxError::InvalidOperation {
                reason: "Page order cannot be empty".to_string(),
            });
        }
        if new_order.contains(&0) {
            return Err(PdfxError::InvalidOperation {
                reason: "Page numbers are 1-indexed and must be >= 1".to_string(),
            });
        }
        Ok(Self {
            input_file,
            new_order,
            output_file,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CompressProfile {
    #[default]
    Balanced,
    Screen,
    Ebook,
    Printer,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressOperation {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub profile: CompressProfile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TextFormat {
    #[default]
    PlainText,
    Json,
    Jsonl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextExtractOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub format: TextFormat,
    pub output_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ImageExportFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderExportOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub format: ImageExportFormat,
    pub dpi: f32,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OcrOperation {
    pub input_file: PathBuf,
    pub page_range: PageRange,
    pub language: String,
    pub output_file: PathBuf,
}

/// Real-time progress update for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationProgress {
    pub current_step: usize,
    pub total_steps: usize,
    pub message: String,
    pub percentage: f32,
    pub eta_seconds: Option<u64>,
}

impl OperationProgress {
    pub fn new(current_step: usize, total_steps: usize, message: impl Into<String>) -> Self {
        let total = total_steps.max(1);
        let pct = (current_step as f32 / total as f32) * 100.0;
        Self {
            current_step,
            total_steps,
            message: message.into(),
            percentage: pct.min(100.0),
            eta_seconds: None,
        }
    }

    pub fn with_eta(mut self, eta_seconds: u64) -> Self {
        self.eta_seconds = Some(eta_seconds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_validation() {
        let op = MergeOperation::new(vec![PathBuf::from("a.pdf")], PathBuf::from("out.pdf"));
        assert!(op.validate().is_err());

        let op2 = MergeOperation::new(
            vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")],
            PathBuf::from("a.pdf"),
        );
        assert!(op2.validate().is_err());

        let op3 = MergeOperation::new(
            vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")],
            PathBuf::from("c.pdf"),
        );
        assert!(op3.validate().is_ok());
    }

    #[test]
    fn test_rotate_validation() {
        assert!(RotateOperation::new(
            PathBuf::from("in.pdf"),
            PageRange::All,
            90,
            PathBuf::from("out.pdf")
        )
        .is_ok());

        assert!(RotateOperation::new(
            PathBuf::from("in.pdf"),
            PageRange::All,
            45,
            PathBuf::from("out.pdf")
        )
        .is_err());
    }

    #[test]
    fn test_progress_calculation() {
        let p = OperationProgress::new(50, 100, "Processing page 50");
        assert_eq!(p.percentage, 50.0);
        assert_eq!(p.current_step, 50);
        assert_eq!(p.total_steps, 100);
    }
}
