use pdfx_core::render::RenderedBitmap;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct HalfBlockWidget<'a> {
    pub bitmap: &'a RenderedBitmap,
}

impl<'a> HalfBlockWidget<'a> {
    pub fn new(bitmap: &'a RenderedBitmap) -> Self {
        Self { bitmap }
    }
}

impl<'a> Widget for HalfBlockWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 || self.bitmap.width == 0 || self.bitmap.height == 0
        {
            return;
        }

        let bmp_w = self.bitmap.width as f32;
        let bmp_h = self.bitmap.height as f32;
        let term_w = area.width as f32;
        let term_h = (area.height * 2) as f32; // 2 vertical subpixels per terminal cell

        let scale_x = bmp_w / term_w;
        let scale_y = bmp_h / term_h;

        for row in 0..area.height {
            let y_top = ((row * 2) as f32 * scale_y) as u32;
            let y_bot = (((row * 2 + 1) as f32) * scale_y) as u32;

            for col in 0..area.width {
                let x = (col as f32 * scale_x) as u32;

                let top_pixel = self
                    .bitmap
                    .get_pixel(
                        x.min(self.bitmap.width - 1),
                        y_top.min(self.bitmap.height - 1),
                    )
                    .unwrap_or((0, 0, 0, 255));

                let bot_pixel = self
                    .bitmap
                    .get_pixel(
                        x.min(self.bitmap.width - 1),
                        y_bot.min(self.bitmap.height - 1),
                    )
                    .unwrap_or((0, 0, 0, 255));

                let fg = Color::Rgb(top_pixel.0, top_pixel.1, top_pixel.2);
                let bg = Color::Rgb(bot_pixel.0, bot_pixel.1, bot_pixel.2);

                let cell_x = area.x + col;
                let cell_y = area.y + row;

                if let Some(cell) = buf.cell_mut((cell_x, cell_y)) {
                    cell.set_symbol("▀")
                        .set_style(Style::default().fg(fg).bg(bg));
                }
            }
        }
    }
}
