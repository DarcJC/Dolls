use async_std::sync::RwLock;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use crate::prelude::{PacketType, RawPacket};

pub type PacketProcessorFn = fn(RawPacket) -> anyhow::Result<()>;

static HANDLERS: Lazy<RwLock<HashMap<u32, PacketProcessorFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

#[macro_export]
macro_rules! register_packet_processor {
    ($packet_id:expr, $handler:expr) => {
        inventory::submit! {
            PacketProcessorRegistration {
                packet_id: $packet_id,
                processor: $handler,
            }
        }
    }
}

pub struct PacketProcessorRegistration {
    pub packet_id: PacketType,
    pub processor: PacketProcessorFn,
}

inventory::collect!(PacketProcessorRegistration);

pub async fn init_packet_processors() {
    if HANDLERS.read().await.is_empty() {
        for registration in inventory::iter::<PacketProcessorRegistration> {
            HANDLERS.write().await.insert(registration.packet_id as u32, registration.processor);
        }
    }
}

pub async fn get_handler(packet_id: u32) -> Option<PacketProcessorFn> {
    HANDLERS.read().await.get(&packet_id).cloned()
}
