use std::sync::Arc;
use super::*;

type Data = Arc<Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataStore {
    Value(Data),
    Deleted,
    NotFound,
}

impl Into<Option<Data>> for DataStore {
    fn into(self) -> Option<Data> {
        match self {
            DataStore::Value(value) => Some(value),
            _ => None,
        }
    }
}

impl Into<Result<Option<Data>>> for DataStore {
    fn into(self) -> Result<Option<Data>> {
        match self {
            DataStore::Value(value) => Ok(Some(value)),
            DataStore::Deleted => Ok(None),
            DataStore::NotFound => Err(DbError::KeyNotFound)
        }
    }
}
