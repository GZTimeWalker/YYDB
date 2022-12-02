use crate::*;

#[cxx::bridge(namespace = "yengine")]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {
        // Init the rust part of the library.
        pub fn rust_init();
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yengine/include/bridge.h");

        // write log to mysql log
        pub fn mysql_log_write(level: i32, message: &str);
    }
}
