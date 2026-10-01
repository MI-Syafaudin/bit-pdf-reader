use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FitMode {
    #[default]
    FitPage,
    FitWidth,
    FitHeight,
    ActualSize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderOptions {
    pub target_width: u32,
    pub target_height: u32,
    pub dpi: f32,
    pub rotate_degrees: u16,
    pub fit_mode: FitMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            target_width: 800,
            target_height: 1000,
            dpi: 150.0,
            rotate_degrees: 0,
            fit_mode: FitMode::FitPage,
        }
    }
}

/// Raw RGBA bitmap in memory (4 bytes per pixel: Red, Green, Blue, Alpha)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedBitmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl RenderedBitmap {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        debug_assert_eq!(
            data.len(),
            (width * height * 4) as usize,
            "Bitmap data size must match width * height * 4 bytes"
        );
        Self {
            width,
            height,
            data,
        }
    }

    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.data.len() {
            Some((
                self.data[index],
                self.data[index + 1],
                self.data[index + 2],
                self.data[index + 3],
            ))
        } else {
            None
        }
    }

    /// Converts RGBA buffer to RGB buffer (drops alpha channel)
    pub fn to_rgb(&self) -> Vec<u8> {
        let pixel_count = (self.width * self.height) as usize;
        let mut rgb = Vec::with_capacity(pixel_count * 3);
        let (chunks, _) = self.data.as_chunks::<4>();
        for chunk in chunks {
            rgb.extend_from_slice(&chunk[0..3]);
        }
        rgb
    }
}

/// Formatted text span positioned in the PDF coordinate space
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TextPageLayout {
    pub page_number: usize,
    pub spans: Vec<TextSpan>,
    pub raw_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendered_bitmap_pixel_access() {
        let width = 2;
        let height = 2;
        let data = vec![
            255, 0, 0, 255, // (0,0) Red
            0, 255, 0, 255, // (1,0) Green
            0, 0, 255, 255, // (0,1) Blue
            255, 255, 255, 255, // (1,1) White
        ];

        let bmp = RenderedBitmap::new(width, height, data);
        assert_eq!(bmp.byte_size(), 16);
        assert_eq!(bmp.get_pixel(0, 0), Some((255, 0, 0, 255)));
        assert_eq!(bmp.get_pixel(1, 0), Some((0, 255, 0, 255)));
        assert_eq!(bmp.get_pixel(0, 1), Some((0, 0, 255, 255)));
        assert_eq!(bmp.get_pixel(1, 1), Some((255, 255, 255, 255)));
        assert_eq!(bmp.get_pixel(2, 2), None);

        let rgb = bmp.to_rgb();
        assert_eq!(rgb.len(), 12);
        assert_eq!(&rgb[0..3], &[255, 0, 0]);
    }
}
