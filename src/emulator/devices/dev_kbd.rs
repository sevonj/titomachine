//!
//! Legacy input device =kbd
//!
//! This will freeze the emulator thread until a reply is received.
//!
use super::{Device, PMIO};
use std::sync::mpsc::{Receiver, Sender};

/// Legacy input device =kbd
pub(crate) struct DevKBD {
    pub input_requester: Option<Sender<()>>,
    pub input_receiver: Option<Receiver<i32>>,
}

impl Default for DevKBD {
    fn default() -> Self {
        DevKBD {
            input_requester: None,
            input_receiver: None,
        }
    }
}

impl DevKBD {
    pub fn connect(&mut self, input: Receiver<i32>, inputreq: Sender<()>) {
        self.input_receiver = Some(input);
        self.input_requester = Some(inputreq);
    }
}

impl Device for DevKBD {
    fn reset(&mut self) {}
    fn on(&mut self) {}
    fn off(&mut self) {}
}

impl PMIO for DevKBD {
    fn read_port(&mut self, port: u8) -> Result<i32, ()> {
        if port != 0 {
            return Err(());
        }
        if let Some(inputreq) = &self.input_requester {
            if let Err(_) = inputreq.send(()) {
                return Err(());
            }
            if let Some(input) = &self.input_receiver {
                if let Ok(val) = input.recv() {
                    return Ok(val);
                }
            }
        }
        Err(())
    }
    fn write_port(&mut self, _port: u8, _value: i32) -> Result<(), ()> {
        Err(()) // You can't write into the keyboard!
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;

    #[test]
    fn test_dev_kbd() -> Result<(), ()> {
        let mut kbd = DevKBD::default();
        let (input_tx, input_rx) = std::sync::mpsc::channel();
        let (requester_tx, requester_rx) = std::sync::mpsc::channel();
        kbd.connect(input_rx, requester_tx);

        // Test wrong usage
        assert!(kbd.read_port(1).is_err());
        assert!(kbd.read_port(2).is_err());
        assert!(kbd.read_port(3).is_err());
        assert!(kbd.write_port(0, 0).is_err());
        assert!(kbd.write_port(1, 55).is_err());
        assert!(kbd.write_port(2, -33).is_err());
        assert!(requester_rx.try_recv().is_err());

        // Test read correctly
        // Because KBD device locks the program, we have to put it into another thread.
        let cpu_thread = thread::spawn(move || {
            return kbd.read_port(0);
        });
        input_tx.send(55).unwrap();
        assert!(cpu_thread.join().unwrap() == Ok(55));
        assert!(requester_rx.try_recv().is_ok());

        Ok(())
    }
}
