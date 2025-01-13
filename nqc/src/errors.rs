pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Integer conversion error: {0}")]
    IntConversion(#[from] std::num::TryFromIntError),

    #[error("Path is not a chardev")]
    NotChardev,

    #[error("Timeout was reached")]
    Timeout,

    #[error("Checksum mismatch")]
    Checksum,

    #[error("Invalid data: {0}")]
    InvalidData(&'static str),

    #[error("RCX error: {0}")]
    RcxError(&'static str),

    #[error("Parse error: {0}")]
    Parse(&'static str),

    #[error("Nom error: {0}")]
    Nom(String),

    #[error("Reached end of input")]
    InsufficientData,

    #[error("Invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),
}

impl<T: std::fmt::Debug> From<nom::Err<T>> for Error {
    fn from(value: nom::Err<T>) -> Self {
        Error::Nom(format!("{value:?}"))
    }
}
