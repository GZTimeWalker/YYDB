fn main() {
    cxx_build::bridge("src/bridge.rs")
        .file("src/handler/ha_wapper.cc")
        .flag_if_supported("-std=c++2a")
        .include("include/mysql")
        .include("include/mysql/include")
        .compile("ha_wapper");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/handler/ha_wapper.cc");
    println!("cargo:rerun-if-changed=include/ha_wapper.hpp");
}
