use growable_bloom_filter::GrowableBloom;

use super::sstable::SSTableKey;

#[derive(Debug, Clone, Encode, Decode)]
pub struct SSTableMeta {
    pub key: SSTableKey,
    pub checksum: u32,
    pub level: u32,
    pub row_size: u32,

    #[bincode(with_serde)]
    pub bloom_filter: GrowableBloom,
}

impl Default for SSTableMeta {
    fn default() -> Self {
        Self {
            key: SSTableKey::new(0u64),
            checksum: 0,
            level: 0,
            row_size: 0,
            // bloom filter's parameters may be different in different level
            bloom_filter: GrowableBloom::new(0.05, 64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut meta = SSTableMeta::default();
        meta.bloom_filter.insert(&[1, 2, 3]);

        let config = bincode::config::standard();

        let bytes = bincode::encode_to_vec(&meta, config).unwrap();

        assert!(bytes.len() > 0);

        println!("Length for MetaData Test: {}", bytes.len());

        let decoded: SSTableMeta = bincode::decode_from_slice(&bytes, config).unwrap().0;

        assert_eq!(meta.key, decoded.key);
    }
}
