#[cxx::bridge]
pub mod ffi {
    // Rust types and signatures exposed to C++.
    extern "Rust" {

    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("yengine_bridge/include/ha_wapper.hpp");

        pub fn do_test();
    }
}
