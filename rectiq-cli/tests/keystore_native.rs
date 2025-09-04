#![cfg(feature = "keystore-native")]
use rectiq_cli::keystore::{self, key_id};

#[test]
fn native_roundtrip_namespaced() {
    let store = keystore::default_store();
    let id = key_id("test", "roundtrip", "api_key");
    let _ = store.delete(&id);
    if let Err(rectiq_cli::keystore::KeyStoreError::Backend(e)) = store.set(&id, "X") {
        eprintln!("skipping native keystore test: backend unavailable: {e}");
        return;
    }
    assert!(store.get(&id).is_ok());
    match store.delete(&id) {
        Ok(()) => {}
        Err(rectiq_cli::keystore::KeyStoreError::Backend(e)) => {
            eprintln!("skipping native keystore cleanup: backend unavailable: {e}");
        }
        Err(e) => panic!("delete failed: {e}"),
    }
}
