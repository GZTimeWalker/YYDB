pub mod metadata;
pub mod sstable;

pub use sstable::*;

pub const MEM_BLOCK_NUM: usize = 64;
pub const MERGE_FACTOR: usize = 4;

fn bloom_size(level: u32) -> usize {
    MEM_BLOCK_NUM * (MERGE_FACTOR - 1).pow(level)
}
