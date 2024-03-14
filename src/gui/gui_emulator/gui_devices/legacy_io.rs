///
/// LegacyIO
/// CRT and KBD
///
use super::GUIDevice;
use egui::{Color32, Context, FontId, Frame, RichText, TextEdit, Ui};
use std::sync::mpsc::{Receiver, Sender};
const FONT_PANELNAME: FontId = FontId::monospace(12.0);

pub(crate) struct GUIDevLegacyIO {
    rx_crt: Receiver<i32>,
    tx_kbd: Sender<i32>,
    rx_kbdreq: Receiver<()>,
    buf_kbd: String,
    buf_crt: String,

    waiting_for_in: bool,
}

impl GUIDevLegacyIO {
    pub(crate) fn new(rx_crt: Receiver<i32>, tx_kbd: Sender<i32>, rx_kbdreq: Receiver<()>) -> Self {
        Self {
            rx_crt,
            tx_kbd,
            rx_kbdreq,
            buf_kbd: String::new(),
            buf_crt: "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n".to_owned(),
            waiting_for_in: false,
        }
    }

    fn crt_out(&mut self, n: i32) {
        self.buf_crt = n.to_string() + "\n" + self.buf_crt.as_str();
        // Add a line to beginning
        self.buf_crt = self // Remove last line
            .buf_crt
            .lines()
            .take(16)
            .map(|s| s.to_string() + "\n")
            .collect();
    }

    /// If the emulator thread is waiting for input, it's frozen until it receives something.
    /// This will free the emulator by sending a dummy value.
    pub(crate) fn clear_kbd(&mut self) {
        if !self.waiting_for_in {
            return;
        }
        let _ = self.tx_kbd.send(0);
    }
}
impl GUIDevice for GUIDevLegacyIO {
    fn gui_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        // CRT Panel

        ui.label("=CRT");
        Frame::side_top_panel(&ctx.style())
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                ui.label(self.buf_crt.as_str());
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0))
            });
        ui.separator();

        // KBD Panel
        ui.add_enabled_ui(self.waiting_for_in, |ui| {
            ui.label(
                RichText::new("=KBD")
                    .font(FONT_PANELNAME.clone())
                    .color(Color32::WHITE),
            );
            TextEdit::singleline(&mut self.buf_kbd)
                .hint_text("Type a number")
                .show(ui);
            if ui.button("Send").clicked() {
                if self.buf_kbd.parse::<i32>().is_ok() {
                    let _ = self.tx_kbd.send(self.buf_kbd.parse::<i32>().unwrap());
                    self.buf_kbd = String::new();
                    self.waiting_for_in = false;
                } else {
                    self.buf_kbd = "Invalid input!".to_owned();
                }
            }
        });
    }

    fn reset(&mut self) {
        self.buf_kbd = String::new();
        self.buf_crt = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n".to_owned();
        self.clear_kbd();
    }

    fn update(&mut self) {
        if let Ok(n) = self.rx_crt.try_recv() {
            self.crt_out(n)
        }
        if let Ok(_) = self.rx_kbdreq.try_recv() {
            self.waiting_for_in = true
        }
    }
}
