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
}
