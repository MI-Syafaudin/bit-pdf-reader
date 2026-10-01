use pdfx_core::errors::PdfxError;
use pdfx_core::render::{RenderOptions, RenderedBitmap};
use std::path::Path;

#[cfg(feature = "pdfium")]
pub fn render_with_pdfium(
    path: &Path,
    page_index: usize,
    options: &RenderOptions,
) -> Result<RenderedBitmap, PdfxError> {
    use pdfium_render::prelude::*;

    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| Pdfium::bind_to_library("libpdfium.so"))
        .map_err(|e| {
            PdfxError::Engine(format!(
                "PDFium library (libpdfium.so) not found in system library path: {}",
                e
            ))
        })?;

    let pdfium = Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| PdfxError::Engine(format!("PDFium failed to load document: {}", e)))?;

    let pages = document.pages();
    let total = pages.len() as usize;
    if page_index == 0 || page_index > total {
        return Err(PdfxError::PageOutOfBounds {
            index: page_index,
            total,
        });
    }

    let page = pages.get((page_index - 1) as u16).map_err(|e| {
        PdfxError::Engine(format!(
            "PDFium failed to retrieve page {}: {}",
            page_index, e
        ))
    })?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(options.target_width as i32)
        .set_maximum_height(options.target_height as i32);

    let bitmap = page.render_with_config(&render_config).map_err(|e| {
        PdfxError::Engine(format!(
            "PDFium failed to render page {}: {}",
            page_index, e
        ))
    })?;

    let width = bitmap.width() as u32;
    let height = bitmap.height() as u32;
    let rgba_bytes = bitmap.as_rgba_bytes().to_vec();

    Ok(RenderedBitmap::new(width, height, rgba_bytes))
}

#[cfg(not(feature = "pdfium"))]
pub fn render_with_pdfium(
    _path: &Path,
    _page_index: usize,
    _options: &RenderOptions,
) -> Result<RenderedBitmap, PdfxError> {
    Err(PdfxError::Engine(
        "PDFium feature is disabled. Compile with --features pdfium to enable raster rendering."
            .to_string(),
    ))
}
