fn main() {
    let profile = std::env::var("PROFILE").unwrap();

    cxx_build::bridge("src/bridge.rs")
        .file("src/bridge.cc")
        .flag_if_supported("-std=c++17")
        .opt_level(3)
        .compile("bridge");

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .opt_level(3)
        .include("include")
        .include("include/mysql")
        .include("include/mysql/include");

    if profile == "debug" {
        build.file("src/handler/ha_wapper.cc").compile("ha_wapper");
    } else {
        println!("cargo:warning={:?}", build.get_compiler());
    }

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=src/handler/ha_wapper.cc");
    println!("cargo:rerun-if-changed=include/ha_wapper.h");
    println!("cargo:rerun-if-changed=include/bridge.h");
}
