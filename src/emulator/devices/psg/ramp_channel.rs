//!
//! Ramp wave generator
//!
//!

use super::{envelope::Envelope, AudioChannel, SAMPLE_RATE};

/// Ramp wave generator
pub struct RampChannel {
    /// Position in current cycle. Range: 0.0 to 1.0
    cycletimer: f32,
    /// Frequency in Hz
    freq: f32,
    /// Volume range: 0.0 to 1.0
    vol: f32,
    /// Duty Cycle range: 0.0 to 1.0. Maps to 50% to 100%
    dc: f32,
    /// Envelope.
    pub env: Envelope,
}
impl RampChannel {
    /// Expected input range: 0..=i32::MAX
    pub fn set_freq(&mut self, value: i32) {
        self.freq = value.max(0) as f32;
        self.env.reset_timer();
    }
    /// Expected input range: 0..=255
    pub fn set_vol(&mut self, value: i32) {
        self.vol = (value & 0xff) as f32 / 255.;
        self.env.reset_timer();
    }
    /// Expected input range: 0..=255
    /// DC range: 50% to 100%
    pub fn set_dc(&mut self, value: i32) {
        self.dc = (value & 0xff) as f32 / 255.;
        self.env.reset_timer();
    }
    /// Set envelope bitmask.
    pub fn set_env_mask(&mut self, value: i32) {
        self.env.set_mask(value);
        self.env.reset_timer();
    }
    /// Expected input range: 0..=i32::MAX
    pub fn set_env_length(&mut self, value: i32) {
        self.env.set_length(value.max(0) as f32 / 1000.);
        self.env.reset_timer();
    }
}

impl Default for RampChannel {
    fn default() -> Self {
        RampChannel {
            cycletimer: 0.,
            freq: 440.,
            vol: 0.,
            dc: 0.5,
            env: Envelope::default(),
        }
    }
}

impl AudioChannel for RampChannel {
    fn get_next_sample(&mut self) -> f32 {
        let delta_t = 1. / SAMPLE_RATE as f32;

        // Envelope
        self.env.update(delta_t);
        let env_value = self.env.get_value();

        // Cycle timer (pitch)
        let delta_cycle = self.freq * delta_t;
        self.cycletimer += delta_cycle;
        self.cycletimer %= 1.0;

        // Add Sweep (envelope affects pitch)
        if self.env.get_mask_freq() {
            match self.env.get_mask_falling() {
                true => self.cycletimer -= delta_cycle - delta_cycle * env_value,
                false => self.cycletimer += delta_cycle * env_value,
            }
        }

        // Volume
        let mut vol = self.vol;
        if self.env.get_mask_vol() {
            vol *= env_value;
        }
        if !self.env.get_gate_value() {
            vol = 0.;
        }
        vol = vol.clamp(0., 1.);

        // Duty Cycle
        let dc: f32 = match self.env.get_mask_pw() {
            true => env_value / 2. + 0.5,
            false => self.dc,
        };

        // Output function
        let rise_time = dc;
        let fall_time = 1.0 - dc;
        match self.cycletimer <= rise_time {
            true => (self.cycletimer / rise_time - 0.5) * 2. * vol, // Rise
            false => -((self.cycletimer - rise_time) / fall_time - 0.5) * 2. * vol, // Fall
        }
    }
}
