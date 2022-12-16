pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    KeyNotFound,
    EmptyFile,
    MissChecksum,
    InvalidMagicNumber,
    InvalidSSTableKey,
    UnknownRowSize,
    EncodeError(bincode::error::EncodeError),
    DecodeError(bincode::error::DecodeError),
    IOError(std::io::Error),
    Other(String),
}

from_error!(std::io::Error, IOError);
from_error!(bincode::error::EncodeError, EncodeError);
from_error!(bincode::error::DecodeError, DecodeError);
