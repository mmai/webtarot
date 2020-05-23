use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::protocol::{
    ProtocolError,
    GameState,
    GameInfo, Message, PlayerDisconnectedMessage, PlayerRole,
};
use crate::universe::Universe;
use tarotgame::{bid, cards};

pub struct Game {
    id: Uuid,
    join_code: String,
    universe: Weak<Universe>,
    game_state: Arc<Mutex<GameState>>,
}

impl Game {
    pub fn new(join_code: String, universe: Arc<Universe>) -> Game {
        Game {
            id: Uuid::new_v4(),
            join_code,
            universe: Arc::downgrade(&universe),
            game_state: Arc::new(Mutex::new(GameState::default())),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn join_code(&self) -> &str {
        &self.join_code
    }

    pub fn game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.id,
            join_code: self.join_code.to_string(),
        }
    }

    pub async fn is_joinable(&self) -> bool {
        self.game_state.lock().await.is_joinable()
    }

    pub fn universe(&self) -> Arc<Universe> {
        self.universe.upgrade().unwrap()
    }

    pub async fn add_player(&self, player_id: Uuid) {
        let universe = self.universe();
        if !universe
            .set_player_game_id(player_id, Some(self.id()))
            .await
        {
            return;
        }

        // TODO: `set_player_game_id` also looks up.
        let player_info = match universe.get_player_info(player_id).await {
            Some(player_info) => player_info,
            None => return,
        };

        let mut game_state = self.game_state.lock().await;
        let pos = game_state.add_player(player_info);
        let player = game_state.player_by_pos(pos).unwrap().clone();
        drop(game_state);
        self.broadcast(&Message::PlayerConnected(player)).await;
    }

    pub async fn remove_player(&self, player_id: Uuid) {
        self.universe().set_player_game_id(player_id, None).await;

        let mut game_state = self.game_state.lock().await;

        if game_state.remove_player(player_id) {
            drop(game_state);
            self.broadcast(&Message::PlayerDisconnected(PlayerDisconnectedMessage {
                player_id,
            }))
            .await;
        }

        if self.is_empty().await {
            self.universe().remove_game(self.id()).await;
        }
    }

    pub async fn set_player_role(&self, player_id: Uuid, role: PlayerRole) {
        let mut game_state = self.game_state.lock().await;
        game_state.set_player_role(player_id, role);
    }

    pub async fn set_player_not_ready(&self, player_id: Uuid) {
        let mut game_state = self.game_state.lock().await;
        game_state.set_player_not_ready(player_id);
    }

    pub async fn mark_player_ready(&self, player_id: Uuid) {
        let mut game_state = self.game_state.lock().await;
        game_state.set_player_ready(player_id);
    }

    pub async fn broadcast(&self, message: &Message) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.get_players().keys().copied() {
            universe.send(player_id, message).await;
        }
    }

    pub async fn send(&self, player_id: Uuid, message: &Message) {
        self.universe().send(player_id, message).await;
    }

    pub async fn broadcast_state(&self) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.get_players().keys().copied() {
            let snapshot = game_state.make_snapshot(player_id);
            universe
                .send(
                    player_id,
                    &Message::GameStateSnapshot(snapshot),
                )
                .await;
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.game_state.lock().await.get_players().is_empty()
    }


    pub async fn set_bid(&self, pid: Uuid, target: bid::Target) -> Result<(), ProtocolError> {
        let mut game_state = self.game_state.lock().await;
        game_state.set_bid(pid, target)?;
        Ok(())
    }

    pub async fn set_pass(&self, pid: Uuid) -> Result<(), ProtocolError> {
        let mut game_state = self.game_state.lock().await;
        game_state.set_pass(pid)?;
        Ok(())
    }

    pub async fn set_play(&self, pid: Uuid, card: cards::Card) -> Result<(), ProtocolError> {
        let mut game_state = self.game_state.lock().await;
        game_state.set_play(pid, card)?;
        Ok(())
    }

    pub async fn call_king(&self, pid: Uuid, card: cards::Card){
        let mut game_state = self.game_state.lock().await;
        game_state.call_king(pid, card);
    }

    pub async fn make_dog(&self, pid: Uuid, cards: cards::Hand){
        let mut game_state = self.game_state.lock().await;
        game_state.make_dog(pid, cards);
    }

}
