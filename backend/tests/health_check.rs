use chat_backend::{rest_server, SharedServerState};

#[tokio::test]
async fn health_check_works() {
    let server_state = SharedServerState::default();
    let address = "127.0.0.1";
    let rest_listener =
        std::net::TcpListener::bind(format!("{address}:0")).expect("unable to bind REST API port");
    let port = rest_listener.local_addr().unwrap().port();
    let _ = tokio::spawn(rest_server::run_rest_server(
        rest_listener,
        server_state.clone(),
    ));
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}:{}/health", address, port))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
