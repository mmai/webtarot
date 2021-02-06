use tungstenite::connect;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

// use std::thread;
use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

use crate::player;

type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

pub fn play(join_code: &str, str_websocket: &str, count: usize) {
    env_logger::init();

    let sockets: Vec<TarotSocket> = (0..count).map(|_| {
        let url_websocket = format!("{}/new_new", str_websocket);
        connect(Url::parse(&url_websocket)
            .unwrap())
            .expect("Can't connect")
            .0
    }).collect();

    sockets.into_par_iter().enumerate().for_each(|(i, socket)| {
        let nickname = format!("TAROBOT-{}", i);
        let mut bot = player::SocketPlayer::new(socket, join_code.to_string(), nickname);
        bot.play();
    });

}
