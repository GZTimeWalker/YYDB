#[cxx::bridge]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {
        pub fn rust_test();
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yengine/include/bridge.h");

        pub fn do_test();
    }
}

pub fn rust_test() {
    let a = 1;
    let b = 2;

    println!("{} + {} = {} -- from rust", a, b, a + b);
    ffi::do_test();
}
