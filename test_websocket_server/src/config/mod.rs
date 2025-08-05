//! Module for loading application configuration from `config.toml`.

use serde::Deserialize;
use std::fs;
use std::error::Error;

/// Configuration loaded from `config.toml`
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Interval in seconds between heartbeat messages
    pub heartbeat_interval_secs: u64,
    /// IP address the server will bind to
    pub bind_ip: String,
    /// Port the server will bind to
    pub bind_port: u16,
}

impl Config {
    /// Load and parse the configuration file
    ///
    /// # Arguments
    /// * `path` - Path to the `config.toml` file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
