#[cfg(feature = "usbtower")]
pub mod usb;

use crate::{opcodes::Opcode, Result};

pub trait IrTower {
    fn send(&mut self, msg: &dyn Opcode) -> Result<()>;
    fn recv(&mut self) -> Result<Vec<u8>>;

    fn send_recv(&mut self, msg: &dyn Opcode) -> Result<Vec<u8>> {
        self.send(msg)?;
        self.recv()
    }
}
