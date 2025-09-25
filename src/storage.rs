use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::{Result, SynkError};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<RwLock<HashMap<String, String>>>,
    file_path: String,
}

impl Storage {
    pub fn new(data_dir: &str) -> Result<Self> {
        let file_path = format!("{}/storage.json", data_dir);
        
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(data_dir)
            .await
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        let storage = Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file_path,
        };
        
        storage.load_from_disk().await?;
        Ok(storage)
    }
    
    async fn load_from_disk(&self) -> Result<()> {
        if !Path::new(&self.file_path).exists() {
            return Ok(());
        }
        
        let mut file = File::open(&self.file_path)
            .await
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        if !contents.is_empty() {
            let data: HashMap<String, String> = serde_json::from_str(&contents)
                .map_err(SynkError::Serialization)?;
                
            let mut storage = self.inner.write()
                .map_err(|e| SynkError::Storage(e.to_string()))?;
            *storage = data;
        }
        
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<()> {
        let storage = self.inner.read()
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        let json = serde_json::to_string(&*storage)
            .map_err(SynkError::Serialization)?;
            
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .await
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        file.write_all(json.as_bytes())
            .await
            .map_err(|e| SynkError::Storage(e.to_string()))?;
            
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        let storage = self.inner.read().ok()?;
        storage.get(key).cloned()
    }
    
    pub async fn set(&self, key: String, value: String) -> Result<()> {
        {
            let mut storage = self.inner.write()
                .map_err(|e| SynkError::Storage(e.to_string()))?;
            storage.insert(key.clone(), value.clone());
        }
        
        self.save_to_disk().await?;
        Ok(())
    }
    
    pub fn keys(&self) -> Vec<String> {
        let storage = self.inner.read().unwrap();
        storage.keys().cloned().collect()
    }
    
    pub fn len(&self) -> usize {
        let storage = self.inner.read().unwrap();
        storage.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let storage = Storage::new(temp_dir.path().to_str().unwrap()).await.unwrap();
        
        storage.set("key1".to_string(), "value1".to_string()).await.unwrap();
        assert_eq!(storage.get("key1"), Some("value1".to_string()));
        assert_eq!(storage.len(), 1);
    }
}