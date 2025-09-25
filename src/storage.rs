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