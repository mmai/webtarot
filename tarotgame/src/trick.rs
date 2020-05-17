//! This module implements a trick in a game of coinche.

use serde::{Serialize, Deserialize};

use super::cards;
use super::points;
use super::pos;

/// The current cards on the table.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Trick {
    /// Cards currently on the table (they are `None` until played).
    pub cards: [Option<cards::Card>; super::NB_PLAYERS],
    /// First player in this trick.
    pub first: pos::PlayerPos,
    /// Current winner of the trick (updated after each card played).
    pub winner: pos::PlayerPos,
}

impl Trick {
    /// Creates a new, empty trick.
    pub fn new(first: pos::PlayerPos) -> Self {
        Trick {
            first,
            winner: first,
            cards: [None; super::NB_PLAYERS],
        }
    }

    /// Creates a default trick
    pub fn default() -> Self {
        let default = pos::PlayerPos::P0;
        Trick {
            first: default,
            winner: default,
            cards: [None; super::NB_PLAYERS],
        }
    }

    /// Returns the points value of this trick.
    pub fn points(&self) -> f32 {
        self.cards
            .iter()
            .map(|c| c.map_or(0.0, |c| points::points(c)))
            .sum()
    }

    pub fn card_played(&self, pos: pos::PlayerPos) -> Option<cards::Card> {
        self.cards[pos.to_n()]
        // let first_pos = self.first.to_n();
        // let player_pos = pos.to_n();
        // let trick_pos = if player_pos < first_pos {
        //     player_pos + 4 - first_pos
        // } else {
        //     player_pos - first_pos
        // };
        // self.cards[trick_pos]
    }

    /// Returns the player who played a card
    pub fn player_played(&self, card: cards::Card) -> Option<pos::PlayerPos> {
        self.cards.iter().position(|c| c == &Some(card)).map(|idx| pos::PlayerPos::from_n(idx))
    }

    /// Returns `true` if `self` contains `card`.
    pub fn has(self, card: cards::Card) -> bool {
        self.cards.contains(&Some(card))
    }


    /// Plays a card.
    ///
    /// Updates the winner.
    ///
    /// Returns `true` if this completes the trick.
    pub fn play_card(
        &mut self,
        player: pos::PlayerPos,
        card: cards::Card,
    ) -> bool {
        self.cards[player as usize] = Some(card);
        if player == self.first {
            return false;
        }

        if points::strength(card)
            > points::strength(self.cards[self.winner as usize].unwrap())
        {
            self.winner = player
        }

        player == self.first.prev()
    }

    /// Returns the starting suit for this trick.
    ///
    /// Returns `None` if the trick hasn't started yet.
    pub fn suit(&self) -> Option<cards::Suit> {
        self.cards[self.first as usize].map(|c| c.suit())
    }
}
