//! This module implements a trick in a game of coinche.

use serde::{Serialize, Deserialize};

use super::cards;
use super::points;
use super::pos;

/// The current cards on the table.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Trick {
    /// Cards currently on the table (they are `None` until played).
    pub cards: [Option<cards::Card>; 4],
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
            cards: [None; 4],
        }
    }

    /// Creates a default trick
    pub fn default() -> Self {
        let default = pos::PlayerPos::P0;
        Trick {
            first: default,
            winner: default,
            cards: [None; 4],
        }
    }

    /// Returns the points value of this trick.
    pub fn score(&self, trump: cards::Suit) -> i32 {
        self.cards
            .iter()
            .map(|c| c.map_or(0, |c| points::score(c, trump)))
            .sum()
    }

    pub fn card_played(&self, pos: pos::PlayerPos) -> Option<cards::Card> {
        let first_pos = self.first.to_n();
        let player_pos = pos.to_n();
        let trick_pos = if player_pos < first_pos {
            player_pos + 4 - first_pos
        } else {
            player_pos - first_pos
        };
        self.cards[trick_pos]
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
        trump: cards::Suit,
    ) -> bool {
        self.cards[player as usize] = Some(card);
        if player == self.first {
            return false;
        }

        if points::strength(card, trump)
            > points::strength(self.cards[self.winner as usize].unwrap(), trump)
        {
            self.winner = player
        }

        (player == self.first.prev())
    }

    /// Returns the starting suit for this trick.
    ///
    /// Returns `None` if the trick hasn't started yet.
    pub fn suit(&self) -> Option<cards::Suit> {
        self.cards[self.first as usize].map(|c| c.suit())
    }
}
