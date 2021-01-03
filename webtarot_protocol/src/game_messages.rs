use serde::{Deserialize, Serialize};

use crate::message::ProtocolError;
use webgame_protocol::ProtocolErrorKind;
use tarotgame::{cards, bid, deal, Announce};

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
#[serde(tag = "gcmd", rename_all = "snake_case")]
pub enum GamePlayCommand {
    Bid(BidCommand),
    Announce(AnnounceCommand),
    Play(PlayCommand),
    Pass,
    CallKing(CallKingCommand),
    MakeDog(MakeDogCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidCommand {
    pub target: bid::Target,
    pub slam: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnnounceCommand {
    pub announce: Announce,
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
    pub slam: bool,
}
