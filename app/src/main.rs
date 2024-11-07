#![feature(future_join)]

use std::future::join;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task::{block_on};
use spdlog::info;
use dolls_network::prelude::DollNetworkServer;

#[derive(Debug)]
pub(crate) struct App {
    network_server: Arc<Mutex<DollNetworkServer>>,
}

// Tell compiler this is none of its business.
unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {
    pub fn new() -> Self {
        Self {
            network_server: Arc::new(Mutex::new(DollNetworkServer::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                25565
            ))),
        }
    }

    pub async fn start(&mut self) {
        let network_handle = {
            let network_server = self.network_server.clone();
            async_std::task::Builder::new()
                .name("Network TCP Listener".to_string())
                .spawn( async move {
                    info!("Network service started.");
                    network_server.lock().await.accept().await;
                }).unwrap()
        };

        block_on(async {
            join!(network_handle).await;
        });
    }
}

fn main() {
    init_logger();
    info!("Running {} version {}.", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let mut app = App::new();
    block_on(app.start());
}

fn init_logger() {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);
}
