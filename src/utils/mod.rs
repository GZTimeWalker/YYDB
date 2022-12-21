#[macro_use]
mod macros;

pub mod bloom_filter;
pub mod data_store;
pub mod deque_iter;
pub mod error;
pub mod io_handler;
pub mod logger;

use async_compression::Level;
use bincode::config::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Write;

pub use bloom_filter::*;
pub use data_store::*;
pub use deque_iter::*;
pub use error::*;
pub use io_handler::*;
pub use macros::*;

pub type CompressionEncoder<T> = async_compression::tokio::write::DeflateEncoder<T>;
pub type CompressionDecoder<T> = async_compression::tokio::bufread::DeflateDecoder<T>;
pub const COMPRESSION_LEVEL: Level = Level::Default;

pub type BincodeConfig = Configuration;
pub const BIN_CODE_CONF: BincodeConfig = bincode::config::standard();

pub fn new_progress_bar(size: u64) -> ProgressBar {
    let bar = ProgressBar::new(size);
    bar.set_style(bar_default_style());
    bar
}

pub fn bar_default_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(
            "    {spinner:.2} [{elapsed_precise:.2}] [{bar:40.6/4} {percent:>3}%] {pos:>7}/{len:7} {per_sec:<7.2}",
        )
        .unwrap()
        .progress_chars("#>-")
}

const HEX_VIEW_WIDTH: usize = 32;
const HEX_VIEW_COL_WIDTH: usize = 8;

pub fn hex_view(buffer: &[u8]) -> Result<String> {
    let mut buf = String::new();
    for i in (0..buffer.len()).step_by(HEX_VIEW_WIDTH) {
        write!(&mut buf, "| {i:08x} | ")?;
        for j in 0..HEX_VIEW_WIDTH {
            if i + j < buffer.len() {
                write!(&mut buf, "{:02x}", buffer[i + j])?;
            } else {
                write!(&mut buf, "  ")?;
            }
            if j % HEX_VIEW_COL_WIDTH == HEX_VIEW_COL_WIDTH - 1 {
                write!(&mut buf, " ")?;
            }
        }
        write!(&mut buf, "| ")?;
        for j in 0..HEX_VIEW_WIDTH {
            if i + j < buffer.len() {
                if buffer[i + j].is_ascii_graphic() {
                    write!(&mut buf, "{}", buffer[i + j] as char)?;
                } else {
                    write!(&mut buf, ".")?;
                }
            } else {
                write!(&mut buf, " ")?;
            }
        }
        writeln!(&mut buf)?;
    }

    Ok(format!(
        "Hex view for buffer ({} bytes):\n\n{}",
        buffer.len(),
        buf
    ))
}

pub fn print_hex_view(buffer: &[u8]) -> Result<()> {
    println!("{}", hex_view(buffer)?);

    Ok(())
}
