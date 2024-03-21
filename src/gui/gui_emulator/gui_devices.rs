pub(crate) mod legacy_io;
use egui::{Context, Ui};

///
/// gui/gui_emulator/gui_devices.rs
///
/// GUI side of devices
///

pub(crate) trait GUIDevice {
    fn gui_panel(&mut self, ctx: &Context, ui: &mut Ui);
    fn reset(&mut self);
    fn update(&mut self);
}
