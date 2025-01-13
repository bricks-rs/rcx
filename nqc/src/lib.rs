pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use error::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    length: usize,
}
