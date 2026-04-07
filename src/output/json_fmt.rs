use serde::Serialize;

/// Serialize any serializable value to pretty JSON.
pub fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}
