use crate::structs::table::TableId;

#[inline]
pub fn open_table(table_name: &str) -> u64 {
    let table_name = table_name.to_string();

    run_async!{
        if let Some(id) = super::runtime::open_table(table_name).await {
            id.0
        } else {
            0
        }
    }
}

#[inline]
pub fn close_table(id: u64) {
    run_async!{
        super::runtime::close_table(&TableId(id)).await;
    }
}

pub unsafe fn insert_row(table_id: u64, key: u64, data: *const u8, len: u32) {
    info!("Inserting row({}) @{:3<} for table {}", len, key, table_id);

    let data = std::slice::from_raw_parts(data, len as usize);

    crate::utils::print_hex_view(data);

    run_async!{
        if let Some(table) = super::runtime::get_table(&TableId(table_id)).await {
            table.set(key, data.to_vec()).await;
        }
    }
}

pub unsafe fn update_row(table_id: u64, key: u64, _data: *const u8, new_data: *const u8, len: u32) {
    info!("Updating row({}) @{:3<} for table {}", len, key, table_id);

    let new_data = std::slice::from_raw_parts(new_data, len as usize);

    run_async!{
        if let Some(table) = super::runtime::get_table(&TableId(table_id)).await {
            table.set(key, new_data.to_vec()).await;
        }
    }
}

pub fn delete_row(table_id: u64, key: u64) {
    info!("Deleting row @{:3<} for table {}", key, table_id);

    run_async!{
        if let Some(table) = super::runtime::get_table(&TableId(table_id)).await {
            table.delete(key).await;
        }
    }
}

pub unsafe fn put_hex(data: *const u8, len: u32) {
    let data = std::slice::from_raw_parts(data, len as usize);
    crate::utils::print_hex_view(data);
}

pub fn rnd_init(_table_id: u64) {
    // if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
    //     debug!("Init round for table {}", table_id);
    //     table.read_init();
    // }
}

pub fn rnd_end(_table_id: u64) {
    // if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
    //     debug!("End round for table {}", table_id);
    //     table.read_end();
    // }
}

pub unsafe fn rnd_next(_table_id: u64, _buf: *mut u8, _len: u32) -> i32 {
    // if let Some(table) = super::runtime::get_table(&TableId(table_id)) {
    //     debug!("Read next row for table {}", table_id);
    //     table.read_next(buf, len)
    // } else {
    //     0
    // }
    0
}
