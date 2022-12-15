use super::mem::{MemTable, DataBlock};
use super::*;
use std::fs::File;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TableId(pub u64);

impl TableId {
    pub fn new(table_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        table_name.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug)]
pub struct Table {
    id: TableId,
    name: String,
    mem: MemTable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TableError {
    FileCreateError,
    FileNotFound,
    FileAlreadyExists,
}

pub type TableResult = Result<Table, TableError>;

impl Table {
    pub fn open(table_name: String) -> TableResult {
        let table_name = format!("{}{}", table_name, TABLE_FILE_SUFFIX);

        info!("Opening table: {}", table_name);

        // open the table file, create if not exists
        File::create(&table_name).map_err(|_| TableError::FileCreateError)?;

        Ok(Table {
            id: TableId::new(&table_name),
            name: table_name,
            mem: MemTable::new(),
        })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn get(&self, key: u64) -> Option<DataBlock> {
        self.mem.get(key).await
    }

    pub async fn set(&self, key: u64, value: Vec<u8>) {
        self.mem.set(key, value).await
    }

    pub async fn delete(&self, key: u64) {
        self.mem.delete(key).await
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Closing table: {}", self.name);
    }
}
