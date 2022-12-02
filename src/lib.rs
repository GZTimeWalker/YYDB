#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
mod utils;

mod bridge;
mod core;

// Init the rust part of the library.
// will be called from C++ code
pub fn rust_init() {
    utils::logger::init();
    core::runtime::init();

    info!("Y-Engine Initialized.");
}
