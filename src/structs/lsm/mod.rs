pub mod metadata;
pub mod sstable;

pub use sstable::*;

pub const MEM_BLOCK_NUM: u64 = 64;
pub const MERGE_FACTOR: u64 = 4;

fn bloom_size(level: u32) -> u64 {
    MEM_BLOCK_NUM * (MERGE_FACTOR - 1).pow(level)
}
