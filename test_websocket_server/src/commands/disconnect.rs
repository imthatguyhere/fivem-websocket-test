//! Console command: disconnect/dc

use crate::commands::CommandRegistry; //=-- Access registry

/// Register the disconnect command (aliases: "disconnect", "dc")
pub fn register(reg: &mut CommandRegistry) {
    reg.register(&["disconnect", "dc"], "Disconnect all connected WebSocket clients", move |ctx, _| {
        let _ = ctx.ctrl_tx.send(crate::state::ControlCommand::DisconnectAll);
        tracing::info!("🔌 Disconnecting all clients on command");
    });
}
