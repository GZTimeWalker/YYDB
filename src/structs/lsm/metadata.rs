use growable_bloom_filter::GrowableBloom;

use super::sstable::SSTableKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSTableMeta {
    pub key: SSTableKey,
    pub checksum: u32,
    pub level: u32,
    pub row_size: u32,
    pub bloom_filter: GrowableBloom
}

impl Default for SSTableMeta {
    fn default() -> Self {
        Self {
            key: SSTableKey::new(0u64),
            checksum: 0,
            level: 0,
            row_size: 0,
            bloom_filter: GrowableBloom::new(0.01, 1000)
        }
    }
}
