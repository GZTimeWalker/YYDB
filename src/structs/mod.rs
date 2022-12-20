pub mod kvstore;
pub mod lsm;
pub mod manifest;
pub mod mem;
pub mod table;
pub mod tracker;

pub use kvstore::*;
pub use mem::*;
pub use table::*;

pub const META_MAGIC_NUMBER: u32 = u32::from_be_bytes(*b"YYMT");
pub const CACHE_MAGIC_NUMBER: u32 = u32::from_be_bytes(*b"YYCA");
pub const SSTABLE_MAGIC_NUMBER: u32 = u32::from_be_bytes(*b"YYST");

pub const MEM_BLOCK_NUM: usize = 0x80;
pub const TABLE_COMPACT_THRESHOLD: usize = 4;
