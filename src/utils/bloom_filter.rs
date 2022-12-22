//! Ref: <https://docs.rs/bincode/2.0.0-rc.2/bincode/index.html>

use std::fmt::Formatter;

use growable_bloom_filter::GrowableBloom;

use crate::structs::MEM_BLOCK_NUM;

pub const MAX_EXPECT_NUM: usize = 0xff000;
pub const MERGE_FACTOR: usize = 4;

#[derive(Clone, Encode, Decode)]
pub struct BloomFilter {
    #[bincode(with_serde)]
    filter: GrowableBloom,
}

impl BloomFilter {
    pub fn new(level: u32) -> Self {
        Self {
            filter: GrowableBloom::new(0.01, bloom_size(level)),
        }
    }

    pub fn new_global() -> Self {
        Self {
            filter: GrowableBloom::new(0.05, MAX_EXPECT_NUM),
        }
    }
}

fn bloom_size(level: u32) -> usize {
    let num = MEM_BLOCK_NUM * MERGE_FACTOR.pow(level) * 2;
    if num > MAX_EXPECT_NUM {
        MAX_EXPECT_NUM
    } else {
        num
    }
}

impl std::ops::Deref for BloomFilter {
    type Target = GrowableBloom;

    fn deref(&self) -> &Self::Target {
        &self.filter
    }
}

impl std::ops::DerefMut for BloomFilter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.filter
    }
}

impl std::fmt::Debug for BloomFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BloomFilter")
            .field("filter", &self.filter)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut bloom = BloomFilter::new(0);
        bloom.insert(&[1, 2, 3]);

        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(&bloom, config).unwrap();

        assert!(bytes.len() > 0);
        debug!("Length for L0 BloomFilter Test: {}", bytes.len());

        let decoded: BloomFilter = bincode::decode_from_slice(&bytes, config).unwrap().0;
        assert_eq!(bloom.filter, decoded.filter);

        let mut bloom = BloomFilter::new_global();
        bloom.insert(&[1, 2, 3]);

        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(&bloom, config).unwrap();

        assert!(bytes.len() > 0);
        debug!("Length for Global BloomFilter Test: {}", bytes.len());

        let decoded: BloomFilter = bincode::decode_from_slice(&bytes, config).unwrap().0;
        assert_eq!(bloom.filter, decoded.filter);
    }
}
