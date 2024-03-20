// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! Home of the configuration struct.
//!

use crate::gui::Radix;

/// Configuration struct. For persistent settings.
/// This is automatically serialized and deserialized by serde.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    // --- Memory Explorer
    pub memview_visible: bool,
    /// Memory view follows PC register while playing
    pub memview_follow_pc: bool,
    /// Which base to show address in
    pub memview_addr_base: Radix,
    /// Which base to show value in
    pub memview_value_base: Radix,
}

impl Default for Config {
    fn default() -> Self {
        todo!()
    }
}