use std::sync::atomic::{AtomicU64, Ordering};
use std::fs::File;
use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TableId(pub u64);

impl TableId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // 0 is reserved for invalid table id
        TableId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    id: TableId,
    name: String,
}

#[derive(Debug, PartialEq)]
pub enum TableError {
    FileCreateError,
    FileNotFound,
    FileAlreadyExists
}

pub type TableResult = Result<Table, TableError>;

impl Table {
    pub fn open(table_name: &str) -> TableResult {
        let table_name = format!("{}{}", table_name, TABLE_FILE_SUFFIX);

        info!("Opening table: {}", table_name);

        // open the table file, create if not exists
        File::create(&table_name).map_err(|_| TableError::FileCreateError)?;

        Ok(Table {
            id: TableId::new(),
            name: table_name,
         })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Closing table: {}", self.name);
        // TODO: cleanup and flush all data
    }
}
