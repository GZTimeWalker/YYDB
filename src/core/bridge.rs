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
pub fn close_table(_id: u64) {
    // super::runtime::close_table(&TableId(id));
}

pub unsafe fn insert_row(table_id: u64, key: u64, data: *const u8, len: u32) {
    info!("Inserting row({}) @{:3<} for table {}", len, key, table_id);

    let data = std::slice::from_raw_parts(data, len as usize);

    crate::utils::print_hex_view(data);

    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        table.set(key, data.to_vec().clone());
    }
}

pub unsafe fn update_row(table_id: u64, key: u64, _data: *const u8, new_data: *const u8, len: u32) {
    info!("Updating row({}) @{:3<} for table {}", len, key, table_id);

    let new_data = std::slice::from_raw_parts(new_data, len as usize);

    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        table.set(key, new_data.to_vec());
    }
}

pub unsafe fn delete_row(table_id: u64, key: u64) {
    info!("Deleting row @{:3<} for table {}", key, table_id);

    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        table.delete(key);
    }
}

pub unsafe fn put_hex(data: *const u8, len: u32) {
    let data = std::slice::from_raw_parts(data, len as usize);
    crate::utils::print_hex_view(data);
}

pub fn rnd_init(table_id: u64) {
    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        debug!("Init round for table {}", table_id);
        table.read_init();
    }
}

pub fn rnd_end(table_id: u64) {
    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        debug!("End round for table {}", table_id);
        table.read_end();
    }
}

pub unsafe fn rnd_next(table_id: u64, buf: *mut u8, len: u32) -> i32 {
    if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
        debug!("Read next row for table {}", table_id);
        table.read_next(buf, len)
    } else {
        0
    }
}
