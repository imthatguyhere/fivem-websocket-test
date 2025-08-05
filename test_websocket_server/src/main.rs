//! Main entry point for the WebSocket heartbeat server.

mod config;
mod websocket;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::config::Config;
use tracing_subscriber;
use std::error::Error;

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

  //=-- Define the router with WebSocket route and shared state
  let app = Router::new()
    .route("/", get(websocket::handler))
    .with_state(shared.clone());

  let addr = SocketAddr::new(
    shared.bind_ip.parse().map_err(|_| "Invalid bind_ip in config.toml")?,
    shared.bind_port,
  );

  //=-- Start the server
  let listener = TcpListener::bind(addr).await
    .map_err(|e| {
      tracing::error!("Failed to bind to address: {}", e);
      e
    })?;
  
  tracing::info!("📡 Listening on ws://{}", addr);
  
  axum::serve(listener, app)
    .await
    .map_err(|e| {
      tracing::error!("Failed to start server: {}", e);
      e
    })?;
  
  Ok(())
}
