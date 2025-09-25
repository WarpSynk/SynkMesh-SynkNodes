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

    // graceful shutdown on Ctrl+C
    let _ = signal::ctrl_c().await;
    log::info!("Shutdown requested, stopping node");

    // Allow tasks to finish
    let _ = network_handle.abort();
    let _ = api_handle.abort();
}