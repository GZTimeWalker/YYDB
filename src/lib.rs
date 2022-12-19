#![allow(dead_code)]
#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate bincode;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
mod utils;

#[cfg(not(test))]
mod bridge;

pub mod core;
mod structs;

/// Init the rust part of the library, should be called
/// from C++ code by Mysql plugin init function.
#[inline]
pub fn rust_init() {
    utils::logger::init();
    core::runtime::init();

    info!("YYDB Version: {} Initialized.", env!("CARGO_PKG_VERSION"));
}

#[inline]
pub fn rust_deinit() {
    core::runtime::deinit();

    info!("YYDB Deinitialized.");
}
