use crate::storage::Storage;
use crate::utils;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct Network {
    pub storage: Arc<Storage>,
    pub listen_addr: SocketAddr,
}

impl Network {
    pub fn new(storage: Arc<Storage>, addr: SocketAddr) -> Self {
        Network { storage, listen_addr: addr }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(self.listen_addr).await.expect("bind failed");
        log::info!("TCP listener running on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((mut socket, peer)) => {
                    let storage = self.storage.clone();
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 1024];
                        match socket.read(&mut buf).await {
                            Ok(n) if n > 0 => {
                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                log::info!("Received from {}: {}", peer, msg.trim());
                                // simple protocol: "PUT key value" or "GET key"
                                let parts: Vec<&str> = msg.trim().splitn(3, ' ').collect();
                                let response = match parts.as_slice() {
                                    ["PUT", key, value] => {
                                        if let Err(e) = storage.set(key, value) {
                                            format!("ERR {}\n", e)
                                        } else {
                                            format!("OK\n")
                                        }
                                    }
                                    ["GET", key] => {
                                        match storage.get(key) {
                                            Some(v) => format!("VALUE {}\n", v),
                                            None => "NOTFOUND\n".into(),
                                        }
                                    }
                                    _ => "ERR invalid\n".into(),
                                };
                                let _ = socket.write_all(response.as_bytes()).await;
                            }
                            _ => {
                                log::warn!("Empty read or error from {}", peer);
                            }
                        }
                    });
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    }
}