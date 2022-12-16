pub mod kvstore;
pub mod lsm;
pub mod manifest;
pub mod mem;
pub mod table;

pub const META_MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"YYMT");
pub const CACHE_MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"YYCA");
pub const SSTABLE_MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"YYST");

pub const MEM_BLOCK_NUM: usize = 0x40;
