//! Main entry point for the WebSocket heartbeat server.

mod config;
mod websocket;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::config::Config;
use tracing_subscriber;

/// Application entry point
#[tokio::main]
async fn main() {
  //=-- Initialize tracing
  tracing_subscriber::fmt::init();
  
  //=-- Load config from disk
  let config: Config = match Config::load("config.toml") {
    Ok(config) => config,
    Err(e) => {
      tracing::error!("❌ Failed to load config.toml: {}", e);
      std::process::exit(1); //=-- Exit on failure to load the config file
    }
  };
  let shared = Arc::new(config);

  //=-- Define the router with WebSocket route and shared state
  let app = Router::new()
    .route("/", get(websocket::handler))
    .with_state(shared.clone());

  let addr = SocketAddr::new(
    shared.bind_ip.parse().expect("Invalid bind_ip in config.toml"),
    shared.bind_port,
  );

  //=-- Start the server
  let listener = TcpListener::bind(addr).await
    .expect("Failed to bind to address");
  
  tracing::info!("📡 Listening on ws://{}", addr);
  
  axum::serve(listener, app)
    .await
    .expect("Failed to start server");
}
