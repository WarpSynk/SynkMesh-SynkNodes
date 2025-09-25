use crate::config::Config;
use crate::storage::Storage;
use crate::network::Network;
use crate::api;
use crate::error::Result;
use tracing::info;

pub struct SynkNode {
    config: Config,
    storage: Storage,
}

impl SynkNode {
    pub fn new(config: Config, storage: Storage) -> Self {
        Self { config, storage }
    }
    
    pub async fn run(self) -> Result<()> {
        info!("Starting SynkNode {}", self.config.node_id);
        
        // Start TCP network server
        let network = Network::new(self.storage.clone(), self.config.tcp_port);
        let network_handle = tokio::spawn(async move {
            network.run().await
        });
        
        // Start HTTP API server
        let api_handle = tokio::spawn(api::run_api(
            self.storage,
            self.config.http_port,
            self.config.node_id,
            self.config.tcp_port,
        ));
        
        // Wait for either server to exit
        tokio::select! {
            result = network_handle => {
                if let Err(e) = result {
                    eprintln!("Network error: {}", e);
                }
            }
            result = api_handle => {
                if let Err(e) = result {
                    eprintln!("API error: {}", e);
                }
            }
        }
        
        Ok(())
    }
}