use std::net::TcpListener;
use tungstenite::accept;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

/// A WebSocket server
/// Each connection creates a twitch client and joins certain channels
#[tokio::main]
pub async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9005").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream).await;
            }
            Err(e) => println!("{e}:::connection failed"),
        }
    }
}

async fn handle_connection(stream: std::net::TcpStream) {
    println!("new connection");
    let emotes = include_str!("emotes.txt");

    let mut websocket = accept(stream).unwrap();
    let mut emote_set = std::collections::HashSet::new();
    for emote in emotes.lines() {
        emote_set.insert(emote.to_owned());
    }
    let streamers = vec!["asmongold", "payo", "staysafetv"];

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // consume incoming Privmsg(s) (standard individual twitch chat messages)
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let key = format!("{}:{}", message.channel_login, token);
                            match websocket.write_message(key.into()) {
                                Ok(_) => (),
                                Err(_) => {
                                    println!("connection closed");
                                    return;
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    });

    // join channels
    for channel in streamers {
        client.join(channel.to_owned()).unwrap();
    }

    join_handle.await.unwrap();
}
