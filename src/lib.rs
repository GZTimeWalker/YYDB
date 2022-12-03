#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
mod utils;

mod bridge;
mod core;

pub use crate::core::*;

/// Init the rust part of the library, should be called
/// from C++ code by Mysql plugin init function.
pub fn rust_init() {
    utils::logger::init();
    core::runtime::init();

    info!("Y-Engine Initialized.");
}
