use std::sync::atomic::{AtomicU64, Ordering};
use std::fs::File;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(pub u64);

impl TableId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // 0 is reserved for invalid table id
        TableId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Table {
    id: TableId,
    name: String,
    file: File,
}

#[derive(Debug, PartialEq)]
pub enum TableError {
    FileCreateError,
    FileNotFound,
    FileAlreadyExists,
    FileTruncError
}

const TABLE_FILE_SUFFIX: &str = ".yyt";

pub type TableResult = Result<Table, TableError>;
pub type OpResult = Result<(), TableError>;

impl Table {
    pub fn open(table_name: &str) -> TableResult {
        let table_name = format!("{}{}", table_name, TABLE_FILE_SUFFIX);

        let file = File::create(&table_name)
            .map_err(|_| TableError::FileCreateError)?;
        Ok(Table {
            id: TableId::new(),
            name: table_name,
            file
         })
    }

    pub fn close(&self) {
        info!("Closing table: {}", self.name);
        self.file.sync_all().unwrap();
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
