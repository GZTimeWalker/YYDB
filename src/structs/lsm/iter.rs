use crate::utils::{IOHandler, KVStore};

#[derive(Debug)]
pub struct SSTableIter {
    io: IOHandler,
    buf: Vec<u8>,
    row_size: u32,
}

impl SSTableIter {
    pub fn new(io: IOHandler, row_size: u32) -> Self {
        Self {
            io,
            buf: Vec::new(),
            row_size,
        }
    }
}

impl Iterator for SSTableIter {
    type Item = KVStore;

    fn next(&mut self) -> Option<Self::Item> {
        // todo!()
        None
    }
}
