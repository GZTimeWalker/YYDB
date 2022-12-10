use crate::structs::table::TableId;

#[inline]
pub fn open_table(table_name: &str) -> u64 {
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

pub unsafe fn insert_row(_table_id: u64, row: *const u8, len: u32) {
    let row = std::slice::from_raw_parts(row, len as usize);
    crate::utils::print_hex_view(row);
}

pub unsafe fn update_row(_table_id: u64, row: *const u8, new_row: *const u8, len: u32) {
    let row = std::slice::from_raw_parts(row, len as usize);
    let new_row = std::slice::from_raw_parts(new_row, len as usize);
    crate::utils::print_hex_view(row);
    crate::utils::print_hex_view(new_row);
}
