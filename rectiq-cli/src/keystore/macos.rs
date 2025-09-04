#![forbid(unsafe_code)]
use super::{KeyStore, KeyStoreError};
use security_framework::passwords::{
    get_generic_password, set_generic_password, delete_generic_password,
};

const SERVICE: &str = "rectiq";

pub struct MacKeychain;

impl Default for MacKeychain {
    fn default() -> Self {
        Self::new()
    }
}

impl MacKeychain {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl KeyStore for MacKeychain {
    fn get(&self, key: &str) -> Result<String, KeyStoreError> {
        match get_generic_password(SERVICE, key) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|e| KeyStoreError::Other(e.to_string())),
            Err(e) => Err(KeyStoreError::Backend(e.to_string())),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeyStoreError> {
        set_generic_password(SERVICE, key, value.as_bytes())
            .map_err(|e| KeyStoreError::Backend(e.to_string()))
    }

    fn delete(&self, key: &str) -> Result<(), KeyStoreError> {
        delete_generic_password(SERVICE, key).map_err(|e| KeyStoreError::Backend(e.to_string()))
    }
}
