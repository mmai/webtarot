use tungstenite::{connect, Message as TMessage};
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::Hand} ;
use webtarot_protocol::{Message, Command};
use webgame_protocol::{AuthenticateCommand, PlayerInfo};

pub fn play() {
    env_logger::init();

    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:8001/ws/new_new").unwrap()).expect("Can't connect");

    let mut player1 = SocketPlayer { socket };
    player1.play();
}

struct SocketPlayer {
    socket: WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>,
}

impl Drop for SocketPlayer {
    fn drop(&mut self) {
        self.socket.close(None);
    }
}

impl SocketPlayer {
    pub fn play(&mut self){
        self.send(&Command::Authenticate(AuthenticateCommand { nickname: String::from("Henri") }));

        
        loop {
            let msg = self.socket.read_message().expect("Error reading message");
            let msg = match msg {
                tungstenite::Message::Text(s) => { s }
                _ => { panic!() }
            };
            let message: Message = serde_json::from_str(&msg).expect("Can't parse JSON");
            self.handle_server_message(message);
        }

    }

    fn send(&mut self, command: &Command) -> Result<()> {
        let json = serde_json::to_string(command)?;
        self.socket.write_message(TMessage::Text(json)).unwrap();
        Ok(())
    }

    fn handle_server_message(&mut self, msg: Message){
        match msg {
            Message::Authenticated(PlayerInfo { id, nickname }) => {
                println!("Authenticated with id {}", id);
                self.send(&Command::Ping);
            }
            Message::Pong => {
                println!("Received a pong !!");
            }
            _ => {
                println!("Unmanaged server message: {:?}", msg);
            }
        }
    }

}

