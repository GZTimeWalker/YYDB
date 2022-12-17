pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    KeyNotFound,
    EmptyFile,
    InvalidData,
    MissChecksum,
    InvalidMagicNumber,
    InvalidSSTableKey,
    UnknownRowSize,
    IOError(std::io::Error),
    FmtError(std::fmt::Error),
    EncodeError(bincode::error::EncodeError),
    DecodeError(bincode::error::DecodeError),
    Other(String),
}

from_error!(std::io::Error, IOError);
from_error!(std::fmt::Error, FmtError);
from_error!(bincode::error::EncodeError, EncodeError);
from_error!(bincode::error::DecodeError, DecodeError);
