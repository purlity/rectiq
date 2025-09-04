#![forbid(unsafe_code)]
use super::{KeyStore, KeyStoreError};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct MemoryKeyStore {
    map: Mutex<HashMap<String, String>>,
}

impl KeyStore for MemoryKeyStore {
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
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), KeyStoreError> {
        self.map.lock().unwrap().remove(key);
        Ok(())
    }
}
