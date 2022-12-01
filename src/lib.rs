pub mod bridge;
mod utils;

#[macro_use]
extern crate log;

pub use bridge::*;

// Init the rust part of the library.
// will be called from C++ code
pub fn rust_init() {
    utils::logger::init();

    info!("Y-Engine Initialized.");
}


#[test]
#[cfg(test)]
fn test_logger() {
    utils::logger::init();
    info!("Y-Engine Logger Test");
    warn!("Y-Engine Logger Test");
    error!("Y-Engine Logger Test");
}
