use tungstenite::connect;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

// use std::thread;
use rayon::prelude::*;

use std::time;
use uuid::Uuid;
use url::Url;
use serde_json::Result;

use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

use crate::player;
use crate::in_out_websocket::TarotWebSocket;

pub fn play(join_code: &str, str_websocket: &str, count: usize) {
    env_logger::init();

    if join_code == "" {
        let in_out = Box::new(TarotWebSocket::new(str_websocket));
        let nickname = format!("parent");
        let delay = time::Duration::from_millis(1000);
        let mut bot = crate::player::Player::new(in_out, join_code.to_string(), nickname, delay);
        bot.play();
    } else {
        let in_outs: Vec<Box<TarotWebSocket>> = (0..count).map(|_| {
            Box::new(TarotWebSocket::new(str_websocket))
        }).collect();

        in_outs.into_par_iter().enumerate().for_each(|(i, in_out)| {
            let nickname = format!("TAROBOT-{}", i);
            let delay = time::Duration::from_millis(1000);

            let mut bot = crate::player::Player::new(in_out, join_code.to_string(), nickname, delay);
            bot.play();
        });
    }

}
