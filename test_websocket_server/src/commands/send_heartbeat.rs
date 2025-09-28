//! Console command: heartbeat/hb

use crate::commands::CommandRegistry; //=-- Access registry
use serde_json::json; //=-- Build JSON payload

/// Register the heartbeat command (aliases: "heartbeat", "hb")
pub fn register(reg: &mut CommandRegistry) {
    reg.register(&["heartbeat", "hb"], "Send a heartbeat JSON payload to all clients", move |ctx, _| {
        let payload = json!({
            "type": "heartbeat",
            "data": "manual",
        }).to_string();
        let _ = ctx.tx.send(payload.clone().into());
        tracing::info!("📤 Broadcast console heartbeat JSON: {}", payload);
    });
}
