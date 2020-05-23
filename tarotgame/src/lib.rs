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
//!     let first = pos::PlayerPos::P0;
//!
//!     // Start the first phase with an auction
//!     let mut auction = bid::Auction::new(first);
//!
//!     // Check their cards
//!     let hands = auction.hands();
//!
//!     // Players bid or pass
//!     auction.bid(pos::PlayerPos::P0, bid::Target::Garde).unwrap();
//!     auction.pass(pos::PlayerPos::P1).unwrap();
//!     auction.pass(pos::PlayerPos::P2).unwrap();
//!     auction.pass(pos::PlayerPos::P3).unwrap();
//!     // The result is `Over` when the auction is ready to complete
//!     match auction.pass(pos::PlayerPos::P4) {
//!         Ok(bid::AuctionState::Over) => (),
//!         _ => panic!("Should not happen"),
//!     };
//!
//!     // Complete the auction to enter the second phase
//!     let mut deal = auction.complete().unwrap();
//!
//!     // Play some cards
//!     deal.play_card(pos::PlayerPos::P0, hands[0].get_card());
//!     // ...
//! }
//! ```
#[macro_use]

#[cfg(feature = "use_bench")]
extern crate test;

pub mod bid;
pub mod cards;
pub mod deal;
pub mod points;
pub mod pos;
pub mod trick;

pub const NB_PLAYERS:usize = 5;
pub const DOG_SIZE:usize = 3;
const DEAL_SIZE:usize = (78 - DOG_SIZE) / NB_PLAYERS ;

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
pub fn deal_hands() -> ([cards::Hand; NB_PLAYERS], cards::Hand) {
    let mut hands = [cards::Hand::new(); NB_PLAYERS];
    let mut dog = cards::Hand::new();

    let mut d = cards::Deck::new();
    d.shuffle();

    d.deal_each(&mut hands, 3);
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);

    (hands, dog)
}

/// Deal cards for 5 players deterministically.
pub fn deal_seeded_hands(seed: [u8; 32]) -> ([cards::Hand; NB_PLAYERS], cards::Hand) {
    let mut hands = [cards::Hand::new(); NB_PLAYERS];
    let mut dog = cards::Hand::new();

    let mut d = cards::Deck::new();
    d.shuffle_seeded(seed);

    d.deal_each(&mut hands, 3);
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);
    dog.add(d.draw());
    d.deal_each(&mut hands, 3);

    (hands, dog)
}

#[test]
fn test_deals() {
    let (hands, dog) = deal_hands();
    assert!(dog.size() == 3);

    let mut count = [0; 78];

    for card in dog.list().iter() {
        count[idx_from_id(card.id()) as usize] += 1;
    }
    for hand in hands.iter() {
        assert!(hand.size() == DEAL_SIZE);
        for card in hand.list().iter() {
            count[idx_from_id(card.id()) as usize] += 1;
        }
    }

    for c in count.iter() {
        assert!(*c == 1);
    }

    fn idx_from_id(id: u32) -> u32 {
        if id < 66 {
            id
        } else {
            id - 4
        }
    }
}

