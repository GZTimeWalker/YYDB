fn main() {
    let profile = std::env::var("PROFILE").unwrap();

    cxx_build::bridge("src/bridge.rs")
        .file("src/bridge.cc")
        .flag_if_supported("-std=c++17")
        .opt_level(3)
        .compile("bridge");

    let mut build = cc::Build::new();

    let mysql_src = std::env::var("MYSQL_SOURCE_DIR");

    if let Ok(src) = mysql_src {
        build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .opt_level(3)
        .include(format!("{}/include", src));
    } else {
        build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .opt_level(3)
        .include("include")
        .include("include/mysql")
        .include("include/mysql/include");
    }

    if profile == "debug" {
        build.file("src/handler/ha_wapper.cc").compile("ha_wapper");
    }

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=src/handler/ha_wapper.cc");
    println!("cargo:rerun-if-changed=include/ha_wapper.h");
    println!("cargo:rerun-if-changed=include/bridge.h");
}
