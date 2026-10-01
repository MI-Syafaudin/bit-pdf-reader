use pdfx_core::cancellation::CancellationToken;
use pdfx_core::document::DocumentMetadata;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::{PageDimensions, PageRange};
use pdfx_core::render::{RenderOptions, RenderedBitmap};
use pdfx_core::search::SearchMatch;
use std::path::{Path, PathBuf};

pub trait DocumentHandle: Send + Sync {
    fn page_count(&self) -> usize;
    fn metadata(&self) -> Result<DocumentMetadata, PdfxError>;
    fn page_dimensions(&self, page_index: usize) -> Result<PageDimensions, PdfxError>;
    fn render_page(
        &self,
        page_index: usize,
        options: &RenderOptions,
    ) -> Result<RenderedBitmap, PdfxError>;
    fn extract_text(&self, page_index: usize) -> Result<String, PdfxError>;
    fn search_text(
        &self,
        page_index: usize,
        query: &str,
        case_sensitive: bool,
    ) -> Result<Vec<SearchMatch>, PdfxError>;
}

pub trait PdfEngine: Send + Sync {
    fn open_document(
        &self,
        path: &Path,
        password: Option<&str>,
    ) -> Result<Box<dyn DocumentHandle>, PdfxError>;
}

pub trait DocumentMutator: Send + Sync {
    fn merge(
        &self,
        inputs: &[PathBuf],
        output: &Path,
        cancel: &CancellationToken,
    ) -> Result<(), PdfxError>;
    fn split(
        &self,
        input: &Path,
        ranges: &[PageRange],
        output_dir: &Path,
        cancel: &CancellationToken,
    ) -> Result<Vec<PathBuf>, PdfxError>;
    fn rotate(
        &self,
        input: &Path,
        pages: &PageRange,
        degrees: u16,
        output: &Path,
    ) -> Result<(), PdfxError>;
    fn delete_pages(&self, input: &Path, pages: &PageRange, output: &Path)
        -> Result<(), PdfxError>;
    fn extract_pages(
        &self,
        input: &Path,
        pages: &PageRange,
        output: &Path,
    ) -> Result<(), PdfxError>;
    fn reorder_pages(
        &self,
        input: &Path,
        new_order: &[usize],
        output: &Path,
    ) -> Result<(), PdfxError>;
    fn compress(&self, input: &Path, output: &Path) -> Result<(), PdfxError>;
}
