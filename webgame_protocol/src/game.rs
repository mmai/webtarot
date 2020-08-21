use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;

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

pub trait GameState {
    type PlayerPos;
    type GamePlayerState;
    type PlayerRole;

    fn is_joinable(&self) -> bool;
    fn get_players(&self) -> &BTreeMap<Uuid, Self::GamePlayerState>;
    fn add_player(&mut self, player_info: PlayerInfo) -> Self::PlayerPos; 
    fn remove_player(&mut self, player_id: Uuid) -> bool;
    fn set_player_role(&mut self, player_id: Uuid, role: Self::PlayerRole);
}
