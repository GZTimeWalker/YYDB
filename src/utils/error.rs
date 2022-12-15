pub type Result<T> = std::result::Result<T, DbError>;


#[derive(Debug)]
pub enum DbError {
    IOError(std::io::Error),
    Other(String),
}

from_error!(std::io::Error, IOError);
