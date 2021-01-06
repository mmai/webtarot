use tungstenite::{connect, Message};
use url::Url;

use tarotgame::{deal_seeded_hands, cards::Hand} ;

pub fn play() {
    env_logger::init();

    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:8001/ws/new_new").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    // println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");
    // for (ref header, _value) in response.headers() {
    //     println!("* {}", header);
    // }

    socket.write_message(Message::Text(r#"{"cmd": "show_uuid"}"#.into())).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
    socket.close(None);
}
