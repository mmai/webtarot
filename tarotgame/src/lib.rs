#![cfg_attr(feature = "use_bench", feature(test))]
//! Models a game of [french tarot](https://en.wikipedia.org/wiki/French_tarot).
//!
//! Here is a simple example:
//!
//! ```rust
//! use tarotgame::{bid,cards,pos};
//!
//! fn main() {
//!     // The first player
//!     let first = pos::PlayerPos::from_n(0, 5);
//!
//!     // Start the first phase with an auction
//!     let mut auction = bid::Auction::new(first);
//!
//!     // Check their cards
//!     let hands = auction.hands();
//!
//!     // Players bid or pass
//!     auction.bid(pos::PlayerPos::from_n(0, 5), bid::Target::Garde).unwrap();
//!     auction.pass(pos::PlayerPos::from_n(1, 5)).unwrap();
//!     auction.pass(pos::PlayerPos::from_n(2, 5)).unwrap();
//!     auction.pass(pos::PlayerPos::from_n(3, 5)).unwrap();
//!     // The result is `Over` when the auction is ready to complete
//!     match auction.pass(pos::PlayerPos::from_n(4, 5)) {
//!         Ok(bid::AuctionState::Over) => (),
//!         _ => panic!("Should not happen"),
//!     };
//!
//!     // Complete the auction to enter the second phase
//!     let mut deal = auction.complete().unwrap();
//!
//!     // Play some cards
//!     // deal.play_card(pos::PlayerPos::from_n(0, 5), hands[0].get_card());
//!     // ...
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(feature = "use_bench")]
extern crate test;

pub mod bid;
pub mod cards;
pub mod deal;
pub mod points;
pub mod pos;
pub mod trick;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AnnounceType {
    Poignee,
    DoublePoignee,
    TriplePoignee
}

impl Display for AnnounceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Poignee => write!(f, "Poignée"),
            Self::DoublePoignee => write!(f, "Double poignee"),
            Self::TriplePoignee => write!(f, "Triple poignee"),
        }
    }
}

impl AnnounceType {
    pub fn poignee_size(&self, players_count: usize) -> usize {
        match players_count {
            3 => match self { Self::Poignee => 13, Self::DoublePoignee => 15, Self::TriplePoignee => 18, },
            4 => match self { Self::Poignee => 10, Self::DoublePoignee => 13, Self::TriplePoignee => 15, },
            5 => match self { Self::Poignee => 8, Self::DoublePoignee => 10, Self::TriplePoignee => 13, },
            _ => self.poignee_size(5),
        }
    }

    pub fn is_eligible(&self, hand: cards::Hand) -> bool {
        let pcount = players_count(hand.size());
        self.poignee_size(pcount) <= hand.trumps_count()
    }

    pub fn check(&self, hand: cards::Hand, proof: cards::Hand) -> bool {
        // TODO check that proof cards are from the player hand
        // check the number of trumps
        let pcount = players_count(hand.size());
        self.poignee_size(pcount) <= proof.trumps_count()
    }

    pub fn eligibles(hand: cards::Hand) -> Vec<AnnounceType> {
        vec![Self::Poignee, Self::DoublePoignee, Self::TriplePoignee].into_iter()
            .filter(|atype| atype.is_eligible(hand))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Announce {
    pub atype: AnnounceType, 
    pub proof: Option<cards::Hand> 
}

// pub const NB_PLAYERS:usize = 5;
// pub const DOG_SIZE:usize = 3;
// const DEAL_SIZE:usize = (78 - DOG_SIZE) / NB_PLAYERS ;

pub fn dog_size(players_count: usize) -> usize {
    match players_count {
        3 => 6,
        4 => 6,
        5 => 3,
        _ => 3
    }
}

pub fn deal_size(players_count: usize) -> usize {
    match players_count {
        3 => 24,
        4 => 18,
        5 => 15,
        _ => 15,
    }
}

pub fn players_count(deal_size: usize) -> usize {
    match deal_size {
        24 => 3,
        18 => 4,
        15 => 5,
        _  => 5,
    }
}

// Expose the module or their content directly? Still unsure.

// pub use bid::*;
// pub use cards::*;
// pub use deal::*;
// pub use points::*;
// pub use pos::*;
// pub use trick::*;

/// Quick method to get cards for 4 players.
///
/// Deals cards to 5 players randomly.
// pub fn deal_hands(count: usize) -> (Vec<cards::Hand>, cards::Hand) {
//     let mut hands = vec![cards::Hand::new(); count];
//     let mut dog = cards::Hand::new();
//
//     let mut d = cards::Deck::new();
//     d.shuffle();
//     d.deal_each(&mut hands, deal_size(count));
//     for idx in 0..dog_size(count) {
//         dog.add(d.draw());
//     }
//
//     (hands, dog)
// }
pub fn deal_hands(count: usize) -> (Vec<cards::Hand>, cards::Hand) {
    let mut dealing: (Vec<cards::Hand>, cards::Hand) = (vec![], cards::Hand::new());
    let mut is_deal_ok = false;
    while !is_deal_ok {
        let mut d = cards::Deck::new();
        d.shuffle();
        dealing = deal_with_deck(d, count);
        is_deal_ok = check_deal_ok(&dealing.0);
    }
    dealing
}

fn check_deal_ok(hands: &Vec<cards::Hand>) -> bool {
   !hands.iter().any(|hand| hand.has_petit_sec())
}

/// Deal cards for players deterministically.
pub fn deal_seeded_hands(seed: [u8; 32], count: usize) -> (Vec<cards::Hand>, cards::Hand) {
    let mut d = cards::Deck::new();
    d.shuffle_seeded(seed);
    deal_with_deck(d, count)
}

fn deal_with_deck(mut d: cards::Deck, count: usize) -> (Vec<cards::Hand>, cards::Hand) {
    let mut hands = vec![cards::Hand::new(); count];
    let mut dog = cards::Hand::new();

    let batch_size = 3;
    let mut batch_done = 0;
    if count == 5 {
        d.deal_each(&mut hands, batch_size);
        batch_done = batch_done + 1;
    }

    let dog_count = dog_size(count);
    for _idx in 0..dog_count {
        d.deal_each(&mut hands, batch_size);
        batch_done = batch_done + 1;
        dog.add(d.draw());
    }

    let left_count = deal_size(count) - batch_size * batch_done;
    d.deal_each(&mut hands, left_count);

    (hands, dog)
}

#[test]
fn test_deals_tarot5() {
    let (hands, dog) = deal_hands(5);
    assert!(dog.size() == 3);

    let mut count = [0; 78];

    for card in dog.list().iter() {
        count[idx_from_id(card.id()) as usize] += 1;
    }
    for hand in hands.iter() {
        assert!(hand.size() == 15);
        for card in hand.list().iter() {
            count[idx_from_id(card.id()) as usize] += 1;
        }
    }

    for c in count.iter() {
        assert!(*c == 1);
    }

}

#[test]
fn test_deals_tarot4() {
    let (hands, dog) = deal_hands(4);
    assert!(dog.size() == 6);

    let mut count = [0; 78];

    for card in dog.list().iter() {
        count[idx_from_id(card.id()) as usize] += 1;
    }
    for hand in hands.iter() {
        assert!(hand.size() == 18);
        for card in hand.list().iter() {
            count[idx_from_id(card.id()) as usize] += 1;
        }
    }

    for c in count.iter() {
        assert!(*c == 1);
    }
}

#[test]
fn test_deals_tarot3() {
    let (hands, dog) = deal_hands(3);
    assert!(dog.size() == 6);

    let mut count = [0; 78];

    for card in dog.list().iter() {
        count[idx_from_id(card.id()) as usize] += 1;
    }
    for hand in hands.iter() {
        assert!(hand.size() == 24);
        for card in hand.list().iter() {
            count[idx_from_id(card.id()) as usize] += 1;
        }
    }

    for c in count.iter() {
        assert!(*c == 1);
    }

}

#[cfg(test)]
fn idx_from_id(id: u32) -> u32 {
    if id < 66 {
        id
    } else {
        id - 4
    }
}
