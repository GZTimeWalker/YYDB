use futures::Future;

use crate::{structs::AsyncKvIterator, utils::*};

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

impl AsyncKvIterator for SSTableIter {
    type NextFuture<'a> = impl Future<Output = Result<(Key, DataInner)>> + 'a;
    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            todo!();
        }
    }
}
