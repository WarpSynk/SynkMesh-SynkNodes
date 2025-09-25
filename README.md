
# SynkNodes

SynkNodes is a Rust-based node implementation for the WarpSynk Protocol.  
It provides peer-to-peer networking, local storage, cryptographic functions, and an HTTP API for external apps.  

## Features

- **Node identity** generated with SHA256 + UUID.  
- **Local storage** persisted in JSON files.  
- **TCP networking** for peer-to-peer communication.  
- **HTTP API** for external apps to query and store values.  
- **Configurable** via environment variables.  

## Project Structure

synknodes/ Cargo.toml src/ main.rs node.rs config.rs crypto.rs storage.rs network.rs api.rs utils.rs

## Install

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
git clone https://github.com/yourusername/synknodes.git
cd synknodes
cargo build --release

Run

cargo run

Or set environment variables:

SYNK_NODE_ID="synk-001" \
SYNK_TCP_PORT=7000 \
SYNK_HTTP_PORT=8080 \
SYNK_DATA_DIR="./data" \
SYNK_PEERS="127.0.0.1:7001,127.0.0.1:7002" \
cargo run

API

GET /status → Returns node id, ports, and stored keys.

GET /data/{key} → Returns the value for a stored key.

POST /store → Stores a key-value pair.


{
  "key": "username",
  "value": "alice"
}

TCP Protocol

PUT key value → stores a value

GET key → retrieves a value


echo "PUT test 123" | nc 127.0.0.1 7000
echo "GET test" | nc 127.0.0.1 7000

Example Code Snippets

Cargo.toml

[package]
name = "synknodes"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.41.0", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
rand = "0.8"
hex = "0.4"
uuid = { version = "1", features = ["v4"] }

main.rs

mod node;
mod config;
mod crypto;
mod storage;
mod network;
mod api;
mod utils;

use config::Config;
use node::SynkNode;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() {
    utils::init_logging();

    let cfg = Config::from_env();
    log::info!("Starting SynkNodes node with id {}", cfg.node_id);

    let storage = storage::Storage::new(&cfg.data_dir).expect("storage init failed");
    let node = Arc::new(SynkNode::new(cfg, storage));

    let node_clone = node.clone();
    let network_handle = tokio::spawn(async move {
        node_clone.run_network().await;
    });

    let node_clone = node.clone();
    let api_handle = tokio::spawn(async move {
        api::run_api(node_clone).await;
    });

    let _ = signal::ctrl_c().await;
    log::info!("Shutdown requested, stopping node");

    let _ = network_handle.abort();
    let _ = api_handle.abort();
}

config.rs

use std::env;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Config {
    pub node_id: String,
    pub host: String,
    pub tcp_port: u16,
    pub http_port: u16,
    pub data_dir: PathBuf,
    pub peers: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let node_id = env::var("SYNK_NODE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());
        let host = env::var("SYNK_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let tcp_port = env::var("SYNK_TCP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(7000);
        let http_port = env::var("SYNK_HTTP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        let data_dir = env::var("SYNK_DATA_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("./data"));
        let peers = env::var("SYNK_PEERS").map(|s| {
            s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
        }).unwrap_or_else(|_| vec![]);

        Config {
            node_id,
            host,
            tcp_port,
            http_port,
            data_dir,
            peers,
        }
    }
}

crypto.rs

use sha2::{Digest, Sha256};
use rand::Rng;
use hex;

pub fn generate_node_id() -> String {
    let mut rng = rand::thread_rng();
    let n: u128 = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(n.to_le_bytes());
    let res = hasher.finalize();
    hex::encode(res)
}

pub fn hash_data(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

storage.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Default)]
struct Store {
    map: HashMap<String, String>,
}

pub struct Storage {
    dir: PathBuf,
    file: PathBuf,
    inner: Mutex<Store>,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        let file = dir.join("store.json");
        let store = if file.exists() {
            let raw = fs::read_to_string(&file)?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            Store::default()
        };
        Ok(Storage {
            dir,
            file,
            inner: Mutex::new(store),
        })
    }

    pub fn set(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let mut s = self.inner.lock().unwrap();
        s.map.insert(key.to_string(), value.to_string());
        let serialized = serde_json::to_string_pretty(&*s)?;
        let mut f = fs::File::create(&self.file)?;
        f.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let s = self.inner.lock().unwrap();
        s.map.get(key).cloned()
    }

    pub fn list_keys(&self) -> Vec<String> {
        let s = self.inner.lock().unwrap();
        s.map.keys().cloned().collect()
    }
}

network.rs

use crate::storage::Storage;
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
                            _ => log::warn!("Empty read or error from {}", peer),
                        }
                    });
                }
                Err(e) => log::error!("Accept error: {}", e),
            }
        }
    }
}

node.rs

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
        task::spawn(async move { network.run().await }).await.ok();
    }
}

api.rs




