#[cxx::bridge]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {

    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yengine/include/ha_wapper.h");

        pub fn do_test();
    }
}
