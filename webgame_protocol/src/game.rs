use std::collections::BTreeMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::player::{PlayerInfo, PlayerState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub join_code: String,
}

//Used for server diagnostics
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameExtendedInfo {
    pub game: GameInfo,
    pub players: Vec<Uuid>
}

pub trait GameState<GamePlayerState: PlayerState, Snapshot: GameStateSnapshot>: Sync+Default+Send {
    type PlayerPos: Send;
    type PlayerRole;

    fn is_joinable(&self) -> bool;
    fn get_players(&self) -> &BTreeMap<Uuid, GamePlayerState>;
    fn add_player(&mut self, player_info: PlayerInfo) -> Self::PlayerPos; 
    fn remove_player(&mut self, player_id: Uuid) -> bool;
    fn set_player_role(&mut self, player_id: Uuid, role: Self::PlayerRole);
    fn player_by_pos(&self, position: Self::PlayerPos) -> Option<&GamePlayerState>;
    fn make_snapshot(&self, player_id: Uuid) -> Snapshot;
    fn set_player_ready(&mut self, player_id: Uuid);
    fn set_player_not_ready(&mut self, player_id: Uuid);
}

pub trait GameStateSnapshot: Debug+Serialize+DeserializeOwned+Send+Sync { }
