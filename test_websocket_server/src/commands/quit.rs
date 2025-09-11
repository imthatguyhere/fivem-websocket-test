//! Console command: quit/exit/q

use crate::commands::CommandRegistry; //=-- Access registry

/// Register the quit command (aliases: "quit", "exit", "q")
pub fn register(reg: &mut CommandRegistry) { //=--
    reg.register(&["quit", "exit", "q"], "Disconnect all clients and gracefully shut down the server", move |ctx, _| {
        let _ = ctx.ctrl_tx.send(crate::state::ControlCommand::DisconnectAll);
        tracing::info!("👋 Quitting server on command");
        ctx.shutdown.cancel();
    });
}
