use super::PMIO;
///
/// devices/crt.rs
///
/// Legacy output device =crt
///
///
use std::sync::mpsc::Sender;

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
