use std::time::Duration;

use num_traits::ToPrimitive;

const SAMPLECNT: usize = 128;

pub struct PerfMonitor {
    samples: Vec<f64>,
    idx: usize,
    target_rate: f64,
}

impl Default for PerfMonitor {
    fn default() -> Self {
        PerfMonitor {
            samples: vec![0.; SAMPLECNT],
            idx: 0,
            target_rate: 1.,
        }
    }
}

impl PerfMonitor {
    pub fn set_rate(&mut self, rate: f32) {
        if rate <= 0. {
            panic!("PerfMonitor: tickrate can't be zero or negative!!")
        }
        self.target_rate = rate.to_f64().unwrap();
    }

    pub fn add_sample(&mut self, dur: Duration) {
        let val = (1. / self.target_rate) / dur.as_secs_f64();
        self.idx += 1;
        self.idx %= self.samples.len();
        self.samples[self.idx] = val;
    }

    pub fn get_percent(&mut self) -> f32 {
        let mut avg = 0.;
        for sample in &self.samples {
            avg += sample;
        }
        avg /= self.samples.len().to_f64().unwrap();
        avg *= 100.;
        avg.to_f32().unwrap()
    }
}
