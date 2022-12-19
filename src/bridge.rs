use crate::core::bridge::*;
use crate::*;

#[cxx::bridge(namespace = "yydb")]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {
        // Init the rust part of the library.
        pub fn rust_init();

        // Deinit the rust part of the library.
        pub fn rust_deinit();

        // Open a table by name.
        pub fn open_table(table_name: &str) -> u64;

        // close a table by id.
        pub fn close_table(id: u64);

        // insert a row to a table.
        pub unsafe fn insert_row(table_id: u64, key: u64, data: *const u8, len: u32);

        // update a row to a table.
        pub unsafe fn update_row(
            table_id: u64,
            key: u64,
            data: *const u8,
            new_data: *const u8,
            len: u32,
        );

        // delete a row to a table.
        pub unsafe fn delete_row(table_id: u64, key: u64);

        // init round
        pub fn rnd_init(table_id: u64);

        // end round
        pub fn rnd_end(table_id: u64);

        // read next row
        pub unsafe fn rnd_next(table_id: u64, buf: *mut u8, len: u32) -> i32;

        // put hex data to log
        pub unsafe fn put_hex(data: *const u8, len: u32);

        // delete a table by name.
        pub fn delete_table(table_name: &str);
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yydb/include/bridge.h");

        // write log to mysql log
        pub fn mysql_log_write(level: i32, message: &str);
    }
}
