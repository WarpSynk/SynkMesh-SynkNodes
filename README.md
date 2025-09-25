# SynkNodes

SynkNodes is a Rust-based node implementation for the WarpSynk Protocol.  
It provides peer-to-peer networking, local storage, cryptographic functions, and an HTTP API for external apps.  

## Features

- Node identity generated with SHA256 + UUID  
- Persistent local storage in JSON files  
- TCP networking for peer-to-peer communication  
- HTTP API for external apps to query and store values  
- Configurable via environment variables  

## Directory Structure

synknodes/ Cargo.toml src/ main.rs        // Entry point node.rs        // Node logic config.rs      // Environment-based config crypto.rs      // Hashing and IDs storage.rs     // Persistent key-value storage network.rs     // TCP communication api.rs         // HTTP API utils.rs       // Logging helpers
## Installation

Make sure [Rust](https://www.rust-lang.org/tools/install) is installed.

```bash
git clone https://github.com/yourusername/synknodes.git
cd synknodes
cargo build --release

## Run
cargo run

## Or with environment variables:

SYNK_NODE_ID="synk-001" \
SYNK_TCP_PORT=7000 \
SYNK_HTTP_PORT=8080 \
SYNK_DATA_DIR="./data" \
SYNK_PEERS="127.0.0.1:7001,127.0.0.1:7002" \
cargo run

## API Examples

GET /status → Node ID, ports, stored keys

GET /data/{key} → Retrieve value for a key

POST /store → Store key-value pair


{
  "key": "username",
  "value": "alice"
}

## TCP Protocol

PUT key value → Stores a value

GET key → Retrieves a value

echo "PUT test 123" | nc 127.0.0.1 7000
echo "GET test" | nc 127.0.0.1 7000

## Example Code Snippets

main.rs

#[tokio::main]
async fn main() {
    let cfg = config::Config::from_env();
    let storage = storage::Storage::new(&cfg.data_dir).unwrap();
    let node = Arc::new(node::SynkNode::new(cfg, storage));

    node.clone().run_network().await;
    api::run_api(node).await;
}

storage.rs

pub fn set(&self, key: &str, value: &str) -> anyhow::Result<()> {
    let mut s = self.inner.lock().unwrap();
    s.map.insert(key.to_string(), value.to_string());
    Ok(())
}

crypto.rs

pub fn generate_node_id() -> String {
    let hash = sha2::Sha256::digest(rand::random::<u128>().to_le_bytes());
    hex::encode(hash)
}

## Roadmap

TLS for secure communication

Peer discovery and sync

Expand API functionality


## License

MIT
