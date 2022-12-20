use std::{fmt::Display, sync::Arc};

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

impl DataStore {
    pub fn unwrap(self) -> Data {
        match self {
            DataStore::Value(value) => value,
            _ => panic!("DataStore is not a Value"),
        }
    }

    pub fn is_deleted(&self) -> bool {
        match self {
            DataStore::Deleted => true,
            _ => false,
        }
    }
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

impl Display for DataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataStore::Value(value) => write!(f, "Value({})", value.len()),
            DataStore::Deleted => write!(f, "Deleted"),
            DataStore::NotFound => write!(f, "NotFound"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, vec};

    use rand::{RngCore, SeedableRng};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    #[tokio::test]
    async fn it_works_async() -> Result<()> {
        const TEST_SIZE: u64 = 1400;
        const DATA_SIZE: usize = 666;
        crate::utils::logger::init();

        let mut data_vec = Vec::<KvStore>::with_capacity(TEST_SIZE as usize);

        for i in 0..TEST_SIZE {
            // random with seed i
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let mut data = vec![0; DATA_SIZE];
            rng.fill_bytes(&mut data);

            data_vec.push((i, DataStore::Value(Arc::new(data))));
        }

        let mut bytes_read = 0;

        let bytes = {
            let mut writer = CompressionEncoder::new(Vec::new());
            for kvstore in data_vec.iter() {
                let row = bincode::encode_to_vec(kvstore, BIN_CODE_CONF).unwrap();
                bytes_read += row.len();
                writer.write_all(&row).await?;
            }
            writer.shutdown().await?;
            writer.into_inner()
        };

        debug!("Length for DataStore Test: {}/{}", bytes.len(), bytes_read);

        let buffer = {
            let mut writer = CompressionEncoder::new(Vec::new());
            writer.write_all(&bytes).await?;
            writer.shutdown().await?;
            writer.into_inner()
        };

        let mut decompressed = Vec::new();
        CompressionDecoder::new(buffer.as_slice())
            .read_to_end(&mut decompressed)
            .await?;

        assert!(bytes == decompressed);

        let mut count = 0;
        let mut bytes_read = 0;
        let mut reader = CompressionDecoder::new(decompressed.as_slice());
        let mut buffer = VecDeque::with_capacity(DATA_SIZE * 2);

        while let Ok((kvstore, offset)) = {
            let remain = buffer.capacity() - buffer.len();
            let mut vec_buffer = vec![0; remain];
            if let Ok(read) = reader.read(&mut vec_buffer).await {
                if read > 0 {
                    bytes_read += read;
                    buffer.extend(&vec_buffer);
                }
            }
            let slice = buffer.make_contiguous();
            bincode::decode_from_slice::<KvStore, BincodeConfig>(slice, BIN_CODE_CONF)
        } {
            let (key, value) = kvstore;

            if count == data_vec.len() {
                debug!("Success decoding {} items", count);
                break;
            }

            let expected = data_vec[count].clone();
            assert_eq!(key, expected.0);

            if expected.1 != value {
                match (value, expected.1) {
                    (DataStore::Value(value), DataStore::Value(data)) => {
                        error!(
                            "Error decoding data [{}@{}]: {}\n{}",
                            key,
                            count,
                            hex_view(&value)?,
                            hex_view(&data)?,
                        );
                    }
                    _ => {}
                }
                panic!();
            }
            count += 1;
            buffer.drain(..offset);
        }

        debug!("Bytes read: {}", bytes_read);

        Ok(())
    }
}
