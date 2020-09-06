use serde::{Deserialize, Serialize};

use crate::message::ProtocolError;
use webgame_protocol::ProtocolErrorKind;
use tarotgame::{cards, bid, deal};

impl From<deal::PlayError> for ProtocolError {
    fn from(error: deal::PlayError) -> Self {
        ProtocolError::new(
            ProtocolErrorKind::BadState,
            format!("play: {}", error)
       )
    }
}

impl From<bid::BidError> for ProtocolError {
    fn from(error: bid::BidError) -> Self {
        ProtocolError::new(
            ProtocolErrorKind::BadState,
            format!("bid: {}", error)
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum GamePlayCommand {
    Bid(BidCommand),
    Play(PlayCommand),
    Pass,
    CallKing(CallKingCommand),
    MakeDog(MakeDogCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidCommand {
    pub target: bid::Target,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayCommand {
    pub card: cards::Card,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallKingCommand {
    pub card: cards::Card,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MakeDogCommand {
    pub cards: cards::Hand,
}
