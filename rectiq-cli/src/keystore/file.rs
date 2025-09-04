#![forbid(unsafe_code)]

use super::{KeyStore, KeyStoreError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

pub struct FileKeyStore {
    map: Mutex<HashMap<String, String>>,
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct DiskFormat {
    secrets: HashMap<String, String>,
}

impl Default for FileKeyStore {
    fn default() -> Self {
        let path = std::env::var("RECTIQ_INSECURE_FILE").map_or_else(
            |_| {
                dirs::config_dir().map_or_else(
                    || PathBuf::from("./.rectiq-secrets.json"),
                    |p| p.join("rectiq").join("secrets.json"),
                )
            },
            PathBuf::from,
        );

        let ks = Self {
            map: Mutex::new(HashMap::new()),
            path,
        };
        let _ = ks.load();
        ks
    }
}

impl FileKeyStore {
    fn load(&self) -> Result<(), KeyStoreError> {
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        match fs::read_to_string(&self.path) {
            Ok(s) => {
                let disk: DiskFormat =
                    serde_json::from_str(&s).map_err(|e| KeyStoreError::Io(e.to_string()))?;
                *self.map.lock().unwrap() = disk.secrets;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    fn persist(&self) -> Result<(), KeyStoreError> {
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let disk = DiskFormat {
            secrets: self.map.lock().unwrap().clone(),
        };
        let data =
            serde_json::to_vec_pretty(&disk).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        fs::write(&self.path, data).map_err(|e| KeyStoreError::Io(e.to_string()))
    }
}

impl KeyStore for FileKeyStore {
    fn get(&self, key: &str) -> Result<String, KeyStoreError> {
        self.map
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or(KeyStoreError::NotFound)
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeyStoreError> {
        self.map
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        self.persist()
    }

    fn delete(&self, key: &str) -> Result<(), KeyStoreError> {
        self.map.lock().unwrap().remove(key);
        self.persist()
    }
}
