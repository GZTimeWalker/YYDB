use crate::utils::BloomFilter;

use super::sstable::SSTableKey;

#[derive(Debug, Clone, Encode, Decode)]
pub struct SSTableMeta {
    pub key: SSTableKey,
    pub entries_count: usize,
    pub bloom_filter: BloomFilter,
}

impl SSTableMeta {
    pub fn new(key: SSTableKey) -> Self {
        Self {
            key,
            entries_count: 0,
            bloom_filter: BloomFilter::new(key.level()),
        }
    }

    #[inline]
    pub fn set_entries_count(&mut self, count: usize) {
        self.entries_count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut meta = SSTableMeta::new(SSTableKey::new(0u64));
        meta.bloom_filter.insert(&[1, 2, 3]);

        let config = bincode::config::standard();

        let bytes = bincode::encode_to_vec(&meta, config).unwrap();

        assert!(bytes.len() > 0);

        debug!("Length for MetaData Test: {}", bytes.len());

        let decoded: SSTableMeta = bincode::decode_from_slice(&bytes, config).unwrap().0;

        assert_eq!(meta.key, decoded.key);
    }
}
