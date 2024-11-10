mod raw;
mod processor;
mod handshake;

pub use raw::*;
pub use processor::*;

use std::pin::Pin;
use async_std::net::TcpStream;
use crate::prelude::{read_varint_and_get_size, read_varint, read_exact_bytes};

/// Packet processor to pack packets from tcp stream.
#[derive(Debug)]
pub struct PacketHandler<'a> {
    stream: Pin<&'a mut TcpStream>,
    enable_compression: bool,
}

impl<'a> PacketHandler<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            stream: Pin::new(stream),
            enable_compression: false,
        }
    }

    pub async fn next_packet(&mut self) -> anyhow::Result<RawPacket> {
        let length = read_varint(&mut *self.stream).await?;
        let (packet_id, packet_id_size) = read_varint_and_get_size(&mut *self.stream).await?;
        let data_length = length - packet_id_size;
        let payload = read_exact_bytes(&mut *self.stream, data_length as usize).await?;
        Ok(RawPacket {
            size_in_bytes: length,
            packet_id,
            payload,
        })
    }
}
