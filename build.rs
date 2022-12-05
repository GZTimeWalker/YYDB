fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let mysql_src =
        std::env::var("MYSQL_SOURCE_DIR").unwrap_or("/usr/local/src/mysql-8.0".to_string());

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
        .define("MYSQL_DYNAMIC_PLUGIN", None)
        .include("include")
        .include("include/mysql")
        .include(&mysql_src)
        .include(format!("{}/include", &mysql_src));

    if profile == "debug" {
        build
            .define("RUST_DEBUG", None)
            .file("src/handler/ha_wapper.cc")
            .file("src/handler/yydb.cc")
            .compile("ha_yydb");
    }

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=src/handler/ha_wapper.cc");
    println!("cargo:rerun-if-changed=src/handler/yydb.cc");
    println!("cargo:rerun-if-changed=include/ha_wapper.h");
    println!("cargo:rerun-if-changed=include/yydb.h");
    println!("cargo:rerun-if-changed=include/bridge.h");
}
