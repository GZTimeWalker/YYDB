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

pub unsafe fn insert_row(_table_id: u64, _key: u64, data: *const u8, len: u32) {
    let data = std::slice::from_raw_parts(data, len as usize);
    crate::utils::print_hex_view(data);
}

pub unsafe fn update_row(_table_id: u64, _key: u64, data: *const u8, new_data: *const u8, len: u32) {
    let data = std::slice::from_raw_parts(data, len as usize);
    let new_data = std::slice::from_raw_parts(new_data, len as usize);
    crate::utils::print_hex_view(data);
    crate::utils::print_hex_view(new_data);
}
