//!
//! Real-time clock in a port. Returns local 32-bit unix-time.
//!
use super::{Device, PMIO};
use chrono::Local;

/// Real-time clock in a port. Returns local 32-bit unix-time.
pub(crate) struct DevRTC {}

impl Default for DevRTC {
    fn default() -> Self {
        DevRTC {}
    }
}

impl Device for DevRTC {
    fn reset(&mut self) {}
    fn on(&mut self) {}
    fn off(&mut self) {}
    fn set_pause(&mut self, _paused: bool) {}
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dev_rtc() -> Result<(), ()> {
        let mut rtc = DevRTC::default();

        // Test wrong usage
        assert!(rtc.read_port(1).is_err());
        assert!(rtc.read_port(2).is_err());
        assert!(rtc.read_port(3).is_err());
        assert!(rtc.write_port(0, 0).is_err());
        assert!(rtc.write_port(1, 55).is_err());
        assert!(rtc.write_port(2, -33).is_err());

        // Test read correctly
        let time = Local::now().timestamp() as i32 + Local::now().offset().local_minus_utc();
        let rtc_time = rtc.read_port(0).unwrap();
        assert_eq!(time, rtc_time);

        Ok(())
    }
}
