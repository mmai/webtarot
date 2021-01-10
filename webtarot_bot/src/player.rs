use tungstenite::{connect, Message as TMessage};
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

// use std::thread;
use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::{Card, Hand}, deal::can_play} ;
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

pub fn play(join_code: String) {
    env_logger::init();

    let sockets: Vec<TarotSocket> =[1,2,3,4].iter().map(|i| {
        connect(Url::parse("ws://127.0.0.1:8001/ws/new_new")
            .unwrap())
            .expect("Can't connect")
            .0
    }).collect();

    sockets.into_par_iter().enumerate().for_each(|(i, socket)| {
        let mut player = SocketPlayer { 
            socket,
            join_code: join_code.clone(),
            game_state: GameStateSnapshot::default(),
            player_info: PlayerInfo { id: Uuid::default(), nickname: format!("TAROBOT-{}", i)} 
        };
        player.play();
    });

}

struct SocketPlayer {
    socket: TarotSocket,
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
    pub fn check_join_code(&mut self) -> bool {
        if self.join_code == "" {
            println!("Get available games codes");
            let json = r#"{"cmd": "show_server_status"}"#.into();
            self.socket.write_message(TMessage::Text(json)).unwrap();
            return false;
        }
        true
    }

    pub fn play(&mut self){
        self.send(&Command::Authenticate(AuthenticateCommand { nickname: self.player_info.nickname.clone() }));
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
                if self.check_join_code() {
                    self.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                }
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
            Message::ServerStatus(server_status) => {
                server_status.games.iter()
                    .next()
                    .map(|g| {
                        println!("Found a game with code {}", g.game.join_code);
                        self.join_code = g.game.join_code.clone();
                        self.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                    });
            }
            _ => {
                println!("Unmanaged server message for {}: {:?}", self.player_info.nickname, msg);
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
            Some(PlayerAction::Play) => {
                self.choose_card().map(|card| {
                    println!("{} is playing {}...", self.player_info.nickname, card.to_string());
                    self.send(&Command::GamePlay(GamePlayCommand::Play(PlayCommand { card })));
                });
            }
            _ => {}
        }
    }

    fn choose_card(&self) -> Option<Card>{
        let deal = &self.game_state.deal;
        let hand = deal.hand;
        hand.list().iter().find(|card| {
            can_play(self.my_state().pos, **card, hand, &deal.last_trick, deal.king, deal.trick_count == 1).is_ok()
        }).map(|c| *c)
    }

}

