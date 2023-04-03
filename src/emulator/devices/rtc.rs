///
/// devices/rtc.rs
///
/// Real-time clock. Returns local 32-bit unix-time.
///
use super::PMIO;
use chrono::Local;

pub(crate) struct DevRTC {}

impl Default for DevRTC {
    fn default() -> Self {
        DevRTC {}
    }
}

impl PMIO for DevRTC {
    fn read_port(&mut self, port: u8) -> Result<i32, ()> {
        if port != 0 {
            return Err(());
        }
        let time = Local::now().timestamp() as i32 + Local::now().offset().local_minus_utc();
        Ok(time)
    }
    fn write_port(&mut self, _port: u8, _value: i32) -> Result<(), ()> {
        Err(()) // You can't write into the clock!
    }
}
