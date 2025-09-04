// Helper function to sort JSON keys canonically
pub fn sort_json_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(mut map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                if let Some(v) = map.remove(&key) {
                    sorted.insert(key, sort_json_keys(v));
                }
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(vec) => {
            serde_json::Value::Array(vec.into_iter().map(sort_json_keys).collect())
        }
        other => other,
    }
}
