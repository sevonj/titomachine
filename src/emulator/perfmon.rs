/*
 * Performance monitor keeps track of how the well
 * the emulator can keep up with target cpu speed.
 *
 */

use std::time::{Duration, Instant};

pub struct PerfMonitor {
    last_reset: Instant,
    last_duration: Duration,
}

impl Default for PerfMonitor {
    fn default() -> Self {
        PerfMonitor {
            last_reset: Instant::now(),
            last_duration: Duration::ZERO,
        }
    }
}

impl PerfMonitor {
    pub fn update(&mut self){
        let now = Instant::now();
        self.last_duration = now - self.last_reset;
        self.last_reset = now;
    }

    pub fn get_last_duration(&mut self) -> f32 {
        self.last_duration.as_secs_f32()
    }
}
