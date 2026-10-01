use crate::traits::DocumentHandle;
use lopdf::{Document, Object};
use pdfx_core::document::DocumentMetadata;
use pdfx_core::errors::PdfxError;
use pdfx_core::page::PageDimensions;
use pdfx_core::render::{RenderOptions, RenderedBitmap};
use pdfx_core::search::SearchMatch;
use std::path::{Path, PathBuf};

pub struct LopdfDocumentHandle {
    path: PathBuf,
    doc: Document,
    file_size_bytes: u64,
}

impl LopdfDocumentHandle {
    pub fn open(path: &Path, _password: Option<&str>) -> Result<Self, PdfxError> {
        if !path.exists() {
            return Err(PdfxError::IoError(format!("File not found: {:?}", path)));
        }

        let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let doc = Document::load(path).map_err(|e| PdfxError::CorruptedDocument {
            path: path.to_path_buf(),
            reason: format!("Failed to parse PDF document structure: {}", e),
        })?;

        Ok(Self {
            path: path.to_path_buf(),
            doc,
            file_size_bytes,
        })
    }

    pub fn from_document(path: PathBuf, doc: Document) -> Self {
        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Self {
            path,
            doc,
            file_size_bytes,
        }
    }

    fn extract_info_string(&self, key: &[u8]) -> Option<String> {
        let info_id = self.doc.trailer.get(b"Info").ok()?.as_reference().ok()?;
        let info_obj = self.doc.get_object(info_id).ok()?;
        let dict = info_obj.as_dict().ok()?;
        let obj = dict.get(key).ok()?;

        match obj {
            Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).trim().to_string()),
            Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).trim().to_string()),
            _ => None,
        }
    }
}

impl DocumentHandle for LopdfDocumentHandle {
    fn page_count(&self) -> usize {
        self.doc.get_pages().len()
    }

    fn metadata(&self) -> Result<DocumentMetadata, PdfxError> {
        let page_count = self.page_count();
        let is_encrypted = self.doc.is_encrypted();

        let title = self.extract_info_string(b"Title");
        let author = self.extract_info_string(b"Author");
        let subject = self.extract_info_string(b"Subject");
        let keywords = self.extract_info_string(b"Keywords");
        let creator = self.extract_info_string(b"Creator");
        let producer = self.extract_info_string(b"Producer");
        let creation_date = self.extract_info_string(b"CreationDate");
        let modification_date = self.extract_info_string(b"ModDate");

        Ok(DocumentMetadata {
            title,
            author,
            subject,
            keywords,
            creator,
            producer,
            creation_date,
            modification_date,
            pdf_version: Some(self.doc.version.clone()),
            is_encrypted,
            page_count,
            file_size_bytes: self.file_size_bytes,
            permissions: Default::default(),
        })
    }

    fn page_dimensions(&self, page_index: usize) -> Result<PageDimensions, PdfxError> {
        let total = self.page_count();
        if page_index == 0 || page_index > total {
            return Err(PdfxError::PageOutOfBounds {
                index: page_index,
                total,
            });
        }

        let pages = self.doc.get_pages();
        let page_id = pages
            .get(&(page_index as u32))
            .ok_or(PdfxError::PageOutOfBounds {
                index: page_index,
                total,
            })?;

        let page_obj = self
            .doc
            .get_object(*page_id)
            .map_err(|e| PdfxError::Engine(format!("Failed to retrieve page object: {}", e)))?;

        let dict = page_obj
            .as_dict()
            .map_err(|e| PdfxError::Engine(format!("Invalid page dictionary: {}", e)))?;

        let (mut width, mut height) = (595.0, 842.0); // Default A4 in points
        if let Ok(box_obj) = dict.get(b"MediaBox").or_else(|_| dict.get(b"CropBox")) {
            if let Ok(arr) = box_obj.as_array() {
                if arr.len() == 4 {
                    let x0 = arr[0].as_float().unwrap_or(0.0);
                    let y0 = arr[1].as_float().unwrap_or(0.0);
                    let x1 = arr[2].as_float().unwrap_or(595.0);
                    let y1 = arr[3].as_float().unwrap_or(842.0);
                    width = (x1 - x0).abs();
                    height = (y1 - y0).abs();
                }
            }
        }

        let rotation = dict
            .get(b"Rotate")
            .ok()
            .and_then(|o| o.as_i64().ok())
            .map(|r| r.rem_euclid(360) as u16)
            .unwrap_or(0);

        Ok(PageDimensions::new(width, height, rotation))
    }

    fn extract_text(&self, page_index: usize) -> Result<String, PdfxError> {
        let total = self.page_count();
        if page_index == 0 || page_index > total {
            return Err(PdfxError::PageOutOfBounds {
                index: page_index,
                total,
            });
        }

        let text = self
            .doc
            .extract_text(&[page_index as u32])
            .unwrap_or_default();

        Ok(text)
    }

    fn search_text(
        &self,
        page_index: usize,
        query: &str,
        case_sensitive: bool,
    ) -> Result<Vec<SearchMatch>, PdfxError> {
        let text = self.extract_text(page_index)?;
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let mut matches = Vec::new();
        let target_query = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        for line in text.lines() {
            let search_line = if case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            if search_line.contains(&target_query) {
                matches.push(SearchMatch {
                    page_number: page_index,
                    match_text: query.to_string(),
                    context_snippet: line.trim().to_string(),
                });
            }
        }

        Ok(matches)
    }

    fn render_page(
        &self,
        page_index: usize,
        options: &RenderOptions,
    ) -> Result<RenderedBitmap, PdfxError> {
        #[cfg(feature = "pdfium")]
        {
            if let Ok(bmp) =
                crate::pdfium_engine::render_with_pdfium(&self.path, page_index, options)
            {
                return Ok(bmp);
            }
        }

        // Clean fallback: generate an informative visual placeholder bitmap
        let width = options.target_width.max(100);
        let height = options.target_height.max(100);
        let mut data = vec![245; (width * height * 4) as usize]; // Light gray background

        // Draw dark border
        let border_color = [80, 80, 80, 255];
        for x in 0..width {
            set_pixel(&mut data, width, x, 0, border_color);
            set_pixel(&mut data, width, x, height - 1, border_color);
        }
        for y in 0..height {
            set_pixel(&mut data, width, 0, y, border_color);
            set_pixel(&mut data, width, width - 1, y, border_color);
        }

        Ok(RenderedBitmap::new(width, height, data))
    }
}

#[inline]
fn set_pixel(data: &mut [u8], stride_width: u32, x: u32, y: u32, rgba: [u8; 4]) {
    let idx = ((y * stride_width + x) * 4) as usize;
    if idx + 3 < data.len() {
        data[idx..idx + 4].copy_from_slice(&rgba);
    }
}
