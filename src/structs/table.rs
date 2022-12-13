use super::mem::MemTable;
use super::*;
use std::fs::File;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TableId(pub u64);

impl TableId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // 0 is reserved for invalid table id
        TableId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TableId {
    fn default() -> Self {
        TableId::new()
    }
}

#[derive(Debug)]
pub struct Table {
    id: TableId,
    name: String,
    mem: Mutex<MemTable>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TableError {
    FileCreateError,
    FileNotFound,
    FileAlreadyExists,
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
            mem: Mutex::new(MemTable::new()),
        })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, key: u64) -> Option<Arc<Vec<u8>>> {
        self.mem.lock().unwrap().get(key)
    }

    pub fn set(&self, key: u64, value: Vec<u8>) {
        self.mem.lock().unwrap().set(key, value);
    }

    pub fn delete(&self, key: u64) {
        self.mem.lock().unwrap().delete(key);
    }

    pub fn read_init(&self) {
        self.mem.lock().unwrap().init_iter();
    }

    pub fn read_end(&self) {
        self.mem.lock().unwrap().end_iter();
    }

    pub unsafe fn read_next(&self, buf: *mut u8, len: u32) -> i32 {
        debug!("Reading next value from table: {}", self.name);
        self.mem.lock().unwrap().next(buf, len as usize)
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Closing table: {}", self.name);
        // TODO: cleanup and flush all data
    }
}
