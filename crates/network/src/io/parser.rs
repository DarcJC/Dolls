use async_std::net::TcpStream;
use async_std::io;
use async_std::prelude::*; // For the `read_exact` method

pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

/// Reads a VarInt from the provided `TcpStream`.
///
/// # Errors
///
/// Returns an `io::Error` if the VarInt is too big or if there is an I/O error.
pub async fn read_varint(stream: &mut TcpStream) -> io::Result<u32> {

    let mut value: u32 = 0;
    let mut position: u32 = 0;

    loop {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer).await?;
        let current_byte = buffer[0];

        value |= ((current_byte & SEGMENT_BITS) as u32) << position;

        if (current_byte & CONTINUE_BIT) == 0 {
            break;
        }

        position += 7;

        if position >= 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt is too big"));
        }
    }

    Ok(value)
}
