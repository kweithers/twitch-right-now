use rand::seq::SliceRandom;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("127.0.0.1:9005").unwrap();

    for stream in server.incoming() {
        println!("new connection");
        spawn(move || {
            let streamers = vec!["asmongold","payo","staysafetv"];
            let emotes = vec!["LULW", "KEKW", "OMEGALUL"];

            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let streamer = streamers.choose(&mut rand::thread_rng()).unwrap().to_string() + ":";
                let emote = emotes.choose(&mut rand::thread_rng()).unwrap().to_string();
                let combo = streamer + emote.as_str();
                match websocket.write_message(combo.into()) {
                    Ok(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(_) => {
                        println!("connection closed");
                        break;
                    },
                }
            }
        });
    }
}
