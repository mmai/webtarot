use tungstenite::connect;
use tungstenite::Message as TMessage;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

use rayon::prelude::*;

use url::Url;
use serde_json::Result;

use webtarot_protocol::{Message, Command};

use crate::player::InOut;

pub struct TarotWebSocket {
    socket: WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>,
}

impl TarotWebSocket {
    pub fn new(str_websocket: &str) -> Self {
        let url_websocket = format!("{}/ws/new_new", str_websocket);
        let socket = connect(Url::parse(&url_websocket)
            .unwrap())
            .expect("Can't connect")
            .0;
        TarotWebSocket {
            socket
        }
    }

}

impl InOut for TarotWebSocket {
    fn read(&mut self) -> Message {
        let msg = self.socket.read_message().expect("Error reading message");
        let msg = match msg {
            tungstenite::Message::Text(s) => { s }
            _ => { panic!() }
        };

        let message: Message = serde_json::from_str(&msg).expect("Can't parse JSON");
        message
    }

    fn send(&mut self, command: &Command) -> Result<()> {
        let json = serde_json::to_string(command)?;
        self.socket.write_message(TMessage::Text(json)).unwrap();
        Ok(())
    }

    fn close(&mut self){
        self.socket.close(None);
    }

}
