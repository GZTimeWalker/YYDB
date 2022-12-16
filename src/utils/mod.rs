#[macro_use]
mod macros;

pub mod bloom_filter;
pub mod data_store;
pub mod error;
pub mod io_handler;
pub mod logger;

pub use bloom_filter::*;
pub use data_store::*;
pub use error::*;
pub use io_handler::*;
pub use macros::*;

pub fn print_hex_view(buffer: &[u8]) {
    let mut i = 0;

    while i < buffer.len() {
        // Print the line number.
        print!("{:08x} | ", i);

        // Print the hexadecimal values of each byte in the line.
        for j in 0..32 {
            if i + j < buffer.len() {
                print!("{:02x}", buffer[i + j]);
            } else {
                // If there are not enough bytes to fill the line,
                // print spaces to align the ASCII values.
                print!("  ");
            }

            if j % 16 == 15 {
                print!(" ");
            }
        }
        print!("| ");

        // Print the ASCII values of each byte in the line.
        for j in 0..32 {
            if i + j < buffer.len() {
                if buffer[i + j].is_ascii_graphic() {
                    // If the byte is an ASCII character, print it.
                    print!("{}", buffer[i + j] as char);
                } else {
                    // If the byte is not an ASCII character, print a dot.
                    print!(".");
                }
            } else {
                // If there are not enough bytes to fill the line,
                // print spaces to align the ASCII values.
                print!(" ");
            }
        }
        println!();

        i += 32;
    }
}
