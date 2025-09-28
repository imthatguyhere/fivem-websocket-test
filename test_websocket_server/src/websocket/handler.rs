//! Handles WebSocket message and heartbeat logic.

use axum::extract::ws::{Message, WebSocket};
use chrono::Local;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use crate::state::{AppState, ControlCommand}; //=-- Access config and control broadcast channel
use serde_json::json;
use tracing;
use tokio_util::sync::CancellationToken;

/// Handle a WebSocket connection with heartbeat and incoming message logging
///
/// - Sends heartbeat every `heartbeat_interval_secs`
/// - Logs any incoming messages to stdout
pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let interval_secs = state.config.heartbeat_interval_secs;
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    let hb_sender = sender.clone();
    let cancel_token = CancellationToken::new();
    let heartbeat_cancel = cancel_token.clone();
    let console_cancel = cancel_token.clone(); //=-- Cancel for console forwarder
    let ctrl_cancel = cancel_token.clone(); //=-- Cancel for control listener

    //=-- Subscribe to console broadcast channel
    let mut rx = state.tx.subscribe();
    let console_sender = sender.clone();
    let close_sender = sender.clone(); //=-- Sender used to close connection on control

    //=-- Spawn the heartbeat task
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_secs));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    //=-- Build JSON heartbeat payload
                    let payload = json!({
                        "type": "heartbeat",
                        "data": "request",
                    }).to_string();
                    tracing::info!("💓 Sending heartbeat JSON: {}", payload);
                    let mut guard = hb_sender.lock().await;
                    if guard.send(Message::Text(payload.into())).await.is_err() {
                        tracing::warn!("❌ Client disconnected during heartbeat");
                        break;
                    }
                }
                _ = heartbeat_cancel.cancelled() => {
                    tracing::info!("🛑 Heartbeat task cancelled");
                    break;
                }
            }
        }
    });

    //=-- Forward console-broadcast JSON to this client
    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(msg) => {
                            let mut guard = console_sender.lock().await;
                            if guard.send(Message::Text(msg.into())).await.is_err() {
                                tracing::warn!("❌ Client disconnected during console forward");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Broadcast receive error: {}", e);
                            break;
                        }
                    }
                }
                _ = console_cancel.cancelled() => {
                    tracing::info!("🛑 Console forwarder cancelled");
                    break;
                }
            }
        }
    });

    //=-- Listen for control commands and close connection on demand
    let mut ctrl_rx = state.ctrl_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                cmd = ctrl_rx.recv() => {
                    match cmd {
                        Ok(ControlCommand::DisconnectAll) => {
                            tracing::info!("🔌 Control: closing client connection");
                            let mut guard = close_sender.lock().await;
                            //=-- Best-effort send a Close frame, then close the sink
                            let _ = guard.send(Message::Close(None)).await;
                            let _ = guard.close().await;
                            ctrl_cancel.cancel(); //=-- Cancel other tasks for this connection
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Control channel receive error: {}", e);
                            break;
                        }
                    }
                }
                _ = ctrl_cancel.cancelled() => {
                    tracing::info!("🛑 Control listener cancelled");
                    break;
                }
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

    // Cancel the heartbeat task when the connection closes
    cancel_token.cancel();
    tracing::info!("💀 WebSocket connection closed");
}
