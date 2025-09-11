//! Module for loading application configuration from `config.toml`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;

/// Configuration loaded from `config.toml`
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Interval in seconds between heartbeat messages
    pub heartbeat_interval_secs: u64,
    /// IP address the server will bind to
    pub bind_ip: String,
    /// Port the server will bind to
    pub bind_port: u16,
    /// Toggle fancy ANSI-styled console help output
    #[serde(default = "default_fancy_help")]
    pub fancy_help: bool,
    /// Seconds to cache the generated help text before regenerating
    #[serde(default = "default_help_cache_secs")]
    pub help_cache_secs: u64,
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

/// Default for `fancy_help` so older config files keep working
fn default_fancy_help() -> bool { //=--
    true
}
/// Default cache TTL for help text generation (in seconds)
fn default_help_cache_secs() -> u64 { //=--
    60
}
