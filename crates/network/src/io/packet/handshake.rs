
use dolls_macros::packet_processor;
use crate::prelude::{PacketType, RawPacket};

#[packet_processor(PacketType::Handshake)]
fn handshake_packet(packet: RawPacket) -> anyhow::Result<()> {
    println!("Handshake: {:?}", packet);

    Ok(())
}
