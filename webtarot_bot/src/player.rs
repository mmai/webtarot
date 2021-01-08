use tungstenite::{connect, Message as TMessage};
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::Hand} ;
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, GamePlayerState};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

pub fn play(join_code: String) {
    env_logger::init();

    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:8001/ws/new_new").unwrap()).expect("Can't connect");

    let mut player1 = SocketPlayer { socket, join_code, game_state: GameStateSnapshot::default(), player_info: PlayerInfo { id: Uuid::default(), nickname: "nobody".into()  }  };
    player1.play();
}

struct SocketPlayer {
    socket: WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>,
    join_code: String,
    game_state: GameStateSnapshot,
    player_info: PlayerInfo,
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

    pub fn my_state(&self) -> &GamePlayerState {
        self.game_state
            .players
            .iter()
            .find(|state| state.player.id == self.player_info.id)
            .unwrap()
    }

    fn send(&mut self, command: &Command) -> Result<()> {
        let json = serde_json::to_string(command)?;
        self.socket.write_message(TMessage::Text(json)).unwrap();
        Ok(())
    }

    fn handle_server_message(&mut self, msg: Message){
        match msg {
            Message::Authenticated(player_info) => {
                self.player_info = player_info;
                println!("Authenticated with id {}", self.player_info.id);
                self.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
            }
            Message::GameJoined(game_info) => {
                println!("Game joined: {}", game_info.game_id);
                self.send(&Command::MarkReady);
            }
            Message::GameStateSnapshot(game_state) => {
                self.game_state = game_state;
                self.handle_new_state();
            }
            Message::Chat(_) => {}
            Message::Pong => {
                println!("Received a pong !!");
            }
            _ => {
                println!("Unmanaged server message: {:?}", msg);
            }
        }
    }

    fn handle_new_state(&mut self){
        let my_state = self.my_state();
        // let card_played = self.game_state.deal.last_trick.card_played(my_state.pos);
        let player_action = my_state.get_turn_player_action(self.game_state.turn);
        // let mypos = my_state.pos.to_n();
        // let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        match player_action {
            Some(PlayerAction::Bid) => {
                println!("I pass...");
                self.send(&Command::GamePlay(GamePlayCommand::Pass));
            }
            _ => {}
        }
    }

}

