///
/// LegacyIO
/// CRT and KBD
///
use super::GUIDevice;
use egui::{Context, Layout, Ui};
use egui_extras::RetainedImage;
use image::{ImageBuffer, Rgba};
use num_traits::clamp;
use std::sync::mpsc::Receiver;

pub(crate) struct GUIDevDisplay {
    rx: Receiver<Vec<Rgba<u8>>>,
    framebuffer: Vec<Rgba<u8>>,
    displaybuf: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    image: Option<RetainedImage>,
}

impl GUIDevDisplay {
    pub(crate) fn new(rx: Receiver<Vec<Rgba<u8>>>) -> Self {
        Self {
            rx,
            framebuffer: vec![image::Rgba([0, 0, 0, 255,]); 120 * 160],
            displaybuf: None,
            image: None,
        }
    }
}
impl GUIDevice for GUIDevDisplay {
    fn gui_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        {
            // Determine image size based on available w / h, whichever fits a smaller image
            let target_h = clamp(ui.available_height(), 120., 400.); // size limited for performance
            let target_w = clamp(ui.available_width(), 160., f32::INFINITY);
            let w;
            let h;
            if target_w > target_h * (160. / 120.) {
                w = (target_h * (160. / 120.)) as u32;
                h = target_h as u32;
            } else {
                w = target_w as u32;
                h = (target_w * (120. / 160.)) as u32;
            }
            ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                self.displaybuf = Some(image::ImageBuffer::new(w, h));
                // This is a terribly inefficient way to make the image
                // TODO: figure out how to just rescale the original res pic.
                for (x, y, pixels) in self.displaybuf.as_mut().unwrap().enumerate_pixels_mut() {
                    // px_off = px_x + px_y * 160
                    let px_off = (x * 160 / w) + (y * 120 / h) * 160;
                    *pixels = self.framebuffer[px_off as usize];
                }
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &self.displaybuf.as_ref().unwrap(),
                );
                let render_result = RetainedImage::from_color_image("0.png", color_image);
                self.image = Some(render_result);
                if let Some(img) = &self.image {
                    img.show(ui);
                }
            });
        }
    }

    fn reset(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
    }
    fn update(&mut self) {
        while let Ok(vec) = self.rx.try_recv() {
            self.framebuffer = vec;
        }
    }
}
