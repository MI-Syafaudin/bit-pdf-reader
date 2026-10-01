use image::{GrayImage, Luma};
use pdfx_core::render::RenderedBitmap;

pub struct ImagePreprocessor;

impl ImagePreprocessor {
    /// Converts a RenderedBitmap to an optimized Grayscale/Binarized image for Tesseract OCR.
    pub fn preprocess_for_ocr(bitmap: &RenderedBitmap, binarize: bool) -> GrayImage {
        let width = bitmap.width;
        let height = bitmap.height;
        let mut gray_img = GrayImage::new(width, height);

        if width == 0 || height == 0 {
            return gray_img;
        }

        // Pass 1: Convert RGBA to Grayscale & compute total luminance for adaptive threshold
        let mut sum_lum: u64 = 0;
        let total_pixels = (width * height) as u64;

        for y in 0..height {
            for x in 0..width {
                if let Some((r, g, b, _)) = bitmap.get_pixel(x, y) {
                    // Standard Rec. 601 luma formula
                    let luma = ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8;
                    gray_img.put_pixel(x, y, Luma([luma]));
                    sum_lum += luma as u64;
                }
            }
        }

        // Pass 2: Optional Binarization (Otsu-style dynamic threshold)
        if binarize && total_pixels > 0 {
            let mean_luma = (sum_lum / total_pixels) as u8;
            // Bias threshold slightly below mean to preserve thin text strokes
            let threshold = mean_luma.saturating_sub(10).max(40);

            for y in 0..height {
                for x in 0..width {
                    let pixel = gray_img.get_pixel_mut(x, y);
                    if pixel[0] > threshold {
                        pixel[0] = 255; // White background
                    } else {
                        pixel[0] = 0; // Black text
                    }
                }
            }
        }

        gray_img
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_preprocessing_binarization() {
        let width = 10;
        let height = 10;
        let mut data = vec![0; (width * height * 4) as usize];

        // Fill upper half with bright (white), lower with dark (black)
        for (i, val) in data.iter_mut().enumerate() {
            if i < (width * height * 2) as usize {
                *val = 240;
            } else {
                *val = 20;
            }
        }

        let bmp = RenderedBitmap::new(width, height, data);
        let processed = ImagePreprocessor::preprocess_for_ocr(&bmp, true);

        assert_eq!(processed.width(), 10);
        assert_eq!(processed.height(), 10);

        // Top pixel should be 255 (white)
        assert_eq!(processed.get_pixel(0, 0)[0], 255);
        // Bottom pixel should be 0 (black)
        assert_eq!(processed.get_pixel(0, 9)[0], 0);
    }
}
