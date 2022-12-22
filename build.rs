fn main() {
    if std::env::var("CARGO_FEATURE_MYSQL").is_ok() {
        cxx_build::bridge("src/bridge.rs")
            .define("RUST_DEBUG", None)
            .file("src/bridge.cc")
            .flag_if_supported("-std=c++17")
            .opt_level(3)
            .compile("bridge");

        println!("cargo:rerun-if-changed=src/bridge.rs");
        println!("cargo:rerun-if-changed=src/bridge.cc");
        println!("cargo:rerun-if-changed=include/bridge.h");
    }
}
