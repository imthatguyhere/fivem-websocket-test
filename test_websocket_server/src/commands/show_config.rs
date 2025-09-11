//! Console command: config/cfg

use crate::commands::CommandRegistry; //=-- Access registry
use std::sync::Arc; //=--
use crate::config::Config; //=--

/// Register the config command (aliases: "config", "cfg")
pub fn register(reg: &mut CommandRegistry, cfg: Arc<Config>) { //=--
    reg.register(&["config", "cfg"], "Show current server configuration", move |_ctx, _| {
        tracing::info!(
            "\nConfiguration:\n  bind_ip: {}\n  bind_port: {}\n  heartbeat_interval_secs: {}\n  fancy_help: {}\n  help_cache_secs: {}",
            cfg.bind_ip,
            cfg.bind_port,
            cfg.heartbeat_interval_secs,
            cfg.fancy_help,
            cfg.help_cache_secs,
        );
    });
}
