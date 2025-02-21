use std::time::Duration;

use chat_backend::{ws_server, Payload, PayloadEventType, SharedServerState};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Receiver,
};
use tokio_tungstenite::{
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

static LOGGER: Lazy<()> = Lazy::new(|| {
    env_logger::init();
});
const HOST: &str = "127.0.0.1";
const TIMEOUT_SECONDS: Duration = tokio::time::Duration::from_secs(5);

struct TestClient {
    username: String,
    writer: Option<StreamWriter>,
    on_message_callback: Option<OnMessageCallback>,
}
type StreamWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type OnMessageCallback = Box<dyn Fn(String) + Send + 'static>;

impl TestClient {
    fn new(username: &str) -> Self {
        TestClient {
            username: username.into(),
            writer: None,
            on_message_callback: None,
        }
    }

    fn set_callback(&mut self, callback: OnMessageCallback) {
        self.on_message_callback = Some(callback);
    }

    async fn connect(&mut self, address: &str, port: u16) {
        let request = format!("ws://{}:{}", address, port)
            .into_client_request()
            .unwrap();
        let (stream, _) = tokio_tungstenite::connect_async(request.clone())
            .await
            .expect(&format!("{} failed to connect", self.username));
        let (writer, mut reader) = stream.split();
        self.writer = Some(writer);
        let callback = self.on_message_callback.take();
        let receiver = async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(msg) => {
                        if let Ok(text) = msg.into_text() {
                            if let Some(ref callback) = callback {
                                callback(text.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        panic!("error during receive: {e}");
                    }
                }
            }
        };
        tokio::spawn(receiver);

        let connect_payload = Payload {
            event_type: PayloadEventType::Connected,
            username: self.username.clone(),
            message: None,
        };
        self.send(&connect_payload).await;
    }

    async fn send(&mut self, payload: &Payload) {
        let j = serde_json::to_string(&payload).unwrap();
        self.writer
            .as_mut()
            .unwrap()
            .send(j.into())
            .await
            .expect(&format!(
                "unable to send {:?} from {}",
                payload, self.username,
            ));
    }

    async fn send_message(&mut self, msg: &str) {
        let payload = Payload {
            event_type: PayloadEventType::Message,
            username: self.username.clone(),
            message: Some(msg.into()),
        };
        self.send(&payload).await;
    }
}

async fn check_payload(rx: &mut Receiver<String>, expected: &Payload) {
    let msg = rx.recv().await.expect("unable to receive message");
    let actual: Payload = serde_json::from_str(&msg).expect("wrong message format");
    assert_eq!(actual, *expected);
}

// TODO: Minimize boilerplate in test code

#[tokio::test]
async fn single_user_gets_notified_on_new_user_join() {
    Lazy::force(&LOGGER);

    let listener = TcpListener::bind(format!("{HOST}:0"))
        .await
        .expect("unable to bind socket");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let server_state = SharedServerState::default();
        ws_server::run_ws_server(listener, server_state).await
    });

    let mut user1 = TestClient::new("user1");
    let (user1_msg_tx, mut user1_msg_rx) = tokio::sync::mpsc::channel(1);
    user1.set_callback(Box::new(move |msg| {
        let tx = user1_msg_tx.clone();
        tokio::spawn(async move {
            tx.send(msg).await.expect("unable to send");
        });
    }));
    user1.connect(HOST, port).await;

    let mut user2 = TestClient::new("user2");
    user2.connect(HOST, port).await;

    tokio::time::timeout(TIMEOUT_SECONDS, async {
        let expected = Payload {
            event_type: PayloadEventType::Connected,
            username: user2.username,
            message: None,
        };
        check_payload(&mut user1_msg_rx, &expected).await;
    })
    .await
    .expect("timed out");
}

#[tokio::test]
async fn multiple_users_get_notified_on_third_user_join() {
    Lazy::force(&LOGGER);

    let listener = TcpListener::bind(format!("{HOST}:0"))
        .await
        .expect("unable to bind socket");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let server_state = SharedServerState::default();
        ws_server::run_ws_server(listener, server_state).await
    });

    let mut user1 = TestClient::new("user1");
    let (user1_msg_tx, mut user1_msg_rx) = tokio::sync::mpsc::channel(2);
    user1.set_callback(Box::new(move |msg| {
        let tx = user1_msg_tx.clone();
        tokio::spawn(async move {
            tx.send(msg).await.expect("unable to send");
        });
    }));
    user1.connect(HOST, port).await;

    let mut user2 = TestClient::new("user2");
    let (user2_msg_tx, mut user2_msg_rx) = tokio::sync::mpsc::channel(1);
    user2.set_callback(Box::new(move |msg| {
        let tx = user2_msg_tx.clone();
        tokio::spawn(async move {
            tx.send(msg).await.expect("unable to send");
        });
    }));
    user2.connect(HOST, port).await;

    let mut user3 = TestClient::new("user3");
    user3.connect(HOST, port).await;

    tokio::time::timeout(TIMEOUT_SECONDS, async {
        let user2_connect_payload = Payload {
            event_type: PayloadEventType::Connected,
            username: user2.username,
            message: None,
        };
        let user3_connect_payload = Payload {
            event_type: PayloadEventType::Connected,
            username: user3.username,
            message: None,
        };
        check_payload(&mut user1_msg_rx, &user2_connect_payload).await;
        check_payload(&mut user1_msg_rx, &user3_connect_payload).await;
        check_payload(&mut user2_msg_rx, &user3_connect_payload).await;
    })
    .await
    .expect("timed out");
}

#[tokio::test]
async fn bidirectional_messaging_between_two_users() {
    Lazy::force(&LOGGER);

    let listener = TcpListener::bind(format!("{HOST}:0"))
        .await
        .expect("unable to bind socket");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let server_state = SharedServerState::default();
        ws_server::run_ws_server(listener, server_state).await
    });

    let mut user1 = TestClient::new("user1");
    let (user1_msg_tx, mut user1_msg_rx) = tokio::sync::mpsc::channel(2);
    user1.set_callback(Box::new(move |msg| {
        let tx = user1_msg_tx.clone();
        tokio::spawn(async move {
            tx.send(msg).await.expect("unable to send");
        });
    }));
    user1.connect(HOST, port).await;

    let mut user2 = TestClient::new("user2");
    let (user2_msg_tx, mut user2_msg_rx) = tokio::sync::mpsc::channel(2);
    user2.set_callback(Box::new(move |msg| {
        let tx = user2_msg_tx.clone();
        tokio::spawn(async move {
            tx.send(msg).await.expect("unable to send");
        });
    }));
    user2.connect(HOST, port).await;

    user1.send_message("hello 1").await;
    user2.send_message("hello 2").await;

    tokio::time::timeout(TIMEOUT_SECONDS, async {
        let expected = Payload {
            event_type: PayloadEventType::Connected,
            username: user2.username.clone(),
            message: None,
        };
        check_payload(&mut user1_msg_rx, &expected).await;

        let expected = Payload {
            event_type: PayloadEventType::Message,
            username: user1.username.clone(),
            message: Some("hello 1".into()),
        };
        check_payload(&mut user2_msg_rx, &expected).await;
        let expected = Payload {
            event_type: PayloadEventType::Message,
            username: user2.username.clone(),
            message: Some("hello 2".into()),
        };
        check_payload(&mut user1_msg_rx, &expected).await;
    })
    .await
    .expect("timed out");
}

#[tokio::test]
async fn disallow_two_clients_with_same_username() {
    Lazy::force(&LOGGER);

    let listener = TcpListener::bind(format!("{HOST}:0"))
        .await
        .expect("unable to bind socket");
    let port = listener.local_addr().unwrap().port();
    let server_state = SharedServerState::default();
    let server_state_clone = server_state.clone();
    tokio::spawn(async move { ws_server::run_ws_server(listener, server_state_clone).await });

    let mut user1 = TestClient::new("user1");
    let mut user2 = TestClient::new("user1");
    user1.connect(HOST, port).await;
    user2.connect(HOST, port).await;

    let user_count = server_state.lock().await.clients.len();
    assert_eq!(user_count, 1);
}
