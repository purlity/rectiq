#![forbid(unsafe_code)]

use zeroize::Zeroize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("key not found")]
    NotFound,
    #[error("backend unavailable: {0}")]
    Backend(String),
    #[error("io: {0}")]
    Io(String),
    #[error("other: {0}")]
    Other(String),
}

pub trait KeyStore: Send + Sync + 'static {
    /// Retrieve a secret value by key.
    ///
    /// # Errors
    /// Returns an error if the key is not found or the backend is unavailable.
    fn get(&self, key: &str) -> Result<String, KeyStoreError>;

    /// Store a secret value by key.
    ///
    /// # Errors
    /// Returns an error if the backend is unavailable or the write fails.
    fn set(&self, key: &str, value: &str) -> Result<(), KeyStoreError>;

    /// Delete a secret by key.
    ///
    /// # Errors
    /// Returns an error if the key cannot be deleted or the backend is unavailable.
    fn delete(&self, key: &str) -> Result<(), KeyStoreError>;
}

pub mod memory;
pub mod file;
#[cfg(all(feature = "keystore-native", target_os = "macos"))]
pub mod macos;
#[cfg(all(feature = "keystore-native", target_os = "linux"))]
pub mod linux;
#[cfg(all(feature = "keystore-native", target_os = "windows"))]
pub mod windows;

pub enum KeyStoreKind {
    Memory,
    File,
    #[cfg(feature = "keystore-native")]
    Native,
}

#[must_use]
pub fn default_store() -> Box<dyn KeyStore> {
    if std::env::var("RECTIQ_INSECURE_ALLOW_FILE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Box::new(file::FileKeyStore::default());
    }
    #[cfg(all(feature = "keystore-native", target_os = "macos"))]
    {
        Box::new(macos::MacKeychain::new())
    }
    #[cfg(all(feature = "keystore-native", target_os = "linux"))]
    {
        match linux::SecretSvc::new() {
            Ok(svc) => Box::new(svc),
            Err(e) => {
                tracing::warn!("libsecret unavailable: {e}");
                Box::new(memory::MemoryKeyStore::default())
            }
        }
    }
    #[cfg(all(feature = "keystore-native", target_os = "windows"))]
    {
        Box::new(windows::WinCred::new())
    }
    // Fallback when native keystore feature is disabled or unsupported OS
    #[cfg(not(all(
        feature = "keystore-native",
        any(target_os = "macos", target_os = "linux", target_os = "windows")
    )))]
    {
        Box::new(memory::MemoryKeyStore::default())
    }
}

pub fn wipe_secret(mut s: String) {
    s.zeroize();
}

#[must_use]
pub fn key_id(env: &str, who: &str, purpose: &str) -> String {
    format!("rectiq:{env}:{who}:{purpose}")
}
