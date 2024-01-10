pub mod opcodes {
    include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));
}

#[cfg(feature = "usbtower")]
pub mod usbtower;

mod errors;
pub use errors::{Error, Result};
