use futures::Future;

use crate::{structs::AsyncIterator, utils::*};
use std::{collections::VecDeque, sync::Arc};

use super::*;

#[derive(Debug)]
pub struct LsmTreeIterator {
    tables: DequeIterator<Arc<SSTable>>,
    cur: Option<Arc<SSTable>>,
}

impl LsmTreeIterator {
    pub fn new(tables: VecDeque<Arc<SSTable>>) -> Self {
        Self {
            tables: DequeIterator::new(tables),
            cur: None,
        }
    }
}

impl AsyncIterator<KvStore> for LsmTreeIterator {
    type NextFuture<'a> = impl Future<Output = Result<Option<KvStore>>> + 'a;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async {
            loop {
                if let Some(cur) = &mut self.cur {
                    let mut iter = cur.iter().await;

                    if let Some(kvstore) = iter.next().await? {
                        return Ok(Some(kvstore));
                    }
                }

                match self.tables.next() {
                    Some(table) => {
                        table.init_iter().await?;
                        self.cur = Some(table);
                    }
                    None => break,
                }
            }

            Ok(None)
        }
    }
}
