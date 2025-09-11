//! Console command: config/cfg

use crate::commands::CommandRegistry; //=-- Access registry
use std::sync::Arc; //=--
use crate::config::Config; //=--

/// Register the config command (aliases: "config", "cfg")
pub fn register(reg: &mut CommandRegistry, cfg: Arc<Config>) { //=--
    reg.register(&["config", "cfg"], "Show current server configuration", move |_ctx, _| {
        //=-- Serialize the config to JSON so we can print KVPs dynamically
        match serde_json::to_value(&*cfg) {
            Ok(serde_json::Value::Object(map)) => {
                //=-- Sort keys for stable output
                let mut entries: Vec<(String, serde_json::Value)> = map.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let mut out = String::from("\nConfiguration:");
                for (k, v) in entries {
                    let v_str = match v {
                        serde_json::Value::String(s) => s,
                        _ => v.to_string(),
                    };
                    out.push_str(&format!("\n  {}: {}", k, v_str));
                }
                tracing::info!("{}", out);
            }
            Ok(other) => {
                tracing::info!("\nConfiguration (raw): {}", other);
            }
            Err(e) => {
                tracing::error!("❌ Failed to serialize configuration: {}", e);
            }
        }
    });
}
