//!
//! Envelope generator used by the audio channels
//!
//!

/// Envelope Controls Volume (Manual volume control should still apply over this).
const FLAG_VOL: i32 = 1;
/// Envelope Controls Pulse Width (Overrides manual control).
const FLAG_PW: i32 = 2;
/// Envelope Controls Pitch (sweep effect).
const FLAG_FREQ: i32 = 4;
/// Envelope Gate: Mute channel when timer runs out (independent from volume flag).
const FLAG_GATE: i32 = 8;
/// Falling instead of rising: Envelope goes from 100% to 0% instead of 0% to 100%.
const FLAG_FALLING: i32 = 16;
/// Loop. Envelope timer starts over if it runs out.
const FLAG_LOOP: i32 = 64;
/// If looping, every other cycle is reversed (rise-fall-rise-fall...).
const FLAG_LOOP_MIRROR: i32 = 128;

/// Envelope generator used by the audio channels
pub(crate) struct Envelope {
    /// Envelope options bitmask. Flags can be found at [module][self] constants.
    mask: i32,
    /// Envelope current position. Range: 0 to 1
    timer: f32,
    /// How long it takes for the timer to finish in seconds
    length: f32,
    /// Envelope output value
    value: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Envelope {
            mask: 0,
            timer: 0.,
            length: 1.,
            value: 0.,
        }
    }
}

impl Envelope {
    /// It is advisable to call this every time a sample is generated.  
    /// delta_t: Time since last update in seconds
    pub fn update(&mut self, delta_t: f32) {
        // Update the timer
        self.timer += delta_t / self.length;

        // If looping, we will modulo the timer. If not, we just let it keep going.
        if self.mask & FLAG_LOOP != 0 {
            match self.mask & FLAG_LOOP_MIRROR != 0 {
                true => self.timer %= 2.0, // Mirrored: Timer counts 2 cycles (2nd is reversed)
                false => self.timer %= 1.0, // No mirroring, just do the 1st cycle
            }
        }

        // Calculate the envelope output value
        // We need the extra accuracy of f64
        let mut value: f64 = self.timer as f64 / self.length as f64;
        value = value.clamp(0., 1.);

        if self.mask & FLAG_FALLING != 0 {
            value = 1. - value;
        }

        // We're currently at the second cycle. Invert the value. (Mirrored Loop)
        if self.mask & FLAG_LOOP != 0 && self.timer > 1. {
            value = 1. - value
        }

        self.value = ((value + 1.0) % 1.0) as f32;
    }

    // Getters

    #[inline]
    pub fn get_mask_vol(&self) -> bool {
        self.mask & FLAG_VOL != 0
    }
    #[inline]
    pub fn get_mask_pw(&self) -> bool {
        self.mask & FLAG_PW != 0
    }
    #[inline]
    pub fn get_mask_freq(&self) -> bool {
        self.mask & FLAG_FREQ != 0
    }
    #[inline]
    pub fn get_mask_falling(&self) -> bool {
        self.mask & FLAG_FALLING != 0
    }
    /// Return value can be used as is to control mute.  
    /// true => play, false => mute
    #[inline]
    pub fn get_gate_value(&self) -> bool {
        // Gate control is off
        if self.mask & FLAG_GATE == 0 {
            return true;
        }
        // If looping is enabled, the timer should never pass 1, but..
        // This check was necessary because of how we implemented mirrored loop. See update fn.
        if self.mask & FLAG_LOOP == 1 {
            return true;
        }
        // Gate is enabled, return value is determined by whether envelope timer has finished.
        self.timer <= 1.0
    }
    /// Envelope output value
    #[inline]
    pub fn get_value(&self) -> f32 {
        self.value
    }

    // Setters

    #[inline]
    pub fn set_mask(&mut self, mask: i32) {
        self.mask = mask;
    }
    #[inline]
    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }
    #[inline]
    pub fn reset_timer(&mut self) {
        self.timer = 0.;
    }
}
