use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;
use tarotgame::{bid, cards, pos, game, trick};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    Pregame,
    Intergame,
    BiddingP0,
    BiddingP1,
    BiddingP2,
    BiddingP3,
    BiddingP4,
    PlayingP0,
    PlayingP1,
    PlayingP2,
    PlayingP3,
    PlayingP4,
    Endgame,
}
// pub enum Turn {
//     Pregame,
//     Intermission,
//     RedSpymasterThinking,
//     BlueSpymasterThinking,
//     RedOperativesGuessing,
//     BlueOperativesGuessing,
//     Endgame,
// }



#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
    Bid,
    Play,
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Turn::Pregame => "pre-game",
                Turn::Intergame => "inter game",
                Turn::BiddingP0 => "Player 0 bidding",
                Turn::BiddingP1 => "Player 1 bidding",
                Turn::BiddingP2 => "Player 2 bidding",
                Turn::BiddingP3 => "Player 3 bidding",
                Turn::BiddingP4 => "Player 4 bidding",
                Turn::PlayingP0 => "Player 0 playing",
                Turn::PlayingP1 => "Player 1 playing",
                Turn::PlayingP2 => "Player 2 playing",
                Turn::PlayingP3 => "Player 3 playing",
                Turn::PlayingP4 => "Player 4 playing",
                Turn::Endgame => "end",
            }
            // match *self {
            //     Turn::Pregame => "pre-game",
            //     Turn::Intermission => "intermission",
            //     Turn::RedSpymasterThinking => "red spymaster",
            //     Turn::RedOperativesGuessing => "red operatives",
            //     Turn::BlueSpymasterThinking => "blue spymaster",
            //     Turn::BlueOperativesGuessing => "blue operatives",
            //     Turn::Endgame => "end",
            // }
        )
    }
}

impl Turn {
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
    pub tiles: Vec<Tile>,
    pub turn: Turn,
    pub deal: DealSnapshot,
}

impl Default for GameStateSnapshot {
    fn default() -> GameStateSnapshot {
        GameStateSnapshot {
            players: vec![],
            tiles: vec![Tile::default(); 25],
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
