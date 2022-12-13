use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct MemTable {
    map: BTreeMap<u64, Arc<Vec<u8>>>,
    data: Option<IntoIter<Arc<Vec<u8>>>>,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            data: None,
        }
    }

    pub fn get(&self, key: u64) -> Option<Arc<Vec<u8>>> {
        self.map.get(&key).cloned()
    }

    pub fn set(&mut self, key: u64, value: Vec<u8>) {
        self.map.insert(key, Arc::new(value));
    }

    pub fn delete(&mut self, key: u64) {
        self.map.remove(&key);
    }

    pub unsafe fn next(&mut self, buf: *mut u8, len: usize) -> i32 {
        if let Some(data) = &mut self.data {
            if let Some(value) = data.next() {
                let value = value.as_slice();
                std::ptr::copy_nonoverlapping(value.as_ptr(), buf, len);
                crate::utils::print_hex_view(value);
                return len as i32;
            }
        }
        0
    }

    pub fn init_iter(&mut self) {
        self.data = Some(self.map.values().cloned().collect::<Vec<_>>().into_iter());
    }

    pub fn end_iter(&mut self) {
        self.data = None;
    }
}
