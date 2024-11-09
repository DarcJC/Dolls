use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::StreamExt;
use spdlog::{critical, debug, error};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use async_std::sync::Mutex;
use async_std::task::JoinHandle;
use crate::prelude::{get_handler, init_packet_processors, PacketHandler};

/// A TCP Server wrapper
#[derive(Debug)]
pub struct DollNetworkServer {
    ip_address: IpAddr,
    port: u16,
    is_running: AtomicBool,
    workers: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

/// Worker context
#[derive(Debug)]
struct WorkerContext {
    pub stream: TcpStream,
}

impl DollNetworkServer {
    pub fn new(ip_addr: IpAddr, port: u16) -> Self {
        Self {
            ip_address: ip_addr,
            port,
            is_running: AtomicBool::new(false),
            workers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn accept(&self) {
        if self.is_running.load(Ordering::Acquire) {
            critical!("DollNetworkServer already running");
            panic!("DollNetworkServer already running");
        }

        init_packet_processors().await;

        self.is_running.store(true, Ordering::Release);
        let tcp_listener = TcpListener::bind(SocketAddr::new(self.ip_address, self.port)).await.unwrap();
        let mut incoming = tcp_listener.incoming();

        while let Some(Ok(stream)) = incoming.next().await {
            debug!("Incoming stream from {}", stream.peer_addr().unwrap());
            self.workers.lock().await.push(DollNetworkServer::create_new_worker(stream));
        }
    }

    fn create_new_worker(stream: TcpStream) -> JoinHandle<()> {
        let mut worker_context = WorkerContext {
            stream,
        };
        async_std::task::spawn(async move {
            worker_context.stream.set_nodelay(true).unwrap();

            let socket_addr = worker_context.stream.peer_addr().unwrap();
            let mut packet_handler = PacketHandler::new(&mut worker_context.stream);

            while let Ok(packet) = packet_handler.next_packet().await {
                if let Some(func) = get_handler(packet.packet_id).await {
                    if let Err(err)  = func(packet) {
                        error!("Error processing packet: {}", err);
                    }
                } else {
                    error!("Unexpected packet(id={}) from client {:?}.", packet.packet_id, socket_addr);
                }
            }
        })
    }
}
