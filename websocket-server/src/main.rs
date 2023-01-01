use rand::seq::SliceRandom;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    for stream in server.incoming() {
        println!("new connection");
        spawn(move || {
            let emotes = vec!["00000", "11111", "22222", "33333", "44444"];

            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                // let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                // if msg.is_binary() || msg.is_text() {
                // websocket.write_message(msg).unwrap();
                // }
                // match websocket.write_message(rand::random::<char>().to_string().into()) {
                let choose = emotes.choose(&mut rand::thread_rng()).unwrap().to_string() + " ";
                match websocket.write_message(choose.into()) {
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
