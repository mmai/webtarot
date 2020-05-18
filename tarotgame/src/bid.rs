//! Auctions and bidding during the first phase of the deal.

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::cards;
use super::deal;
use super::pos;

/// Goal set by a contract.
///
/// Determines the winning conditions and the score on success.
#[derive(EnumIter, PartialEq, PartialOrd, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Target {
    Prise,
    Garde,
    GardeSans,
    GardeContre,
}

impl Target {
    /// Returns the score this target would give on success.
    pub fn multiplier(self) -> i32 {
        match self {
            Target::Prise => 1,
            Target::Garde => 2,
            Target::GardeSans => 4,
            Target::GardeContre => 6,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Target::Prise => "prise",
            Target::Garde => "garde",
            Target::GardeSans => "garde sans",
            Target::GardeContre => "garde contre",
        }
    }

}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "prise" => Ok(Target::Prise),
            "garde" => Ok(Target::Garde),
            "garde sans" => Ok(Target::GardeSans),
            "garde contre" => Ok(Target::GardeContre),
            _ => Err(format!("invalid target: {}", s)),
        }
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        self.to_str().to_owned()
    }
}

/// Contract taken by a team.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    /// Initial author of the contract.
    pub author: pos::PlayerPos,
    /// Target for the contract.
    pub target: Target,
}

impl Contract {
    fn new(author: pos::PlayerPos, target: Target) -> Self {
        Contract {
            author,
            target,
        }
    }
}

impl ToString for Contract {
    fn to_string(&self) -> String {
        format!("{}", self.target.to_str())
    }
}


/// Current state of an auction
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AuctionState {
    /// Players are still bidding for the highest contract
    Bidding,
    /// Auction is over, deal will begin
    Over,
    /// No contract was taken, a new deal will start
    Cancelled,
}

/// Represents the entire auction process.
pub struct Auction {
    history: Vec<Contract>,
    pass_count: usize,
    first: pos::PlayerPos,
    state: AuctionState,
    players: [cards::Hand; super::NB_PLAYERS],
    dog: cards::Hand,
}

/// Possible error occuring during an Auction.
#[derive(PartialEq, Debug)]
pub enum BidError {
    /// The auction was closed and does not accept more contracts.
    AuctionClosed,
    /// A player tried bidding before his turn.
    TurnError,
    /// The given bid was not higher than the previous one.
    NonRaisedTarget,
    /// Cannot complete the auction when it is still running.
    AuctionRunning,
    /// No contract was offered during the auction, it cannot complete.
    NoContract,
}

impl fmt::Display for BidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BidError::AuctionClosed => write!(f, "auctions are closed"),
            BidError::TurnError => write!(f, "invalid turn order"),
            BidError::NonRaisedTarget => write!(f, "bid must be higher than current contract"),
            BidError::AuctionRunning => write!(f, "the auction are still running"),
            BidError::NoContract => write!(f, "no contract was offered"),
        }
    }
}

impl Auction {
    /// Starts a new auction, starting with the player `first`.
    pub fn new(first: pos::PlayerPos) -> Self {
        let (hands, dog) = super::deal_hands();
        Auction {
            history: Vec::new(),
            pass_count: 0,
            state: AuctionState::Bidding,
            first,
            players: hands,
            dog
        }
    }

    /// Override Auction hands (for tests)
    pub fn set_hands(&mut self, hands: [cards::Hand; super::NB_PLAYERS]) {
        self.players = hands;
    }

    /// Returns the current state of the auctions.
    pub fn get_state(&self) -> AuctionState {
        self.state
    }

    fn can_bid(&self, target: Target) -> Result<(), BidError> {
        if self.state != AuctionState::Bidding {
            return Err(BidError::AuctionClosed);
        }

        if !self.history.is_empty()
            && target.multiplier() <= self.history[self.history.len() - 1].target.multiplier()
        {
            return Err(BidError::NonRaisedTarget);
        }

        Ok(())
    }

    /// Returns the player that is expected to play next.
    pub fn next_player(&self) -> pos::PlayerPos {
        let base = if let Some(contract) = self.history.last() {
            contract.author.next()
        } else {
            self.first
        };
        base.next_n(self.pass_count)
    }

    /// Bid a new, higher contract.
    pub fn bid(
        &mut self,
        pos: pos::PlayerPos,
        target: Target,
    ) -> Result<AuctionState, BidError> {
        if pos != self.next_player() {
            return Err(BidError::TurnError);
        }

        self.can_bid(target)?;

        // If we're all the way to the top, there's nowhere else to go
        if target == Target::GardeContre {
            self.state = AuctionState::Over;
        }

        let contract = Contract::new(pos, target);
        self.history.push(contract);
        self.pass_count = 0;

        // Only stops the bids if the guy asked for a capot
        Ok(self.state)
    }

    /// Look at the last offered contract.
    ///
    /// Returns `None` if no contract was offered yet.
    pub fn current_contract(&self) -> Option<&Contract> {
        if self.history.is_empty() {
            None
        } else {
            Some(&self.history[self.history.len() - 1])
        }
    }

    /// Returns the players cards.
    pub fn hands(&self) -> [cards::Hand; super::NB_PLAYERS] {
        self.players
    }

    /// The current player passes his turn.
    ///
    /// Returns the new auction state :
    ///
    /// * `AuctionState::Cancelled` if all players passed
    /// * `AuctionState::Over` if 5 players passed in a row
    /// * The previous state otherwise
    pub fn pass(&mut self, pos: pos::PlayerPos) -> Result<AuctionState, BidError> {
        if pos != self.next_player() {
            return Err(BidError::TurnError);
        }

        self.pass_count += 1;

        // After 4 passes, we're back to the contract author, and we can start.
        if !self.history.is_empty() {
            if self.pass_count >= super::NB_PLAYERS - 1 {
                self.state = AuctionState::Over;
            }
        } else if self.pass_count >= super::NB_PLAYERS {
            self.state = AuctionState::Cancelled;
        };

        Ok(self.state)
    }

    /// Consumes a complete auction to enter the second deal phase.
    ///
    /// If the auction was ready, returns `Ok<DealState>`
    pub fn complete(&mut self) -> Result<deal::DealState, BidError> {
        if self.state != AuctionState::Over {
            Err(BidError::AuctionRunning)
        } else if self.history.is_empty() {
            Err(BidError::NoContract)
        } else {
            Ok(deal::DealState::new(
                self.first,
                self.players,
                self.dog,
                self.history.pop().expect("contract history empty"),
                pos::PlayerPos::P0, //XXX placeholder
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    #[test]
    fn test_auction() {
        let mut auction = Auction::new(pos::PlayerPos::P0);

        assert!(auction.state == AuctionState::Bidding);

        // First four people pass.
        assert_eq!(auction.pass(pos::PlayerPos::P0), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P1), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P2), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P3), Ok(AuctionState::Bidding));

        assert_eq!(auction.pass(pos::PlayerPos::P1), Err(BidError::TurnError));

        // Someone bids.
        assert_eq!(
            auction.bid(pos::PlayerPos::P4, Target::Garde),
            Ok(AuctionState::Bidding)
        );
        assert_eq!(
            auction.bid(pos::PlayerPos::P0, Target::Garde).err(),
            Some(BidError::NonRaisedTarget)
        );
        assert_eq!(
            auction.bid(pos::PlayerPos::P1, Target::GardeSans).err(),
            Some(BidError::TurnError)
        );
        assert_eq!(auction.pass(pos::PlayerPos::P0), Ok(AuctionState::Bidding));
        // Surbid
        assert_eq!(
            auction.bid(pos::PlayerPos::P1, Target::GardeSans),
            Ok(AuctionState::Bidding)
        );
        assert_eq!(auction.pass(pos::PlayerPos::P2), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P3), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P4), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::P0), Ok(AuctionState::Over));

        assert!(auction.state == AuctionState::Over);

        match auction.complete() {
            Err(_) => assert!(false),
            _ => {}
        }
    }
}
