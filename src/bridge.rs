use crate::*;
use crate::core::bridge::*;

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
        pub unsafe fn insert_row(table_id: u64, row: *const u8, len: u32);

        // update a row to a table.
        pub unsafe fn update_row(table_id: u64, row: *const u8, new_row: *const u8, len: u32);
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yydb/include/bridge.h");

        // write log to mysql log
        pub fn mysql_log_write(level: i32, message: &str);
    }
}
