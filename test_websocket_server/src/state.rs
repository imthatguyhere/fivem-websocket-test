//! Shared application state for the WebSocket server

use std::sync::Arc; //=--
use tokio::sync::broadcast; //=--
use crate::config::Config; //=--
use axum::extract::ws::Utf8Bytes; //=-- Use Utf8Bytes for axum 0.8 Message::Text

/// Control commands broadcast to all connections
#[derive(Debug, Clone)]
pub enum ControlCommand { //=--
    DisconnectAll, //=-- Drop all clients
}

/// Shared state passed to Axum handlers
pub struct AppState { //=--
    pub config: Arc<Config>, //=--
    pub tx: broadcast::Sender<Utf8Bytes>, //=-- Broadcast channel for server-originated messages
    pub ctrl_tx: broadcast::Sender<ControlCommand>, //=-- Control channel (disconnects, etc.)
}
