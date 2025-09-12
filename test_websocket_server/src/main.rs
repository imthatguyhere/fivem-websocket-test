//! Main entry point for the WebSocket heartbeat server.

mod config;
mod websocket;
mod state;
mod commands;

use axum::{routing::get, Router};
use axum::extract::ws::Utf8Bytes; //=-- Use Utf8Bytes for axum 0.8 text frames
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use crate::config::Config;
use tracing_subscriber;
use std::error::Error;
use crate::state::{
  AppState,
  ControlCommand,
}; //=-- Shared app state and control commands
use crate::commands::{CommandRegistry, CommandContext}; //=-- Console command registry
use tokio::io::{self, AsyncBufReadExt, BufReader}; //=-- For async stdin JSON input
use tokio::sync::broadcast; //=-- Broadcast channel for manual sends
use tokio_util::sync::CancellationToken; //=-- Graceful shutdown token
use std::time::Duration; //=-- Help cache TTL

/// Helper to load dynamic payload commands and (re-)register help so it reflects the latest registry
fn load_dynamic_commands(reg: &mut CommandRegistry, fancy_help: bool, help_ttl: Duration) {
  //=-- Load dynamic payload commands from commands.toml (if present)
  crate::commands::dynamic_payload::register_from_file(reg, "commands.toml");
  //=-- Register help after dynamic commands so it captures the latest commands list
  crate::commands::help::register(reg, fancy_help, help_ttl);
}

/// Application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  //=-- Initialize tracing
  tracing_subscriber::fmt::init();
  
  //=-- Load config from disk
  let config: Config = Config::load("config.toml").map_err(|e| {
    tracing::error!("❌ Failed to load config.toml: {}", e);
    e  //=-- Propagate the original error
  })?;
  let shared = Arc::new(config);

  //=-- Decide whether to use fancy ANSI-styled help output
  let fancy_help = shared.fancy_help && std::env::var_os("NO_COLOR").is_none();
  let help_ttl = Duration::from_secs(shared.help_cache_secs); //=-- Cache TTL (seconds)

  //=-- Create broadcast channels and shutdown token
  let (tx, _rx) = broadcast::channel::<Utf8Bytes>(100);
  let (ctrl_tx, _ctrl_rx) = broadcast::channel::<ControlCommand>(16);
  let shutdown = CancellationToken::new();

  let app_state = Arc::new(AppState {
    config: shared.clone(),
    tx: tx.clone(),
    ctrl_tx: ctrl_tx.clone(),
  });

  //=-- Define the router with WebSocket route and shared state
  let app = Router::new()
    .route("/", get(websocket::handler))
    .with_state(app_state.clone());

  let addr = SocketAddr::new(
    shared.bind_ip.parse().map_err(|e| {
      tracing::error!("Invalid bind_ip in config.toml: {}", e);
      e
    })?,
    shared.bind_port,
  );

  //=-- Start the server
  let listener = TcpListener::bind(addr).await
    .map_err(|e| {
      tracing::error!("Failed to bind to address: {}", e);
      e
    })?;
  
  tracing::info!("📡 Listening on ws://{}", addr);
  tracing::info!("💬 Type a JSON payload and press Enter to broadcast to all clients");
  tracing::info!("❓ Type 'help' to see available console commands");

  //=-- Spawn a task to read JSON from stdin and broadcast to clients
  let stdin_shutdown = shutdown.clone();
  let fancy_help_on = fancy_help; //=-- capture for the stdin task
  let help_ttl_spawn = help_ttl; //=-- capture TTL
  let shared_cfg = shared.clone(); //=-- capture config for commands
  tokio::spawn(async move {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    //=-- Build console command registry (shared for help supplier)
    let commands = Arc::new(RwLock::new(CommandRegistry::new()));
    {
      let mut reg = commands.write().expect("commands lock poisoned");
      //=-- Register commands from submodules
      crate::commands::show_config::register(&mut reg, shared_cfg.clone());
      crate::commands::send_heartbeat::register(&mut reg);
      crate::commands::disconnect::register(&mut reg);
      crate::commands::quit::register(&mut reg);
      //=-- Register a reload command that triggers actual reload via context closure
      reg.register(&["reload", "rl"], "Reload dynamic payload commands from commands.toml", move |ctx, _| {
        (ctx.reload_fn)();
      });
      //=-- Load dynamic commands and help
      load_dynamic_commands(&mut reg, fancy_help_on, help_ttl_spawn);
    }
    //=-- Supplier for rendering help on demand (used by help handler's TTL cache)
    let help_supplier = {
      let commands = commands.clone();
      Arc::new(move |fancy: bool| {
        let reg = commands.read().expect("commands lock poisoned");
        reg.help_text_with_fancy(fancy)
      })
    };
    //=-- Closure to perform reload without holding locks during handler execution
    let reload_fn = {
      let commands = commands.clone();
      Arc::new(move || {
        tracing::info!("🔁 Reloading dynamic payload commands...");
        {
          let mut reg = commands.write().expect("commands lock poisoned");
          //=-- Reload dynamic commands and help
          load_dynamic_commands(&mut reg, fancy_help_on, help_ttl_spawn);
        }
        {
          let reg = commands.read().expect("commands lock poisoned");
          tracing::info!("🧩 Commands now: {}", reg.primary_names_distinct().join(", "));
        }
        tracing::info!("✅ Reload complete");
      })
    };
    let ctx = CommandContext { tx: tx.clone(), ctrl_tx: ctrl_tx.clone(), shutdown: stdin_shutdown.clone(), help_supplier, reload_fn };
    //=-- Log all primary command names loaded at boot
    {
      let reg = commands.read().expect("commands lock poisoned");
      tracing::info!("🧩 Commands loaded: {}", reg.primary_names_distinct().join(", "));
    }

    loop {
      line.clear();
      match reader.read_line(&mut line).await {
        Ok(0) => {
          //=-- EOF reached
          tracing::info!("🧵 Stdin closed - stopping console broadcaster");
          break;
        }
        Ok(_) => {
          let trimmed = line.trim();
          if trimmed.is_empty() { continue; }
          //=-- Try command registry first (fetch handler, then drop lock before executing)
          let handler_opt = {
            let reg = commands.read().expect("commands lock poisoned");
            reg.handler_for(trimmed)
          };
          if let Some(handler) = handler_opt {
            handler(&ctx, trimmed);
            if ctx.shutdown.is_cancelled() { break; }
            continue;
          }
          //=-- Validate JSON before broadcasting
          match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(_) => {
              //=-- Convert once to Utf8Bytes and broadcast
              if let Err(e) = tx.send(trimmed.to_owned().into()) {
                tracing::warn!("⚠️ Failed to broadcast console payload: {}", e);
              } else {
                tracing::info!("📤 Broadcast console JSON: {}", trimmed);
              }
            }
            Err(e) => {
              //=-- If it doesn't even look like JSON, tell the user the command doesn't exist
              match trimmed.chars().next() {
                Some('{') | Some('[') | Some('"') | Some('t') | Some('f') | Some('n') | Some('-') | Some('0'..='9') => {
                  tracing::warn!("⚠️ Invalid JSON - not sent: {} | error: {}", trimmed, e);
                }
                _ => {
                  tracing::warn!("⚠️ Command doesn't exist: {} — type 'help' for a list of commands", trimmed);
                }
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("❌ Error reading stdin: {}", e);
          break;
        }
      }
    }
  });
  
  axum::serve(listener, app)
    .with_graceful_shutdown(async move {
      shutdown.cancelled().await;
    })
    .await
    .map_err(|e| {
      tracing::error!("Failed to start server: {}", e);
      e
    })?;
  
  Ok(())
}
