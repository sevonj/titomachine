//!
//! Programmable Sound Generator
//!
//! PSG is an audio device with similar capabilities to an '80s home console.
//!
//! It has four channels:
//! - ch0: PulseChannel
//! - ch1: PulseChannel
//! - ch2: RampChannel
//! - ch3: NoiseChannel
//!
//! The channel structs implement the AudioChannel trait.
//!
//! They are wrapped in are wrapped in `Arc<Mutex<_>>>`.  
//! Example:
//! `let ch0 = Arc::new(Mutex::new(PulseChannel::default()));`
//!

use super::{Device, MMIO};
use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};

mod envelope;
mod noise_channel;
mod pulse_channel;
mod ramp_channel;
use self::noise_channel::NoiseChannel;
use self::pulse_channel::PulseChannel;
use self::ramp_channel::RampChannel;

const SAMPLE_RATE: u32 = 22050;

/// Device struct.
///
pub(crate) struct DevPSG {
    stream: Option<OutputStream>,
    sink0: Sink,
    sink1: Sink,
    sink2: Sink,
    sink3: Sink,
    ch0: Arc<Mutex<PulseChannel>>,
    ch1: Arc<Mutex<PulseChannel>>,
    ch2: Arc<Mutex<RampChannel>>,
    ch3: Arc<Mutex<NoiseChannel>>,
}

impl Default for DevPSG {
    fn default() -> Self {
        //
        let try_default = OutputStream::try_default();

        let sink0;
        let sink1;
        let sink2;
        let sink3;

        let stream;
        // Host has an audio device, create sinks.
        if try_default.is_ok() {
            let (_stream, _stream_handle) = try_default.unwrap();
            stream = Some(_stream);
            sink0 = Sink::try_new(&_stream_handle).unwrap();
            sink1 = Sink::try_new(&_stream_handle).unwrap();
            sink2 = Sink::try_new(&_stream_handle).unwrap();
            sink3 = Sink::try_new(&_stream_handle).unwrap();
        }
        // No audio devices available. Create sinks that do nothing.
        else {
            stream = None;
            (sink0, _) = Sink::new_idle();
            (sink1, _) = Sink::new_idle();
            (sink2, _) = Sink::new_idle();
            (sink3, _) = Sink::new_idle();
        }

        sink0.pause();
        sink1.pause();
        sink2.pause();
        sink3.pause();

        // Create channels
        let ch0 = Arc::new(Mutex::new(PulseChannel::default()));
        let ch1 = Arc::new(Mutex::new(PulseChannel::default()));
        let ch2 = Arc::new(Mutex::new(RampChannel::default()));
        let ch3 = Arc::new(Mutex::new(NoiseChannel::default()));

        // Create sources
        sink0.append(AudioSource::new(ch0.clone()));
        sink1.append(AudioSource::new(ch1.clone()));
        sink2.append(AudioSource::new(ch2.clone()));
        sink3.append(AudioSource::new(ch3.clone()));

        DevPSG {
            stream: stream,
            sink0,
            sink1,
            sink2,
            sink3,
            ch0,
            ch1,
            ch2,
            ch3,
        }
    }
}

impl Device for DevPSG {
    fn reset(&mut self) {
        self.sink0.clear();
        self.sink1.clear();
        self.sink2.clear();
        self.sink3.clear();
        self.ch0 = Arc::new(Mutex::new(PulseChannel::default()));
        self.ch1 = Arc::new(Mutex::new(PulseChannel::default()));
        self.ch2 = Arc::new(Mutex::new(RampChannel::default()));
        self.ch3 = Arc::new(Mutex::new(NoiseChannel::default()));
        self.sink0.append(AudioSource::new(self.ch0.clone()));
        self.sink1.append(AudioSource::new(self.ch1.clone()));
        self.sink2.append(AudioSource::new(self.ch2.clone()));
        self.sink3.append(AudioSource::new(self.ch3.clone()));
    }

    fn on(&mut self) {
        self.sink0.play();
        self.sink1.play();
        self.sink2.play();
        self.sink3.play();
    }

    fn off(&mut self) {
        self.sink0.pause();
        self.sink1.pause();
        self.sink2.pause();
        self.sink3.pause();
    }
}

impl MMIO for DevPSG {
    fn read(&mut self, _addr: usize) -> Result<i32, ()> {
        Err(())
    }

    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()> {
        //
        match addr {
            // ch0
            0x00 => self.ch0.lock().unwrap().set_freq(value),
            0x01 => self.ch0.lock().unwrap().set_vol(value),
            0x02 => self.ch0.lock().unwrap().set_pw(value),
            0x03 => self.ch0.lock().unwrap().set_env_mask(value),
            0x04 => self.ch0.lock().unwrap().set_env_length(value),
            // ch1
            0x10 => self.ch1.lock().unwrap().set_freq(value),
            0x11 => self.ch1.lock().unwrap().set_vol(value),
            0x12 => self.ch1.lock().unwrap().set_pw(value),
            0x13 => self.ch1.lock().unwrap().set_env_mask(value),
            0x14 => self.ch1.lock().unwrap().set_env_length(value),
            // ch2
            0x20 => self.ch2.lock().unwrap().set_freq(value),
            0x21 => self.ch2.lock().unwrap().set_vol(value),
            0x22 => self.ch2.lock().unwrap().set_dc(value),
            0x23 => self.ch2.lock().unwrap().set_env_mask(value),
            0x24 => self.ch2.lock().unwrap().set_env_length(value),
            // ch3
            0x30 => self.ch3.lock().unwrap().set_freq(value),
            0x31 => self.ch3.lock().unwrap().set_vol(value),
            //0x32 =>
            0x33 => self.ch3.lock().unwrap().set_env_mask(value),
            0x34 => self.ch3.lock().unwrap().set_env_length(value),

            _ => return Err(()),
        }
        Ok(())
    }
}

/// Any audio generator needs to implement this trait.
pub trait AudioChannel {
    /// This is called SAMPLE_RATE times a second.
    fn get_next_sample(&mut self) -> f32;
}

/// Channel wrapper. This is passed to the audio sink.  
/// ```
/// // Example usage:
/// let channel = Arc::new(Mutex::new(PulseChannel::default())); // Create the channel
/// let source = AudioSource::new(channel.clone());              // Create the source
/// sink.append(source)                    // The source now belongs to the sink...
/// channel.lock().unwrap().freq = 261.63; // ...but we can still control it via the channel
/// ```
pub(crate) struct AudioSource<T: AudioChannel> {
    channel: Arc<Mutex<T>>,
}

impl<T: AudioChannel> AudioSource<T> {
    pub(crate) fn new(channel: Arc<Mutex<T>>) -> Self {
        Self { channel }
    }
}

impl<T: AudioChannel> Iterator for AudioSource<T> {
    type Item = f32;
    // Iterator.next returns the next sample.
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.channel.lock().unwrap().get_next_sample())
    }
}

impl<T: AudioChannel> Source for AudioSource<T> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

/*
// For waveform debugging
fn oscilloscope(sample: f32) {
    let mut display_sample = sample;

    display_sample += 1.;
    display_sample /= 2.;

    if display_sample < 0. {
        display_sample = 0.;
    }
    if display_sample > 1. {
        display_sample = 1.;
    }

    let w = 100 - 1;
    let l = (w as f32 * display_sample) as usize;
    let r = w - l;

    println!("{}#{}    {}", ".".repeat(l), ".".repeat(r), sample);
}
*/
