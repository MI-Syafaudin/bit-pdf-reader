pub mod config;
pub mod pipeline;
pub mod preprocess;
pub mod tesseract;
pub mod worker;

pub use config::{OcrConfig, OcrFormat, PageSegmentationMode};
pub use pipeline::{OcrPipeline, OcrProgress, OcrResult};
pub use preprocess::ImagePreprocessor;
pub use tesseract::TesseractRunner;
pub use worker::{OcrJobHandle, OcrWorker};
