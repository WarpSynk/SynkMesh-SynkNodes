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
git clone https://github.com/warpsynk/synknodes.git
cd synknodes
cargo build --release

# Run with default settings
cargo run

# Run with custom configuration
SYNK_NODE_ID="node-001" \
SYNK_TCP_PORT=7000 \
SYNK_HTTP_PORT=8080 \
SYNK_DATA_DIR="./data" \
SYNK_PEERS="127.0.0.1:7001" \
cargo run
