use crate::config::Config;
use crate::network::Network;
use crate::storage::Storage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task;

pub struct SynkNode {
    pub id: String,
    pub tcp_port: u16,
    pub http_port: u16,
    pub storage: Arc<Storage>,
    pub peers: Vec<String>,
}

impl SynkNode {
    pub fn new(cfg: Config, storage: Storage) -> Self {
        SynkNode {
            id: cfg.node_id,
            tcp_port: cfg.tcp_port,
            http_port: cfg.http_port,
            storage: Arc::new(storage),
            peers: cfg.peers,
        }
    }

    pub async fn run_network(self: Arc<Self>) {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.tcp_port).parse().expect("invalid addr");
        let network = Network::new(self.storage.clone(), addr);
        // spawn the listener as a blocking task
        task::spawn(async move {
            network.run().await;
        }).await.ok(); // wait this task until it runs; we keep this simple
    }
}