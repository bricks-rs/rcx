pub mod opcodes {
    use crate::Result;
    pub trait Opcode {
        fn request_opcode(&self) -> u8;
        fn response_opcode(&self) -> Option<u8>;
        fn serialise(&self, buf: &mut [u8]) -> Result<usize>;
    }
    include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));
}

pub mod tower;

mod errors;
pub use errors::{Error, Result};

use tower::IrTower;

pub struct Rcx {
    tower: Box<dyn IrTower>,
}

impl Rcx {
    pub fn new(tower: impl IrTower + 'static) -> Self {
        Self {
            tower: Box::new(tower),
        }
    }

    pub fn alive(&mut self) -> Result<()> {
        self.tower.send_recv(&opcodes::Alive {})?;
        Ok(())
    }
}
