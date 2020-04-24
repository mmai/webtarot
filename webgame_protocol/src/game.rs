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

    pub fn deal_auction(&self) -> Option<&bid::Auction> {
        match self {
            Deal::Bidding(bid) => Some(bid),
            Deal::Playing(_) => None,
        }
    }

    pub fn deal_state(&self) -> Option<&deal::DealState> {
        match self {
            Deal::Bidding(_) => None,
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
    pub contract: Option<bid::Contract>,
    pub points: [i32; 2],
    // pub tricks: Vec<trick::Trick>,
}

impl DealSnapshot {
    pub fn contract_target(&self) -> Option<bid::Target> {
        //let target = &self.contract.map(|c| c.target); // INFO : doesn't work...(2h to get the solution below)
        self.contract.as_ref().map(|c| c.target)
        // match &self.contract {
        //     None => None,
        //     Some(contract) => Some(contract.target)
        // }
    }

    pub fn contract_trump(&self) -> Option<cards::Suit> {
        match &self.contract {
            None => None,
            Some(contract) => Some(contract.trump)
        }
    }

    pub fn contract_coinche(&self) -> i32 {
        match &self.contract {
            None => 0,
            Some(contract) => contract.coinche_level
        }
    }
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
                contract: None,
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
        match turn {
            Turn::Bidding((_, pos)) if pos == self.pos => Some(PlayerAction::Bid),
            Turn::Playing(pos) if pos == self.pos => Some(PlayerAction::Play),
            _ => None
        }
    }
}
