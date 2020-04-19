use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;
use tarotgame::{bid, cards, pos, deal, trick};

/// Describe a single deal.
pub enum Deal {
    /// The deal is still in the auction phase
    Bidding(bid::Auction),
    /// The deal is in the main playing phase
    Playing(deal::DealState),
}

impl Deal {
    pub fn next_player(&self) -> pos::PlayerPos {
        match self {
            &Deal::Bidding(ref auction) => auction.next_player(),
            &Deal::Playing(ref deal) => deal.next_player(),
        }
    }

    pub fn hands(&self) -> [cards::Hand; 4] {
        match self {
            &Deal::Bidding(ref auction) => auction.hands(),
            &Deal::Playing(ref deal) => deal.hands(),
        }
    }

    pub fn deal_state(&self) -> Option<&deal::DealState> {
        match self {
            Deal::Bidding(bid) => None,
            Deal::Playing(state) => Some(state),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    Pregame,
    Interdeal,
    Bidding((bid::AuctionState, pos::PlayerPos)),
    Playing(pos::PlayerPos),
    Endgame,
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strpos;
        write!(
            f,
            "{}",
            match *self {
                Turn::Pregame => "pre-game",
                Turn::Interdeal => "inter deal",
                Turn::Bidding((_, pos)) => {
                    strpos = format!("{:?} to bid", pos);
                    &strpos
                }
                Turn::Playing(pos) => {
                    strpos = format!("{:?} to play", pos);
                    &strpos
                }
                Turn::Endgame => "end",
            }
        )
    }
}

impl Turn {
    pub fn has_player_pos(&self) -> bool {
        match self {
            Self::Pregame => false,
            Self::Interdeal => false,
            Self::Endgame => false,
            _ => true
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
    Bid,
    Play,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DealSnapshot {
    pub hand: cards::Hand,
    pub current: pos::PlayerPos,
    // pub contract: bid::Contract,
    pub points: [i32; 2],
    // pub tricks: Vec<trick::Trick>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameStateSnapshot {
    pub players: Vec<GamePlayerState>,
    pub turn: Turn,
    pub deal: DealSnapshot,
}

impl GameStateSnapshot {
    // pub fn get_current_player(self) -> Option<PlayerInfo> {
    //     let player_info;
    //     match self.turn {
    //         Turn::Playing(pos) => player_info = Some(self.players[pos.to_n()].player.clone()),
    //         Turn::Bidding((_, pos)) => player_info = Some(self.players[pos.to_n()].player.clone()),
    //         _ => player_info = None
    //     }
    //     player_info
    // }
    pub fn get_playing_pos(&self) -> Option<pos::PlayerPos> {
        match self.turn {
            Turn::Playing(pos) => Some(pos),
            Turn::Bidding((_, pos)) => Some(pos),
            _ => None
        }
    }
}

impl Default for GameStateSnapshot {
    fn default() -> GameStateSnapshot {
        GameStateSnapshot {
            players: vec![],
            turn: Turn::Pregame,
            deal: DealSnapshot {
                hand: cards::Hand::new(),
                current: pos::PlayerPos::P2,
                points: [0;2]
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tile {
    pub codeword: String,
    pub spotted: bool,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            codeword: "".into(),
            spotted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlayerRole {
    Taker,
    Partner,
    Opponent,
    Unknown,
    Spectator,
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
        if self.role == PlayerRole::Spectator {
            return None;
        } else {
        // } else if (self.pos == {
            return Some(PlayerAction::Bid);
        }
    }
}
