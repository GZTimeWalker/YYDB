use std::collections::BTreeMap;
use std::sync::Mutex;
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

#[derive(Debug)]
pub struct Table {
    id: TableId,
    name: String,
    mem: Mutex<BTreeMap<u64, Vec<u8>>>
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
            mem: Mutex::new(BTreeMap::new())
         })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, key: u64) -> Option<Vec<u8>> {
        let mem = self.mem.lock().unwrap();
        mem.get(&key).map(|v| v.clone())
    }

    pub fn set(&self, key: u64, value: Vec<u8>) {
        let mut mem = self.mem.lock().unwrap();
        mem.insert(key, value);
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Closing table: {}", self.name);
        // TODO: cleanup and flush all data
    }
}
