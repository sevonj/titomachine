///
/// devices/kbd.rs
///
/// Legacy input device =kbd
/// This will freeze the emulator thread until a reply is received.
///
use super::PMIO;
use std::sync::mpsc::{Receiver, Sender};

pub(crate) struct DevKBD {
    pub input: Option<Receiver<i32>>,
    pub inputreq: Option<Sender<()>>,
}

impl Default for DevKBD {
    fn default() -> Self {
        DevKBD {
            input: None,
            inputreq: None,
        }
    }
}

impl DevKBD {
    pub fn connect(&mut self, input: Receiver<i32>, inputreq: Sender<()>) {
        self.input = Some(input);
        self.inputreq = Some(inputreq);
    }
}

impl PMIO for DevKBD {
    fn read_port(&mut self, port: u8) -> Result<i32, ()> {
        if port != 0 {
            return Err(());
        }
        if let Some(inputreq) = &self.inputreq {
            if let Err(_) = inputreq.send(()) {
                return Err(());
            }
            if let Some(input) = &self.input {
                if let Ok(val) = input.recv() {
                    return Ok(val);
                }
            }
        }
        Err(())
    }
    fn write_port(&mut self, _port: u8, _value: i32) -> Result<(), ()> {
        Err(()) // You can't write into the clock!
    }
}
