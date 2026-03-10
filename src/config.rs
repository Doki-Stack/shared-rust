use crate::error::{Error, Result};

/// Read a required environment variable, returning an error if not set.
pub fn require_env(key: &str) -> Result<String> {
    std::env::var(key)
        .map_err(|_| Error::internal(format!("Missing required environment variable: {key}")))
}

/// Read an optional environment variable with a default value.
pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
