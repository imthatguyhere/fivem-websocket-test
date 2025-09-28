//! WebSocket server 

mod handler;

use axum::{
  extract::ws::WebSocketUpgrade,
  extract::State,
  response::IntoResponse,
};
use std::sync::Arc;
use crate::state::AppState; //=-- Use AppState to access config and broadcaster

/// Axum route handler for WebSocket upgrade
pub async fn handler(
  ws: WebSocketUpgrade,
  State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
  //=-- Upgrade the request to a WebSocket connection
  ws.on_upgrade(move |socket| handler::handle_socket(socket, state))
}
