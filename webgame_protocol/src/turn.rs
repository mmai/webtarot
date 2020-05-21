use std::fmt;
use serde::{Deserialize, Serialize};

use tarotgame::{bid, pos};
use crate::deal::Deal;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    Pregame,
    Intertrick,
    Interdeal,
    Bidding((bid::AuctionState, pos::PlayerPos)),
    CallingKing,
    MakingDog,
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
                Turn::Intertrick => "inter trick",
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
                Turn::CallingKing => "calling king",
                Turn::MakingDog => "making dog",
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

    pub fn from_deal(deal: &Deal) -> Self {
        match deal {
            Deal::Bidding(auction) => {
                Self::Bidding((auction.get_state(), auction.next_player()))
            },
            Deal::Playing(deal_state) => {
                Self::Playing(deal_state.next_player())
            },
        }
    }
}

