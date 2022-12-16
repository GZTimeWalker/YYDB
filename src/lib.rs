#![allow(dead_code)]

#[macro_use]
extern crate bincode;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
mod utils;

mod bridge;
pub mod core;
mod structs;

/// Init the rust part of the library, should be called
/// from C++ code by Mysql plugin init function.
pub fn rust_init() {
    utils::logger::init();
    core::runtime::init();

    info!("YYDB Initialized.");
}

pub fn rust_deinit() {
    // do something
    info!("YYDB Deinitialized.");
}
