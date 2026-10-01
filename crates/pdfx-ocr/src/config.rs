use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OcrFormat {
    #[default]
    SearchablePdf,
    PlainText,
    Hocr,
    Tsv,
}

impl OcrFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            OcrFormat::SearchablePdf => "pdf",
            OcrFormat::PlainText => "txt",
            OcrFormat::Hocr => "hocr",
            OcrFormat::Tsv => "tsv",
        }
    }

    pub fn tesseract_config_flag(&self) -> &'static str {
        match self {
            OcrFormat::SearchablePdf => "pdf",
            OcrFormat::PlainText => "txt",
            OcrFormat::Hocr => "hocr",
            OcrFormat::Tsv => "tsv",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageSegmentationMode {
    Auto,         // 3: Fully automatic page segmentation
    SingleColumn, // 4: Assume a single column of text of variable sizes
    SingleBlock,  // 6: Assume a single uniform block of text
    SingleLine,   // 7: Treat the image as a single text line
    SparseText,   // 11: Find as much text as possible in no particular order
}

impl PageSegmentationMode {
    pub fn psm_number(&self) -> &'static str {
        match self {
            PageSegmentationMode::Auto => "3",
            PageSegmentationMode::SingleColumn => "4",
            PageSegmentationMode::SingleBlock => "6",
            PageSegmentationMode::SingleLine => "7",
            PageSegmentationMode::SparseText => "11",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    pub language: String,
    pub format: OcrFormat,
    pub dpi: u32,
    pub psm: Option<PageSegmentationMode>,
    pub binarize: bool,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            format: OcrFormat::SearchablePdf,
            dpi: 300,
            psm: Some(PageSegmentationMode::Auto),
            binarize: true,
        }
    }
}
