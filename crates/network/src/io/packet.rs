use std::pin::Pin;
use async_std::net::TcpStream;

/// Packet processor to pack packets from tcp stream.
#[derive(Debug)]
pub struct PacketHandler<'a> {
    stream: Pin<&'a mut TcpStream>,
}

impl<'a> PacketHandler<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            stream: Pin::new(stream),
        }
    }

    pub async fn next_packet(&mut self) -> anyhow::Result<Box<dyn Packet>> {
        Ok(Box::new(DummyPacket))
    }
}

/// Dynamic dispatch base of packet
pub trait Packet {
    fn length(&self) -> usize;
    fn packet_id(&self) -> u32;
}

#[derive(Debug)]
pub struct DummyPacket;

impl Packet for DummyPacket {
    fn length(&self) -> usize { 0 }
    fn packet_id(&self) -> u32 { u32::MAX }
}
