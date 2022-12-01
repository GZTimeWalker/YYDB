use crate::*;

#[cxx::bridge(namespace = "yengine")]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {
        // Init the rust part of the library.
        pub fn rust_init();

        pub fn rust_test();
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yengine/include/bridge.h");

        // write log to mysql log
        pub fn mysql_log_write(level: i32, message: &str);

        pub fn do_test();
    }
}

pub fn rust_test() {
    let a = 1;
    let b = 2;

    println!("{} + {} = {} -- from rust", a, b, a + b);
    ffi::do_test();
}
