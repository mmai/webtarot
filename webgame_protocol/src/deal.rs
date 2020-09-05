use serde::{Deserialize, Serialize};

use tarotgame::{NB_PLAYERS, bid, cards, pos, deal, trick};

/// Describe a single deal.
pub enum Deal {
    /// The deal is still in the auction phase
    Bidding(bid::Auction),
    /// The deal is in the main playing phase
    Playing(deal::DealState),
}

impl Deal {
    // Creates a new deal, starting with an auction.
    pub fn new(first: pos::PlayerPos) -> Self {
        let auction = bid::Auction::new(first);
        Deal::Bidding(auction)
    }

    pub fn next_player(&self) -> pos::PlayerPos {
        match self {
            &Deal::Bidding(ref auction) => auction.next_player(),
            &Deal::Playing(ref deal) => deal.next_player(),
        }
    }

    pub fn hands(&self) -> [cards::Hand; NB_PLAYERS] {
        match self {
            &Deal::Bidding(ref auction) => auction.hands(),
            &Deal::Playing(ref deal) => deal.hands(),
        }
    }

    pub fn deal_contract(&self) -> Option<&bid::Contract> {
        match self {
            Deal::Bidding(auction) => auction.current_contract(),
            Deal::Playing(deal_state) => Some(deal_state.contract()),
        }
    }

    pub fn deal_auction(&self) -> Option<&bid::Auction> {
        match self {
            Deal::Bidding(bid) => Some(bid),
            Deal::Playing(_) => None,
        }
    }

    pub fn deal_auction_mut(&mut self) -> Option<&mut bid::Auction> {
        match self {
            Deal::Bidding(ref mut auction) => Some(auction),
            Deal::Playing(_) => None,
        }
    }

    pub fn deal_state(&self) -> Option<&deal::DealState> {
        match self {
            Deal::Bidding(_) => None,
            Deal::Playing(state) => Some(state),
        }
    }

    pub fn deal_state_mut(&mut self) -> Option<&mut deal::DealState> {
        match self {
            Deal::Bidding(_) => None,
            Deal::Playing(ref mut state) => Some(state),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DealSnapshot {
    pub hand: cards::Hand,
    pub current: pos::PlayerPos,
    pub contract: Option<bid::Contract>,
    pub king: Option<cards::Card>,
    pub scores: [f32; NB_PLAYERS],
    pub last_trick: trick::Trick,
    pub initial_dog: cards::Hand,
    pub dog: cards::Hand, // set to empty hand until the deal is over
    pub taker_diff: f32,
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

}

