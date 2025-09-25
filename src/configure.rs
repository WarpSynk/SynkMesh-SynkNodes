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