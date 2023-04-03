/*
 * Performance monitor keeps track of how the well
 * the emulator can keep up with target cpu speed.
 *
 */

use std::time::{Duration, Instant};

pub struct PerfMonitor {
    speed_percent: f32,
    counter: i32,
    update_rate: f32, // How often to update speed_percent
    update_timer: Duration,
    target_rate: f32,
    t_last_tick: Instant,
}

impl Default for PerfMonitor {
    fn default() -> Self {
        PerfMonitor {
            speed_percent: 0.,
            counter: 0,
            update_rate: 1.,
            update_timer: Duration::ZERO,
            target_rate: 1.,
            t_last_tick: Instant::now(),
        }
    }
}

impl PerfMonitor {
    pub fn set_rate(&mut self, rate: f32) {
        if rate <= 0. {
            panic!("PerfMonitor: tickrate can't be zero or negative!!")
        }
        self.target_rate = rate;
        self.update_rate = rate.min(4.);
    }

    pub fn tick(&mut self) {
        let t_now = Instant::now();
        self.update_timer += t_now - self.t_last_tick;
        self.t_last_tick = t_now;
        self.counter += 1;

        let update_wait = Duration::from_secs_f32(1. / self.update_rate);
        if self.update_timer >= update_wait {
            let target_count = self.target_rate / self.update_rate;
            self.speed_percent = (self.counter as f32 / target_count) * 100.;
            self.counter = 0;
            self.update_timer -= update_wait;
        }
    }

    pub fn get_percent(&mut self) -> f32 {
        self.speed_percent
    }
}
