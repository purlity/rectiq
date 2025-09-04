#![forbid(unsafe_code)]
use super::{KeyStore, KeyStoreError};
use secret_service::blocking::SecretService;
use secret_service::EncryptionType;
use std::collections::HashMap;

pub struct SecretSvc {
    ss: SecretService<'static>,
}

impl SecretSvc {
    /// Create a new secret service client.
    ///
    /// # Errors
    /// Returns an error if the `DBus` connection or collection retrieval fails.
    pub fn new() -> Result<Self, KeyStoreError> {
        let ss = SecretService::connect(EncryptionType::Dh)
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        Ok(Self { ss })
    }
}

impl KeyStore for SecretSvc {
    fn get(&self, account: &str) -> Result<String, KeyStoreError> {
        let mut attrs = HashMap::new();
        attrs.insert("service", "rectiq");
        attrs.insert("account", account);
        let col = self
            .ss
            .get_default_collection()
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        let items = col
            .search_items(attrs)
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        let item = items.into_iter().next().ok_or(KeyStoreError::NotFound)?;
        let secret = item
            .get_secret()
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        String::from_utf8(secret).map_err(|e| KeyStoreError::Other(e.to_string()))
    }

    fn set(&self, account: &str, value: &str) -> Result<(), KeyStoreError> {
        let mut attrs = HashMap::new();
        attrs.insert("service", "rectiq");
        attrs.insert("account", account);
        let col = self
            .ss
            .get_default_collection()
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        col.create_item(
            "Rectiq API Key",
            attrs,
            value.as_bytes(),
            true,
            "text/plain",
        )
        .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<(), KeyStoreError> {
        let mut attrs = HashMap::new();
        attrs.insert("service", "rectiq");
        attrs.insert("account", account);
        let col = self
            .ss
            .get_default_collection()
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        let items = col
            .search_items(attrs)
            .map_err(|e| KeyStoreError::Backend(e.to_string()))?;
        for it in items {
            let _ = it.delete();
        }
        Ok(())
    }
}
