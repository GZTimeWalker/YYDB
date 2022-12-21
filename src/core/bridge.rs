use crate::{
    structs::{kvstore::*, table::TableId},
    utils::DataStore,
};

#[inline(always)]
pub fn open_table(table_name: &str) -> u64 {
    let table_name = table_name.to_string();
    info!("Opening table       : {}", table_name);

    run_async! {
        if let Some(id) = super::Runtime::global().open_table(table_name).await {
            id.0
        } else {
            0
        }
    }
}

#[inline(always)]
pub fn close_table(id: u64) {
    run_async! {
        super::Runtime::global().close_table(&TableId(id)).await;
    }
}

/// # Safety
/// mysql will pass a pointer to a buffer, and we need to get data from it
pub unsafe fn insert_row(table_id: u64, key: u64, data: *const u8, len: u32) {
    trace!(
        "Inserting row       : [{:3<}]<{}> @{:016x}",
        key, len, table_id
    );

    let data = std::slice::from_raw_parts(data, len as usize);

    if log::max_level() >= log::LevelFilter::Trace {
        crate::utils::print_hex_view(data).unwrap();
    }

    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            table.set(key, data.to_vec()).await;
        } else {
            warn!("Table not found     : @{:016x}", table_id);
        }
    }
}

/// # Safety
/// mysql will pass a pointer to a buffer, and we need to get data from it
pub unsafe fn update_row(table_id: u64, key: u64, _data: *const u8, new_data: *const u8, len: u32) {
    trace!(
        "Updating row        : [{:3<}]<{}> @{:016x}",
        key, len, table_id
    );

    let new_data = std::slice::from_raw_parts(new_data, len as usize);

    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            table.set(key, new_data.to_vec()).await;
        } else {
            warn!("Table not found     : @{:016x}", table_id);
        }
    }
}

pub fn delete_row(table_id: u64, key: u64) {
    trace!("Deleting row        : [{:3<}] @{:016x}", key, table_id);

    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            table.delete(key).await;
        } else {
            warn!("Table not found     : @{:016x}", table_id);
        }
    }
}

/// # Safety
/// mysql will pass a pointer to a buffer, and we need to fill it with data
pub unsafe fn put_hex(data: *const u8, len: u32) {
    let data = std::slice::from_raw_parts(data, len as usize);
    crate::utils::print_hex_view(data).unwrap();
}

pub fn rnd_init(table_id: u64) {
    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            trace!("Init iter round     : @{:016x}", table_id);
            table.init_iter().await;
        } else {
            warn!("Table not found     : @{:016x}", table_id);
        }
    }
}

pub fn rnd_end(table_id: u64) {
    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            trace!("End iter round      : @{:016x}", table_id);
            table.end_iter().await;
        } else {
            warn!("Table not found     : @{:016x}", table_id);
        }
    }
}

/// # Safety
/// mysql will pass a pointer to a buffer, and we need to fill it with data
pub unsafe fn rnd_next(table_id: u64, buf: *mut u8, len: u32) -> i32 {
    let buf = std::slice::from_raw_parts_mut(buf, len as usize);

    run_async! {
        if let Some(table) = super::Runtime::global().get_table(&TableId(table_id)).await {
            trace!("Read next row       : @{:016x}", table_id);
            match table.next().await {
                Ok(Some((_, DataStore::Value(value)))) => {
                    let value = value.as_slice();

                    if log::max_level() >= log::LevelFilter::Trace {
                        crate::utils::print_hex_view(value).unwrap();
                    }

                    let len = len as usize;
                    let value_len = value.len();

                    if len != value_len {
                        error!("Buffer size mismatch: {} != {}", len, value_len);
                        return -1;
                    }

                    buf.copy_from_slice(value);
                    1
                }
                Err(e) => {
                    error!("Error while reading next row: {:#?}", e);
                    -1
                },
                _ => 0
            }
        }  else {
            warn!("Table not found     : @{:016x}", table_id);
            -1
        }
    }
}

#[inline(always)]
pub fn delete_table(table_name: &str) {
    let table_name = table_name.to_string();
    let id = TableId::new(&table_name);

    run_async! {
        if super::Runtime::global().contains_table(&id).await {
            super::Runtime::global().close_table(&id).await;
        }

        // do fs cleanup
        std::fs::remove_dir_all(table_name).ok();
    }
}
