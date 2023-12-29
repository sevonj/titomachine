//!
//! Legacy output device =crt
//! Communication happens via an mpsc channel, which could be refactored away.
//!
//!
use super::{Device, PMIO};
use std::sync::mpsc::Sender;

/// Legacy output device =crt
pub(crate) struct DevCRT {
    output: Option<Sender<i32>>,
}

impl Default for DevCRT {
    fn default() -> Self {
        DevCRT { output: None }
    }
}

impl DevCRT {
    pub fn connect(&mut self, output: Sender<i32>) {
        self.output = Some(output);
    }
}

impl Device for DevCRT {
    fn reset(&mut self) {}
    fn on(&mut self) {}
    fn off(&mut self) {}
}

/// Port 0: crt output
impl PMIO for DevCRT {
    fn read_port(&mut self, _port: u8) -> Result<i32, ()> {
        Err(()) // You can't read from the crt!
    }
    fn write_port(&mut self, port: u8, value: i32) -> Result<(), ()> {
        if port != 0 {
            return Err(());
        }
        if let Some(output) = &self.output {
            if let Err(_) = output.send(value) {
                return Err(());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dev_crt() -> Result<(), ()> {
        let mut crt = DevCRT::default();
        let (tx, rx) = std::sync::mpsc::channel();
        crt.connect(tx);

        // Write to correct port.
        crt.write_port(0, 55)?;
        assert!(rx.try_recv().unwrap() == 55);

        // Write to incorrect port.
        assert!(crt.write_port(1, 55).is_err());
        assert!(crt.write_port(2, 55).is_err());
        assert!(crt.write_port(3, 55).is_err());
        assert!(rx.try_recv().is_err());

        // Try reading from it
        assert!(crt.read_port(0).is_err());
        assert!(crt.read_port(1).is_err());
        assert!(crt.read_port(2).is_err());
        assert!(rx.try_recv().is_err());

        Ok(())
    }
}
