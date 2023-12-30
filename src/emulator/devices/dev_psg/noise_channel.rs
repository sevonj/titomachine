//!
//! Noise generator
//!
//!

use super::{envelope::Envelope, AudioChannel, SAMPLE_RATE};

/// Noise generator
pub struct NoiseChannel {
    /// Position in current cycle. Range: 0.0 to 1.0
    cycletimer: f32,
    /// Frequency in Hz
    freq: f32,
    /// Volume range: 0.0 to 1.0
    vol: f32,
    /// Shift register state used for NES noise function
    sr: i32,
    /// Envelope.
    pub env: Envelope,
}

impl NoiseChannel {
    /// Expected input range: 0..=i32::MAX
    pub fn set_freq(&mut self, value: f32) {
        self.freq = value;
        //let actual_freq = ( ( 12 * log(f / 220.0) / log(2.0) ) + 57.01 )
        //self.freq = value.max(0) as f32;
        self.env.reset_timer();
    }
    /// Expected input range: 0..=255
    pub fn set_vol(&mut self, value: i32) {
        self.vol = (value & 0xff) as f32 / 255.;
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

impl Default for NoiseChannel {
    fn default() -> Self {
        NoiseChannel {
            cycletimer: 0.,
            freq: 440.,
            vol: 0.,
            sr: 1,
            env: Envelope::default(),
        }
    }
}

impl AudioChannel for NoiseChannel {
    fn get_next_sample(&mut self) -> f32 {
        let delta_t = 1. / SAMPLE_RATE as f32;

        // Envelope
        self.env.update(delta_t);
        let env_value = self.env.get_value();

        // Cycle timer (pitch)
        let delta_cycle = self.freq * delta_t;
        self.cycletimer += delta_cycle;
        if self.cycletimer >= 1.0 {
            // Update noise state
            self.sr = (self.sr >> 1) | ((self.sr & 1) ^ ((self.sr >> 1) & 1)) << 14;
        }
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

        // Output function
        match self.sr & 1 == 1 {
            true => vol,   // Hi
            false => -vol, // Lo
        }
    }
}
