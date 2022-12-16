#[macro_use]
mod macros;

pub mod bloom_filter;
pub mod data_store;
pub mod error;
pub mod io_handler;
pub mod logger;

use bincode::config::*;
use std::fmt::Write;

pub use bloom_filter::*;
pub use data_store::*;
pub use error::*;
pub use io_handler::*;
pub use macros::*;

pub type BincodeConfig = Configuration;
pub const BIN_CODE_CONF: BincodeConfig = bincode::config::standard();

const HEX_VIEW_WIDTH: usize = 32;
const HEX_VIEW_COL_WIDTH: usize = 8;

pub fn print_hex_view(buffer: &[u8]) -> Result<()> {
    let mut buf = String::new();
    for i in (0..buffer.len()).step_by(HEX_VIEW_WIDTH) {
        write!(&mut buf, "| {:08x} | ", i)?;
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

    debug!("Hex view for buffer ({} bytes):\n\n{}", buffer.len(), buf);
    Ok(())
}
