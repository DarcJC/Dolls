use async_std::{io, stream};
use async_std::io::ReadExt;
use thiserror::Error;

// For the `read_exact` method
pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

/// Error type for parsing operations.
/// This enum represents various errors that can occur during parsing operations.
#[derive(Debug, Error)]
pub enum ParsingError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// VarInt is too big
    #[error("VarInt is too big")]
    VarIntTooBig,
    /// VarLong is too big
    #[error("VarLong is too big")]
    VarLongTooBig,
    /// Invalid UTF-8 string
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    /// Identifier too long
    #[error("Identifier too long")]
    IdentifierTooLong(String),
    /// UUID parsing error
    #[error("UUID parsing error: {0}")]
    Uuid(#[from] uuid::Error),
}

pub type Result<T> = std::result::Result<T, ParsingError>;

/// Reads a VarInt from the provided `TcpStream`.
/// An integer between -2147483648 and 2147483647.
/// Variable-length data encoding a two's complement signed 32-bit integer; more info in their section
///
/// # Errors
///
/// Returns an `io::Error` if the VarInt is too big or if there is an I/O error.
pub async fn read_varint(stream: &mut (impl ReadExt + Unpin)) -> Result<u32> {

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
            return Err(ParsingError::VarIntTooBig);
        }
    }

    Ok(value)
}

/// Reads a VarInt and actual size of it from the provided `TcpStream`.
///
/// # Errors
///
/// Returns an `io::Error` if the VarInt is too big or if there is an I/O error.
pub async fn read_varint_and_get_size(stream: &mut (impl ReadExt + Unpin)) -> Result<(u32, u32)> {
    let mut value: u32 = 0;
    let mut position: u32 = 0;
    let mut size = 0;

    loop {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer).await?;
        size = size + 1;
        let current_byte = buffer[0];

        value |= ((current_byte & SEGMENT_BITS) as u32) << position;

        if (current_byte & CONTINUE_BIT) == 0 {
            break;
        }

        position += 7;

        if position >= 32 {
            return Err(ParsingError::VarIntTooBig);
        }
    }

    Ok((value, size))
}

/// Reads a VarLong from the provided `TcpStream`.
/// An integer between -9223372036854775808 and 9223372036854775807.
/// Variable-length data encoding a two's complement signed 64-bit integer.
pub async fn read_varlong(stream: &mut (impl ReadExt + Unpin)) -> Result<u64> {
    let mut value: u64 = 0;
    let mut position: u32 = 0;

    loop {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer).await?;
        let current_byte = buffer[0];

        value |= ((current_byte & SEGMENT_BITS) as u64) << position;

        if (current_byte & CONTINUE_BIT) == 0 {
            break;
        }

        position += 7;

        if position >= 64 {
            return Err(ParsingError::VarLongTooBig);
        }
    }

    Ok(value)
}

/// Reads a specified number of bytes from the provided `TcpStream`.
/// The number of bytes to read is specified by the `size` parameter.
pub async fn read_exact_bytes(stream: &mut (impl ReadExt + Unpin), size: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; size];
    stream.read_exact(&mut buffer).await?;
    Ok(buffer)
}

/// Reads a specified number of bytes from the provided `TcpStream` into the provided buffer.
/// The number of bytes to read is specified by the `size` parameter.
/// The buffer is resized to the specified size before reading.
///
/// # Errors
///
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_exact_bytes_into_buffer(stream: &mut (impl ReadExt + Unpin), size: usize, output_buffer: &mut Vec<u8>) -> Result<()> {
    output_buffer.resize(size, 0);
    stream.read_exact(output_buffer).await?;
    Ok(())
}

/// Reads a boolean value from the provided `TcpStream`.
///
/// True is encoded as 0x01, false as 0x00.
/// 
/// # Errors
/// 
/// Returns an `io::Error` if there is an I/O error.
/// The boolean value is represented as a single byte, where `0` is `false` and any other value is `true`.
pub async fn read_boolean(stream: &mut (impl ReadExt + Unpin)) -> Result<bool> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer).await?;
    Ok(buffer[0] != 0)
}

/// Reads a u8 value from the provided `TcpStream`.
/// Unsigned 8-bit integer
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
/// The u8 value is represented as a single byte.
pub async fn read_u8(stream: &mut (impl ReadExt + Unpin)) -> Result<u8> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer).await?;
    Ok(buffer[0])
}

/// Reads a i8 value from the provided `TcpStream`.
/// Signed 8-bit integer, two's complement
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
/// The i8 value is represented as a single byte.
pub async fn read_i8(stream: &mut (impl ReadExt + Unpin)) -> Result<i8> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer).await?;
    Ok(buffer[0] as i8)
}

/// Reads a u16 value from the provided `TcpStream`.
/// Unsigned 16-bit integer
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
/// The u16 value is represented as two bytes in big-endian order.
pub async fn read_u16(stream: &mut (impl ReadExt + Unpin)) -> Result<u16> {
    let mut buffer = [0u8; 2];
    stream.read_exact(&mut buffer).await?;
    Ok(u16::from_be_bytes(buffer))
}

/// Reads a i16 value from the provided `TcpStream`.
/// Signed 16-bit integer, two's complement
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_i16(stream: &mut (impl ReadExt + Unpin)) -> Result<i16> {
    let mut buffer = [0u8; 2];
    stream.read_exact(&mut buffer).await?;
    Ok(i16::from_be_bytes(buffer))
}

/// Reads a i32 value from the provided `TcpStream`.
/// Signed 32-bit integer, two's complement
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
/// The i32 value is represented as four bytes in big-endian order.
pub async fn read_i32(stream: &mut (impl ReadExt + Unpin)) -> Result<i32> {
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer).await?;
    Ok(i32::from_be_bytes(buffer))
}

/// Reads a i64 value from the provided `TcpStream`.
//// Signed 64-bit integer, two's complement
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_i64(stream: &mut (impl ReadExt + Unpin)) -> Result<i64> {
    let mut buffer = [0u8; 8];
    stream.read_exact(&mut buffer).await?;
    Ok(i64::from_be_bytes(buffer))
}

/// Reads a float value from the provided `TcpStream`.
/// A single-precision 32-bit IEEE 754 floating point number
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_float(stream: &mut (impl ReadExt + Unpin)) -> Result<f32> {
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer).await?;
    Ok(f32::from_be_bytes(buffer))
}

/// Reads a double value from the provided `TcpStream`.
/// A double-precision 64-bit IEEE 754 floating point number
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_double(stream: &mut (impl ReadExt + Unpin)) -> Result<f64> {
    let mut buffer = [0u8; 8];
    stream.read_exact(&mut buffer).await?;
    Ok(f64::from_be_bytes(buffer))
}

/// Reads a string from the provided `TcpStream`.
/// UTF-8 string prefixed with its size in bytes as a VarInt. 
/// Maximum length of n characters, which varies by context. 
/// The encoding used on the wire is regular UTF-8, not Java's "slight modification". 
/// However, the length of the string for purposes of the length limit is its number of UTF-16 code units, that is, scalar values > U+FFFF are counted as two. 
/// Up to n × 3 bytes can be used to encode a UTF-8 string comprising n code units when converted to UTF-16, and both of those limits are checked. 
/// Maximum n value is 32767. The + 3 is due to the max size of a valid length VarInt.
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error or if the string is not valid UTF-8.
pub async fn read_string(stream: &mut (impl ReadExt + Unpin)) -> Result<String> {
    let length = read_varint(stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;
    Ok(String::from_utf8(buffer).unwrap())
}

/// Reads a JSON object from the provided `TcpStream`.
/// The maximum permitted length when decoding is 262144, 
/// but the vanilla server since 1.20.3 refuses to encode longer than 32767. This may be a bug.
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error or if the JSON is not valid.
pub async fn read_json<T: serde::de::DeserializeOwned>(stream: &mut (impl ReadExt + Unpin)) -> Result<T> {
    let length = read_varint(stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;
    let json_str = String::from_utf8(buffer).unwrap();
    let json: T = serde_json::from_str(&json_str).unwrap();
    Ok(json)
}

/// Reads an identifier from the provided `TcpStream`.
/// Encoded as a String with max length of 32767.
pub async fn read_identifier(stream: &mut (impl ReadExt + Unpin)) -> Result<String> {
    let result = read_string(stream).await;

    // check if the length is greater than 32767
    if result.as_ref().map(|s| s.len()).unwrap_or(0) > 32767 {
        return Err(ParsingError::IdentifierTooLong(result.unwrap()));
    }

    result
}

/// Reads a UUID from the provided `TcpStream`.
/// Miscellaneous information about an entity	.
pub async fn read_entity_metadata(_stream: &mut (impl ReadExt + Unpin)) -> Result<Vec<u8>> {
    todo!("Implement read_entity_metadata");
}

/// An item stack in an inventory or container	
pub async fn read_slot(_stream: &mut (impl ReadExt + Unpin)) -> Result<u8> {
    todo!("Implement read_slot");
}

/// Reads a text component from the provided `TcpStream`.
/// Encoded as a `NBT Tag`, with the type of tag used depending on the case:
/// As a `String Tag`: For components only containing text (no styling, no events etc.).
/// As a `Compound Tag`: Every other case.
pub async fn read_text_component(_stream: &mut (impl ReadExt + Unpin)) -> Result<String> {
    todo!("Implement read_text_component");
}

/// Reads a `Position` from the provided `TcpStream`.
/// Encoded as a `Long` in the format `x | (y << 38) | (z << 12)`.
/// An integer/block position: x (-33554432 to 33554431), z (-33554432 to 33554431), y (-2048 to 2047)
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_position(stream: &mut (impl ReadExt + Unpin)) -> Result<u64> {
    let mut buffer = [0u8; 8];
    stream.read_exact(&mut buffer).await?;
    let x = i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let y = i32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    let z = i32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    let position = ((x as u64) & 0x3FFFFFFF) | (((y as u64) & 0x3FFF) << 38) | (((z as u64) & 0x3FFFFFFF) << 12);
    Ok(position)
}

/// Reads a `Rotation` from the provided `TcpStream`.
/// Whether or not this is signed does not matter, since the resulting angles are the same.
/// A rotation angle in steps of 1/256 of a full turn.
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_angle(stream: &mut (impl ReadExt + Unpin)) -> Result<u8> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer).await?;
    Ok(buffer[0])
}

/// Reads a `UUID` from the provided `TcpStream`.
/// Encoded as an unsigned 128-bit integer (or two unsigned 64-bit integers: 
/// the most significant 64 bits and then the least significant 64 bits)
///
/// # Errors
/// Returns an `io::Error` if there is an I/O error or if the UUID is not valid.
pub async fn read_uuid(stream: &mut (impl ReadExt + Unpin)) -> Result<uuid::Uuid> {
    let mut buffer = [0u8; 16];
    stream.read_exact(&mut buffer).await?;
    let uuid = uuid::Uuid::from_slice(&buffer)?;
    Ok(uuid)
}

/// Reads a bitset from the provided `TcpStream`.
/// Bit sets of type BitSet are prefixed by their length in longs.
/// Layout: | Length: VarInt | Long[Length] |
/// The *i*th bit is set when `(Data[i / 64] & (1 << (i % 64))) != 0`, where `i` starts at `0`.
pub async fn read_bitset(stream: &mut (impl ReadExt + Unpin)) -> Result<Vec<bool>> {
    let length = read_varint(stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;
    let mut result = Vec::new();
    for byte in buffer {
        for i in 0..8 {
            result.push((byte >> i) & 1 == 1);
        }
    }
    Ok(result)
}

/// Reads a fixed-size bitset from the provided `TcpStream`.
/// Bit sets of type Fixed BitSet (n) have a fixed length of n bits, encoded as ceil(n / 8) bytes. 
/// Note that this is different from BitSet, which uses longs.
/// The ith bit is set when `(Data[i / 8] & (1 << (i % 8))) != 0`, where `i` starts at `0`.
/// This encoding is not equivalent to the long array in BitSet.
/// 
/// # Errors
/// Returns an `io::Error` if there is an I/O error.
pub async fn read_fixed_bitset(stream: &mut (impl ReadExt + Unpin), size: usize) -> Result<Vec<bool>> {
    let mut buffer = vec![0u8; size];
    stream.read_exact(&mut buffer).await?;
    let mut result = Vec::new();
    for byte in buffer {
        for i in 0..8 {
            result.push((byte >> i) & 1 == 1);
        }
    }
    Ok(result)
}

bitflags::bitflags! {
    /// Bit field specifying how a teleportation is to be applied on each axis.
    /// A bit field represented as an Int, specifying how a teleportation is to be applied on each axis.
    #[derive(Default)]
    pub struct TeleportFlags: u32 {
        const RelativeX = 0x01;
        const RelativeY = 0x02;
        const RelativeZ = 0x04;
        const RelativeYaw = 0x08;
        const RelativePitch = 0x10;
        const RelativeVelocityX = 0x20;
        const RelativeVelocityY = 0x40;
        const RelativeVelocityZ = 0x80;
        const RotateVelocityBasedOnDeltaRotation = 0x100;
    }
}

/// Reads a teleport flag from the provided `TcpStream`.
/// Bit field specifying how a teleportation is to be applied on each axis.
/// A bit field represented as an Int, specifying how a teleportation is to be applied on each axis.
pub async fn read_teleport_flags(stream: &mut (impl ReadExt + Unpin)) -> Result<TeleportFlags> {
    let value = read_i32(stream).await?;
    // In the lower 8 bits of the bit field,
    // a set bit means the teleportation on the corresponding axis is relative, and an unset bit that it is absolute.
    Ok(TeleportFlags::from_bits_truncate(value as u32))
}
