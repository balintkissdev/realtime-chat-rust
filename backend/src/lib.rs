//! Collection of POD (Plain Old Data) types shared by both REST API and WebSocket components.

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

pub mod configuration;
pub mod rest_server;
pub mod ws_server;

/// Message payload that is passed around on WebSocket as JSON string.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub event_type: PayloadEventType,
    pub username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadEventType {
    Connected,
    Disconnected,
    Message,
}

/// "Global" state of server application shared between REST API and WebSocket components.
#[derive(Debug, Default)]
pub struct ServerState {
    /// Flat store of all available clients for easy lookup during accepting client connections.
    pub clients: HashMap<SocketAddr, ChatClient>,

    /// List of messages and connection status event logs in chronological order, available for
    /// `GET /history` endpoint response.
    pub history: Vec<Payload>,
}
pub type SharedServerState = Arc<Mutex<ServerState>>;

#[derive(Debug)]
pub struct ChatClient {
    pub username: String,
    pub tx: Tx,
}

// Kept as String instead of Payload to avoid wasted repeated deserializations for each
// broadcast target.
pub type Tx = UnboundedSender<String>;
