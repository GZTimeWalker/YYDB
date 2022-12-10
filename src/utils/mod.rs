#[macro_use]
mod macros;

pub mod logger;

pub use macros::*;

pub fn print_hex_view(buffer: &[u8]) {
    let mut line_number = 0;
    let mut i = 0;

    while i < buffer.len() {
        // Print the line number.
        print!("{:08x} ", line_number);

        // Print the hexadecimal values of each byte in the line.
        for j in 0..16 {
            if i + j < buffer.len() {
                print!("{:02x} ", buffer[i + j]);
            } else {
                // If there are not enough bytes to fill the line,
                // print spaces to align the ASCII values.
                print!("   ");
            }
        }
        print!("|");

        // Print the ASCII values of each byte in the line.
        for j in 0..16 {
            if i + j < buffer.len() {
                if buffer[i + j].is_ascii() {
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
        println!("");

        line_number += 16;
        i += 16;
    }
}
