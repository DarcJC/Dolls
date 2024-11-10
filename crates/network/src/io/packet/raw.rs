
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PacketType {
    Handshake = 0x00,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawPacket {
    pub size_in_bytes: u32,
    pub packet_id: u32,
    pub payload: Vec<u8>,
}
