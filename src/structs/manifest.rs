use avl::AvlTreeMap;

use crate::utils::io_handler::IOHandlerFactory;

use super::sstable::sstable::{SSTable, SSTableKey};

#[derive(Debug)]
pub struct Manifest {
    tables: AvlTreeMap<SSTableKey, SSTable>,
    factory: IOHandlerFactory
}

impl Manifest {
    pub fn new(table_name: &str) -> Self {
        Self {
            tables: AvlTreeMap::new(),
            factory: IOHandlerFactory::new(table_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::sstable::metadata::SSTableMeta;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        std::fs::remove_dir_all("helper/sstable").ok();

        let mut manifest = Manifest {
            tables: AvlTreeMap::new(),
            factory: IOHandlerFactory::new("helper/sstable")
        };
        for _ in 0..10 {
            let rnd = rand::random::<u32>() % 6;
            let mut meta = SSTableMeta::default();
            meta.key = SSTableKey::new(rnd);
            manifest.tables.insert(
                meta.key,
                SSTable::new(meta, &manifest.factory).await
            );
        }
        for (k, v) in manifest.tables.iter() {
            println!("{:?}: \n{:#?}", k, v);
        }
    }
}
