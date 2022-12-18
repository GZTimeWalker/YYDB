pub mod iter;
pub mod metadata;
pub mod sstable;
pub mod sstable_iter;

pub use iter::*;
pub use metadata::*;
pub use sstable::*;
pub use sstable_iter::*;

pub type SSTableLevel = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SSTableStatus {
    Available,
    Compacting,
    Compacted,
}
