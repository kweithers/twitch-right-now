use axum::{
    extract::{ConnectInfo, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{stream::StreamExt, SinkExt};
use std::net::SocketAddr;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket))
}

async fn websocket(stream: WebSocket) {
    // By splitting we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // This task will receive twitch chat messages and forward any emotes in the emote_set to the client.
    let mut send_task = tokio::spawn(async move {
        let emotes = include_str!("emote-urls.txt");
        let mut emote_set = std::collections::HashSet::new();
        for emote in emotes.lines() {
            let emote_name = emote.split(":").next().unwrap();
            emote_set.insert(emote_name.to_owned());
        }

        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let msg = format!("{}:{}", message.channel_login, token);
                            tracing::info!("{}", msg);
                            match sender.send(Message::Text(msg)).await {
                                Ok(_) => tracing::info!("sent"),
                                Err(e) => tracing::info!("failed to send {e}"),
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    });

    // This task will receive twitch channel names from the client and then join/part them
    let mut recv_task = tokio::spawn(async move {
        let mut chats = std::collections::HashSet::new();
        while let Some(Ok(Message::Text(channel_name))) = receiver.next().await {
            if !chats.contains(&channel_name) {
                tracing::info!("Joining {}'s twitch chat", channel_name);
                client.join(channel_name.to_owned()).unwrap();
                chats.insert(channel_name);
            } else {
                tracing::info!("Leaving {}'s twitch chat", channel_name);
                client.part(channel_name.to_owned());
                chats.remove(&channel_name);
            }
        }

    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => {recv_task.abort(); tracing::info!("send_task exited. exiting recv_task.");},
        _ = (&mut recv_task) => {send_task.abort(); tracing::info!("recv_task exited. exiting send_task.");},
    };
}

async fn index(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Html<&'static str> {
    tracing::info!("connection from {addr}");
    Html(include_str!("index.html"))
}
