use crate::structs::table::TableId;

#[inline]
pub fn open_table(table_name: &str) -> u64 {
    info!("open table: {}", table_name);

    if let Some(id) = super::runtime::open_table(table_name) {
        id.0
    } else {
        0
    }
}

#[inline]
pub fn close_table(id: u64) {
    super::runtime::close_table(&TableId(id));
}
