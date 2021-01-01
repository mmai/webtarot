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

/// Contract taken
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    /// Initial author of the contract.
    pub author: pos::PlayerPos,
    /// Target for the contract.
    pub target: Target,
    /// Slam asked ?
    pub slam: bool,
}

impl Contract {
    fn new(author: pos::PlayerPos, target: Target, slam: bool) -> Self {
        Contract {
            author,
            target,
            slam,
        }
    }
}

impl ToString for Contract {
    fn to_string(&self) -> String {
        let str_slam = if self.slam { " SLAM" } else { "" };
        format!("{}{}", self.target.to_str(), str_slam)
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

/// Bidding status for a player
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum BidStatus {
    Todo,
    Passed,
    Bid,
}


/// Represents the entire auction process.
#[derive(Debug, Clone)]
pub struct Auction {
    contract: Option<Contract>,
    players_status: Vec<BidStatus>, 
    first: pos::PlayerPos,
    state: AuctionState,
    players: Vec<cards::Hand>,
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
        let count = first.count as usize;
        let (hands, dog) = super::deal_hands(count);
        Auction {
            contract: None,
            players_status: vec![BidStatus::Todo; count],
            state: AuctionState::Bidding,
            first,
            players: hands,
            dog
        }
    }

    /// Override Auction hands (for tests)
    pub fn set_hands(&mut self, hands: Vec<cards::Hand>, dog: cards::Hand) {
        self.players = hands;
        self.dog = dog;
    }

    /// Returns the current state of the auctions.
    pub fn get_state(&self) -> AuctionState {
        self.state
    }

    fn can_bid(&self, target: Target) -> Result<(), BidError> {
        if self.state != AuctionState::Bidding {
            return Err(BidError::AuctionClosed);
        }

        if let Some(contract) = self.contract.clone() {
            if target.multiplier() <= contract.target.multiplier() {
                return Err(BidError::NonRaisedTarget);
            }
        }

        Ok(())
    }

    fn get_player_status(&self, pos: pos::PlayerPos) -> BidStatus {
        self.players_status[pos.to_n()]
    }

    fn set_player_status(&mut self, pos: pos::PlayerPos, status: BidStatus) {
        self.players_status[pos.to_n()] = status;
    }

    /// Returns the player that is expected to bid next.
    pub fn next_player(&self) -> pos::PlayerPos {
        let pos_init = if let Some(contract) = self.contract.clone() {
            contract.author.next()
        } else {
            self.first
        };

        let mut next_pos = pos_init;
        while self.get_player_status(next_pos) != BidStatus::Todo {
            next_pos = next_pos.next();
            if next_pos == pos_init {
                panic!("all players have talked")
            }
        }
        next_pos
    }

    /// Check if there are still players waiting for bidding
    fn no_player_left(&self) -> bool {
        !self.players_status.contains(&BidStatus::Todo)
    }

    /// Bid a new, higher contract.
    pub fn bid(
        &mut self,
        pos: pos::PlayerPos,
        target: Target,
        slam: bool,
    ) -> Result<AuctionState, BidError> {
        if pos != self.next_player() {
            return Err(BidError::TurnError);
        }

        self.can_bid(target)?;

        // Reset previous bidder status
        if let Some(contract) = self.contract.clone() {
            self.set_player_status(contract.author, BidStatus::Todo);
        }

        let contract = Contract::new(pos, target, slam);
        self.contract = Some(contract);
        self.set_player_status(pos, BidStatus::Bid);

        // If we're all the way to the top, there's nowhere else to go
        if self.no_player_left() || target == Target::GardeContre {
            self.state = AuctionState::Over;
        }

        Ok(self.state)
    }

    /// Look at the last offered contract.
    ///
    /// Returns `None` if no contract was offered yet.
    pub fn current_contract(&self) -> Option<&Contract> {
        self.contract.as_ref()
    }

    /// Returns the players cards.
    pub fn hands(&self) -> &Vec<cards::Hand> {
        &self.players
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
        self.set_player_status(pos, BidStatus::Passed);

        if self.no_player_left() {
            self.state = if self.contract.is_some() {
                AuctionState::Over
            } else {
                AuctionState::Cancelled
            }
        }

        Ok(self.state)
    }

    /// Consumes a complete auction to enter the second deal phase.
    ///
    /// If the auction was ready, returns `Ok<DealState>`
    pub fn complete(&self) -> Result<deal::DealState, BidError> {
        if self.state != AuctionState::Over {
            Err(BidError::AuctionRunning)
        // } else if self.contract.is_none() {
        } else {
            if let Some(contract) = self.contract.clone() {
                Ok(deal::DealState::new(
                    self.first,
                    self.players.clone(),
                    self.dog,
                    contract,
                    pos::PlayerPos::from_n(0,5), //XXX placeholder
                ))
            } else {
                Err(BidError::NoContract)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    #[test]
    fn test_auction() {
        let mut auction = Auction::new(pos::PlayerPos::from_n(0, 5));

        assert!(auction.state == AuctionState::Bidding);

        assert_eq!(auction.pass(pos::PlayerPos::from_n(0, 5)), Ok(AuctionState::Bidding));
        assert_eq!(auction.pass(pos::PlayerPos::from_n(1, 5)), Ok(AuctionState::Bidding));

        assert_eq!(auction.pass(pos::PlayerPos::from_n(3, 5)), Err(BidError::TurnError));

        assert_eq!(auction.pass(pos::PlayerPos::from_n(2, 5)), Ok(AuctionState::Bidding));


        // Someone bids.
        assert_eq!(
            auction.bid(pos::PlayerPos::from_n(3, 5), Target::Garde, false),
            Ok(AuctionState::Bidding)
        );

        assert_eq!(
            auction.bid(pos::PlayerPos::from_n(4, 5), Target::Garde, false).err(),
            Some(BidError::NonRaisedTarget)
        );
        // Surbid
        assert_eq!(
            auction.bid(pos::PlayerPos::from_n(4, 5), Target::GardeSans, false),
            Ok(AuctionState::Bidding)
        );

        // Allready passed
        assert_eq!(auction.pass(pos::PlayerPos::from_n(0, 5)), Err(BidError::TurnError));

        // Last to pass
        assert_eq!(auction.pass(pos::PlayerPos::from_n(3, 5)), Ok(AuctionState::Over));

        assert!(auction.state == AuctionState::Over);

        match auction.complete() {
            Err(_) => assert!(false),
            _ => {}
        }
    }
}
