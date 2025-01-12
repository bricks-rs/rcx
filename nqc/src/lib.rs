pub mod ast;
pub mod error;
pub mod lalrpop_version;
pub mod lexer;

pub use error::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    length: usize,
}
