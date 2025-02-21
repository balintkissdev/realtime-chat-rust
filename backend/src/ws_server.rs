//! WebSocket component for listening and routing real-time chat messages from
//! WebSocket clients.
//!
//! Messages sent by a client is broadcasted to all other clients currently connected to the chat.
//! Client is removed from the chat on disconnect.

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::{ChatClient, Payload, PayloadEventType, ServerState, SharedServerState, Tx};

/// Entry for starting WebSocket server to manage chat operations.
pub async fn run_ws_server(listener: TcpListener, server_state: SharedServerState) {
    while let Ok((tcp_stream, client_address)) = listener.accept().await {
        tokio::spawn(client_handler(
            tcp_stream,
            client_address,
            server_state.clone(),
        ));
    }
}

/// Task for accepting client, message broadcasting and disconnect when finished.
async fn client_handler(
    tcp_stream: TcpStream,
    client_address: SocketAddr,
    server_state: SharedServerState,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("websocket handshake error");
    let (tx, mut rx) = mpsc::unbounded_channel();
    log::trace!("received new client connection");

    // Duplex stream, use it as reader/writer
    let (mut ws_writer, ws_reader) = ws_stream.split();

    // Forward messages coming from current connected single client to all other clients
    let send_broadcast = ws_reader.try_for_each(|msg| {
        let server_state = server_state.clone();
        let tx = tx.clone();
        async move {
            // Skip Ping, Pong and Close messages
            if msg.is_text() {
                log::trace!("received message {:?}", msg.to_text().unwrap());

                let mut server_state = server_state.lock().await;

                // User connecting for the first time
                let payload: Payload = serde_json::from_str(&msg.to_string()).unwrap();
                if payload.event_type == PayloadEventType::Connected {
                    if !server_state.clients.contains_key(&client_address) {
                        match add_client(
                            &mut server_state,
                            client_address,
                            tx.clone(),
                            &msg.to_text().unwrap(),
                        )
                        .await
                        {
                            Ok(()) => (),
                            Err(e) => {
                                log::error!("user add error: {e}");
                                return Err(
                                    tokio_tungstenite::tungstenite::Error::ConnectionClosed,
                                );
                            }
                        }
                    }
                }

                broadcast(&mut server_state, &msg.to_string(), Some(client_address)).await;
            }
            Ok(())
        }
    });

    // Receive message broadcasted by others
    let receive_broadcast = async move {
        while let Some(msg) = rx.recv().await {
            ws_writer.send(msg.into()).await?;
        }
        Ok::<(), tokio_tungstenite::tungstenite::Error>(())
    };

    // Use tokio::select!() instead of tokio::try_join!() to avoid deadlock. select!() waits for
    // either sender or receiver task to finish. try_for_each() finishes when client disconnects and server receives a
    // "Close" WebSocket message.
    tokio::select! {
        _ = send_broadcast => {},
        _ = receive_broadcast => {},
    }

    remove_client(server_state.clone(), client_address).await;
}

async fn add_client(
    server_state: &mut ServerState,
    client_address: SocketAddr,
    tx: Tx,
    msg: &str,
) -> Result<(), String> {
    let payload: Payload = serde_json::from_str(msg).unwrap();
    if payload.event_type == PayloadEventType::Connected {
        let username = payload.username;
        if server_state
            .clients
            .iter()
            .find(|(_addr, client)| client.username == username)
            .is_some()
        {
            return Err(format!("user already exists: {username}"));
        }

        server_state.clients.insert(
            client_address,
            ChatClient {
                username: username.clone(),
                tx,
            },
        );
    }

    Ok(())
}

/// Send out message to multiple users in the chat. `sender` is excluded from
/// the list of message recipients. If `sender` is not specified, all members of the
/// chat receive the message and is treated as a server status message.
async fn broadcast(server_state: &mut ServerState, msg: &String, sender: Option<SocketAddr>) {
    let broadcast_recipients = server_state
        .clients
        .iter()
        .filter(|(addr, _client)| {
            // Exclude message sender from broadcast
            sender.map_or(true, |sender_addr| addr != &&sender_addr)
        })
        .map(|(_, ws_sink)| ws_sink);
    for broadcast_user in broadcast_recipients {
        broadcast_user
            .tx
            .send(msg.clone())
            .expect("unable to broadcast message");
        log::trace!("sent {:?} to {}", msg, broadcast_user.username);
    }

    // Save message to history
    server_state
        .history
        .push(serde_json::from_str(msg).unwrap());
}

/// Remove user from the list of users. Notifies remaining members in the chat about
/// the disconnected users.
async fn remove_client(server_state: SharedServerState, disconnected_client_address: SocketAddr) {
    let mut server_state = server_state.lock().await;
    let Some(disconnected_client) = server_state.clients.get(&disconnected_client_address) else {
        return;
    };
    let username = disconnected_client.username.clone();

    // Update client list
    server_state.clients.remove(&disconnected_client_address);
    log::trace!("user {:?} left the chat", username);

    // Notify remaining chat members
    let payload = Payload {
        event_type: PayloadEventType::Disconnected,
        username: username.clone(),
        message: None,
    };
    let j = serde_json::to_string(&payload).unwrap();
    broadcast(&mut server_state, &j, None).await;
}
