
use dolls_macros::packet_processor;
use crate::io::packet::raw::RawPacket;

#[packet_processor(0)]
fn handshake_packet(packet: RawPacket) -> anyhow::Result<()> {
    println!("Handshake: {:?}", packet);

    Ok(())
}
