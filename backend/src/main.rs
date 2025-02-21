//! Server application entrypoint that acts as logger setup, REST API and WebSocket listener
//! startup and CTRL+C interrupt handling.

use chat_backend::{configuration, rest_server, ws_server, SharedServerState};
use env_logger::Env;

#[tokio::main]
async fn main() {
    // TODO: Switch to `tracing` crate and ecosystem once project becomes larger in scope
    let log_env = Env::default().default_filter_or("chat_backend=trace");
    env_logger::Builder::from_env(log_env).init();

    let config = configuration::get_config().expect("failed to read configuration");
    let server_state = SharedServerState::default();

    let rest_address = format!("{}:{}", config.host, config.backend.rest_port);
    let rest_listener =
        std::net::TcpListener::bind(&rest_address).expect("unable to bind REST API port");
    let rest_task = tokio::spawn(rest_server::run_rest_server(
        rest_listener,
        server_state.clone(),
    ));

    let ws_address = format!("{}:{}", config.host, config.backend.ws_port);
    let ws_listener = tokio::net::TcpListener::bind(&ws_address)
        .await
        .expect("unable to bind WebSocket port");
    let ws_task = tokio::spawn(ws_server::run_ws_server(ws_listener, server_state.clone()));

    log::info!("real-time chat server backend is functional");
    log::info!("REST API listener is on {}", &rest_address);
    log::info!("WebSocket listener is on {}", &ws_address);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            log::info!("CTRL+C received, initiating graceful shutdown...");
        }
        res = async { tokio::try_join!(rest_task, ws_task) } => {
            if let Err(e) = res {
                log::error!("abnormal server shutdown: {e}");
            }
        }
    }
}
