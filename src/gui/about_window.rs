// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! Module for about window
//!

use egui::{Context, CursorIcon, Image, include_image, OpenUrl, Pos2, Vec2, Window};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const WINDOW_SIZE: Vec2 = Vec2 { x: 192.0, y: 256.0 };

/// Display about window
pub fn about_window(ctx: &Context, open: &mut bool) {
    // Top left corner position for a fully centered window
    let window_topleft = ctx.input(|i| i.screen_rect.size()) / 2.0 - WINDOW_SIZE / 2.0;
    // Same, but a Pos2 instead of Vec2. Position is nudged up slightly.
    let window_pos = Pos2 { x: window_topleft.x, y: window_topleft.y - 64.0 };

    let window = Window::new("")
        .open(open)
        .collapsible(false)
        .fixed_pos(window_pos)
        .fixed_size(WINDOW_SIZE)
        ;

    window.show(ctx, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());

        ui.vertical_centered(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::ZERO;
            ui.add_space(16.0);

            // Title
            ui.add(Image::new(include_image!("../assets/titobox.png")).fit_to_original_size(1.0));
            ui.heading("TiTo");
            ui.small(format!("titomachine {VERSION}"));
            ui.add_space(16.0);

            // Info
            if ui.small_button("Project repository").clicked() {
                ui.output_mut(|o| o.open_url = Some(OpenUrl {
                    url: env!("CARGO_PKG_REPOSITORY").into(),
                    new_tab: false,
                }));
            }
            ui.add_space(24.0);

            // License
            ui.small("© 2022-2024 sevonj");
            ui.small("Titomachine is licensed under MPL-2.0");
            if ui.small("https://mozilla.org/MPL/2.0/")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked() {
                ui.output_mut(|o| o.open_url = Some(OpenUrl {
                    url: "https://mozilla.org/MPL/2.0/".into(),
                    new_tab: true,
                }));
            }
        });
    });
}

