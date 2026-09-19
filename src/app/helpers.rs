use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    format!("{}-{}", prefix, millis)
}
