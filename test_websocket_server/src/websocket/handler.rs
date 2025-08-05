//! Handles WebSocket message and heartbeat logic.

use axum::extract::ws::{Message, WebSocket};
use chrono::Local;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use crate::config::Config;
use tracing;

/// Handle a WebSocket connection with heartbeat and incoming message logging
///
/// - Sends heartbeat every `heartbeat_interval_secs`
/// - Logs any incoming messages to stdout
pub async fn handle_socket(socket: WebSocket, config: Arc<Config>) {
    let interval_secs = config.heartbeat_interval_secs;
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    let hb_sender = sender.clone();

    //=-- Spawn the heartbeat task
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_secs));
        loop {
            ticker.tick().await;
            let timestamp = Local::now();
            let msg = format!("💓 Heartbeat Sent @ {}", timestamp.format("%Y-%m-%d--%H-%M-%S"));
            tracing::info!("{}", msg);
            let mut guard = hb_sender.lock().await;
            if guard.send(Message::Text(msg.into())).await.is_err() {
                tracing::warn!("❌ Client disconnected during heartbeat");
                break;
            }
        }
    });

    //=-- Receive and print incoming messages from client
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                tracing::info!(
                    "📥 Received @ {}: {}",
                    Local::now().format("%Y-%m-%d--%H-%M-%S"),
                    text
                );
            }
            Message::Binary(_) => {
                tracing::info!(
                    "📥 Received binary @ {}",
                    Local::now().format("%Y-%m-%d--%H-%M-%S")
                );
            }
            Message::Close(_) => {
                tracing::info!("👋 Client disconnected");
                break;
            }
            _ => {}
        }
    }

    tracing::info!("💀 WebSocket connection closed");
}
