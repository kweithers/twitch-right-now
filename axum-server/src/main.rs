use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
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
        .route("/websocket", get(websocket_handler))
        .route("/", get(index));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket));
}

async fn websocket(stream: WebSocket) {
    // By splitting we can send and receive at the same time.
    let (mut sender, _receiver) = stream.split();

    let emotes = include_str!("emotes.txt");
    let mut emote_set = std::collections::HashSet::new();
    for emote in emotes.lines() {
        emote_set.insert(emote.to_owned());
    }

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // This task will receive twitch chat messages and forward emotes to the client.
    let mut send_task = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    // tracing::info!("{}", message.message_text);
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let msg = format!("{}:{}", message.channel_login, token);
                            tracing::info!("{}",msg);
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

    // This task will receive twitch channel names from the client and then join them
    // let mut recv_task = tokio::spawn(async move {
    //     // while let Some(Ok(Message::Text(channel_name))) = receiver. .next().await {
    //     //     tracing::info!("Joining {}'s twitch chat", channel_name);
    //     //     client.join(channel_name.to_owned()).unwrap();
    //     // }
    //     loop {
    //         match receiver.next().await {
    //             Some(message) => {
    //                 match message {
    //                     Ok(m) => tracing::info!("{:#?}", m),
    //                     Err(e) => tracing::info!("{:#?}", e),
    //                 }
    //             }
    //             None => (),
    //         }
    //     }
    // });
    client.join("asmongold".to_owned()).unwrap();
    client.join("payo".to_owned()).unwrap();
    client.join("staysafetv".to_owned()).unwrap();
    // // If any one of the tasks exit, abort the other.
    // tokio::select! {
    //     _ = (&mut send_task) => {recv_task.abort(); tracing::info!("sender closed; aborting");},
    //     _ = (&mut recv_task) => {send_task.abort(); tracing::info!("recv closed; aborting");},
    // };

    send_task.await.unwrap();
    // recv_task.await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}
