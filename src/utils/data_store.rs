use std::sync::Arc;

use super::*;

pub type Key = u64;
pub type KvStore = (Key, DataStore);
pub type DataInner = Vec<u8>;
pub type Data = Arc<DataInner>;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum DataStore {
    Value(Data),
    Deleted,
    NotFound,
}

impl From<DataStore> for Option<Data> {
    fn from(data: DataStore) -> Self {
        match data {
            DataStore::Value(value) => Some(value),
            _ => None,
        }
    }
}

impl From<DataStore> for Result<Option<Data>> {
    fn from(data: DataStore) -> Self {
        match data {
            DataStore::Value(value) => Ok(Some(value)),
            DataStore::Deleted => Ok(None),
            DataStore::NotFound => Err(DbError::KeyNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = vec![
            DataStore::Value(Arc::new(vec![122; 64])),
            DataStore::Deleted,
            DataStore::NotFound,
        ];

        let bytes = bincode::encode_to_vec(&data, BIN_CODE_CONF).unwrap();

        debug!("Length for DataStore Test: {}", bytes.len());

        let decoded: Vec<DataStore> = bincode::decode_from_slice(&bytes, BIN_CODE_CONF).unwrap().0;

        assert!(data == decoded);
    }
}
