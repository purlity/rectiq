#![forbid(unsafe_code)]
use super::{KeyStore, KeyStoreError};
use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB};
use windows::Win32::Foundation::PWSTR;

pub struct WinCred;
impl WinCred {
    pub fn new() -> Self {
        Self
    }
}

fn protect(bytes: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut _,
    };
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        if !CryptProtectData(&mut in_blob, None, None, None, None, 0, &mut out).as_bool() {
            return Err(KeyStoreError::Backend("CryptProtectData".into()));
        }
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        Ok(v)
    }
}
fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut _,
    };
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        if !CryptUnprotectData(
            &mut in_blob,
            Some(PWSTR::null()),
            None,
            None,
            None,
            0,
            &mut out,
        )
        .as_bool()
        {
            return Err(KeyStoreError::Backend("CryptUnprotectData".into()));
        }
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        Ok(v)
    }
}

impl KeyStore for WinCred {
    fn get(&self, account: &str) -> Result<String, KeyStoreError> {
        let p = dirs::data_dir()
            .ok_or_else(|| KeyStoreError::Io("no data dir".into()))?
            .join("Rectiq/keys")
            .join(format!("{account}.bin"));
        let enc = std::fs::read(p).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        let dec = unprotect(&enc)?;
        String::from_utf8(dec).map_err(|e| KeyStoreError::Other(e.to_string()))
    }
    fn set(&self, account: &str, value: &str) -> Result<(), KeyStoreError> {
        let enc = protect(value.as_bytes())?;
        let dir = dirs::data_dir()
            .ok_or_else(|| KeyStoreError::Io("no data dir".into()))?
            .join("Rectiq/keys");
        std::fs::create_dir_all(&dir).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        let p = dir.join(format!("{account}.bin"));
        std::fs::write(p, enc).map_err(|e| KeyStoreError::Io(e.to_string()))
    }
    fn delete(&self, account: &str) -> Result<(), KeyStoreError> {
        let p = dirs::data_dir()
            .ok_or_else(|| KeyStoreError::Io("no data dir".into()))?
            .join("Rectiq/keys")
            .join(format!("{account}.bin"));
        let _ = std::fs::remove_file(p);
        Ok(())
    }
}
