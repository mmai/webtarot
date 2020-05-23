use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::turn::Turn;

use tarotgame::pos;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlayerRole {
    Taker,
    Partner,
    Opponent,
    Unknown,
    PreDeal,
    Spectator,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
    Bid,
    CallKing,
    MakeDog,
    Play,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GamePlayerState {
    pub player: PlayerInfo,
    pub pos: pos::PlayerPos,
    pub role: PlayerRole,
    pub ready: bool,
}

impl GamePlayerState {
    pub fn get_turn_player_action(&self, turn: Turn) -> Option<PlayerAction> {
        match turn {
            Turn::Bidding((_, pos)) if pos == self.pos => Some(PlayerAction::Bid),
            Turn::Playing(pos) if pos == self.pos => Some(PlayerAction::Play),
            Turn::CallingKing if self.role == PlayerRole::Taker => Some(PlayerAction::CallKing),
            Turn::MakingDog if self.role == PlayerRole::Taker => Some(PlayerAction::MakeDog),
            _ => None
        }
    }
}
