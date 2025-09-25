use synknodes::{Config, Storage, SynkNode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up a basic node with in-memory storage
    let config = Config::from_env();
    let storage = Storage::new(&config.data_dir).await?;
    let node = SynkNode::new(config, storage);
    
    node.run().await?;
    Ok(())
}