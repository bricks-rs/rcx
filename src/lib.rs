pub mod opcodes {
    use crate::Result;
    use std::io::{self, Write};

    trait WriteParam {
        fn write_param(&self, buf: impl Write) -> io::Result<()>;
    }

    macro_rules! writeparamimpl {
        ($ty:ty) => {
            impl WriteParam for $ty {
                fn write_param(&self, mut buf: impl Write) -> io::Result<()> {
                    buf.write_all(&self.to_be_bytes())
                }
            }
        };
    }

    writeparamimpl!(u8);
    writeparamimpl!(i8);
    writeparamimpl!(u16);
    writeparamimpl!(i16);

    impl<const N: usize> WriteParam for [u8; N] {
        fn write_param(&self, mut buf: impl Write) -> io::Result<()> {
            buf.write_all(self)
        }
    }

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
