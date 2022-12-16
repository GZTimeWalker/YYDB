use std::sync::Arc;

use super::*;

type Data = Arc<Vec<u8>>;

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
        let config = bincode::config::standard();

        let bytes = bincode::encode_to_vec(&data, config).unwrap();

        println!("Length for DataStore Test: {}", bytes.len());

        let decoded: Vec<DataStore> = bincode::decode_from_slice(&bytes, config).unwrap().0;

        assert!(data == decoded);
    }
}
