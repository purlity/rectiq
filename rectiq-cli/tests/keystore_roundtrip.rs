use rectiq_cli::keystore::{self, KeyStore, KeyStoreError};

fn roundtrip(store: &dyn KeyStore) {
    // Use a unique key per run to avoid colliding with any pre-existing entries
    let key = format!("rectiq_cli_test_key_{}", uuid::Uuid::new_v4());

    // Ensure clean state; delete is idempotent. If backend is unavailable (e.g., headless keychain), skip.
    let _ = store.delete(&key);
    if let Err(KeyStoreError::Backend(e)) = store.get(&key) {
        eprintln!("skipping native keystore test: backend unavailable: {e}");
        return;
    }

    // Set value
    let val = "SECRET_VALUE";
    store.set(&key, val).expect("set ok");
    let got = store.get(&key).expect("get ok");
    assert_eq!(got, val);

    // Delete and ensure gone (or backend unavailable)
    store.delete(&key).expect("delete ok");
    match store.get(&key) {
        Err(KeyStoreError::NotFound) => {}
        Err(KeyStoreError::Backend(e)) => {
            eprintln!("skipping native keystore trailing check: backend unavailable: {e}");
        }
        other => panic!("unexpected keystore result after delete: {other:?}"),
    }
}

#[test]
fn keystore_memory_roundtrip() {
    let store = keystore::memory::MemoryKeyStore::default();
    roundtrip(&store);
}

#[cfg(feature = "keystore-native")]
#[test]
fn keystore_native_roundtrip() {
    let store = keystore::default_store();
    roundtrip(store.as_ref());
}
