// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! Home of the configuration struct.
//!

use std::env::current_dir;
use std::path::PathBuf;
use crate::FreqMagnitude;
use crate::gui::Radix;

/// Configuration struct. For persistent settings.
/// This is automatically serialized and deserialized by serde.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    // --- General
    /// Remember current working directory for file dialogs.
    pub workdir: PathBuf,

    // --- Emulator
    pub emu_cpuspeedmul: FreqMagnitude,
    pub emu_speed: f32,

    // --- Memory Explorer
    pub memview_visible: bool,
    /// Memory view follows PC register while playing
    pub memview_follow_pc: bool,
    /// Which base to show address in
    pub memview_addr_base: Radix,
    /// Which base to show value in
    pub memview_value_base: Radix,
    /// Breakpoint settings belong to Memory Viewer
    pub memview_breakpoints_enabled: bool,

    // --- Graphics Display
    pub display_visible: bool,

    // --- CPU State
    pub cpuview_regs_base: Radix,

}

impl Default for Config {
    fn default() -> Self {
        Config {
            workdir: current_dir().unwrap(),

            emu_speed: 10.,
            emu_cpuspeedmul: FreqMagnitude::Hz,

            memview_visible: true,
            memview_follow_pc: true,
            memview_addr_base: Default::default(),
            memview_value_base: Default::default(),
            memview_breakpoints_enabled: false,

            display_visible: false,

            cpuview_regs_base: Default::default(),
        }
    }
}