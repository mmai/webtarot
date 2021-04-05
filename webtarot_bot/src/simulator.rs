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

pub fn simulate(count: usize) {
    env_logger::init();

    let str_websocket = "ws://127.0.0.1:8001";
    // ----- Game ----------
    
    // let game_executor = GameExecutor::new();

    //----- BOTS ----------
    let in_outs: Vec<Box<TarotWebSocket>> = (0..count).map(|_| {
        Box::new(TarotWebSocket::new(str_websocket))
    }).collect();

    in_outs.into_par_iter().enumerate().for_each(|(i, in_out)| {
        let nickname = format!("BOT-{}", i);
        let delay = time::Duration::from_millis(6000); // 6s

        let mut bot = crate::player::Player::new(in_out, "dummycode".to_string(), nickname, delay);
        bot.play();
    });

}

/*
struct GameExecutor {
    game: Game,
}

impl GameExecutor {
    pub fn new(variant: Variant<GameStateType::VariantParameters>) -> Self {
        let game = Game::new(join_code, self.clone(), variant);
        GameExecutor {
            game
        }
    }

    pub fn add_player(&mut self) {
        let pos = game_state.add_player(user.into());
        let player = game_state.player_by_pos(pos).unwrap().clone();
    }


    pub async fn on_player_bid(
        player_id: Uuid,
        cmd: BidCommand,
    ) -> Result<(), ProtocolError> {
        game_state.set_bid(player_id, cmd.target, cmd.slam)?;
        self.broadcast_current_state().await;
        Ok(())
    }

    pub async fn on_player_announce(
        player_id: Uuid,
        cmd: AnnounceCommand,
    ) -> Result<(), ProtocolError> {
        let ann = cmd.announce.clone();
        if let Err(e) = game_state.set_announce(player_id, cmd.announce) {
            drop(game_state);
            game.send(player_id, &Message::Error(e.into())).await;
        } else {
            drop(game_state);
            game.broadcast(&Message::PlayEvent(PlayEvent::Announce ( player_id, ann))).await;
        }
        Ok(())
    }



}
*/
