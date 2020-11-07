//! This module implements a trick in a game of coinche.

use serde::{Serialize, Deserialize};

use super::cards;
use super::points;
use super::pos;


const MAX_PLAYERS: usize = 5;

/// The current cards on the table.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Trick {
    /// Cards currently on the table (they are `None` until played).
    pub cards: [Option<cards::Card>; MAX_PLAYERS],
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
            cards: [None; MAX_PLAYERS],
        }
    }

    /// Creates a default trick
    pub fn default() -> Self {
        let default = pos::PlayerPos::from_n(0, 5);
        Trick {
            first: default,
            winner: default,
            cards: [None; MAX_PLAYERS],
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
    pub fn player_played(&self, card: cards::Card) -> Option<pos::AbsolutePos> {
        self.cards.iter().position(|c| c == &Some(card)).map(|idx| pos::PlayerPos::from_n(idx, MAX_PLAYERS as u8).pos)
    }

    /// Returns `true` if `self` contains `card`.
    pub fn has(self, card: cards::Card) -> bool {
        self.cards.contains(&Some(card))
    }

    pub fn has_oudlers(self) -> (bool, bool, bool) {
        let petit = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1);
        let vingtetun = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank21);
        let excuse = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22);
        (
            self.cards.contains(&Some(petit)),
            self.cards.contains(&Some(vingtetun)),
            self.cards.contains(&Some(excuse))
        )
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
        self.cards[player.pos as usize] = Some(card);
        if player == self.first {
            return false;
        }

        let winner_card = self.cards[self.winner.pos as usize].unwrap();
        if points::strength(winner_card) == 0 || // when the Excuse is played by the first player
           (  points::strength(card) > points::strength(winner_card)
           && (card.suit() == winner_card.suit() || card.suit() == cards::Suit::Trump )
           )
        {
            self.winner = player
        }

        player == self.first.prev()
    }

    /// Returns the starting suit for this trick.
    ///
    /// Returns `None` if the trick hasn't started yet.
    pub fn suit(&self) -> Option<cards::Suit> {
        // self.cards[self.first as usize].map(|c| c.suit())
        if let Some(first_card) = self.cards[self.first.pos as usize]{
            if first_card.rank() == cards::Rank::Rank22 {
                // first card is the Excuse : we look at the second card played
                return self.cards[self.first.next().pos as usize].map(|c| c.suit())
            } else {
                return Some(first_card.suit())
            }
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cards, pos};

    #[test]
    fn test_play_card() {
        let mut trick = Trick::default();
        trick.play_card(
            pos::PlayerPos::from_n(0, 5),
            cards::Card::new(cards::Suit::Club, cards::Rank::Rank5)
        );
        assert_eq!( trick.winner, pos::PlayerPos::from_n(0, 5));

        //Higher card
        trick.play_card(
            pos::PlayerPos::from_n(1, 5),
            cards::Card::new(cards::Suit::Club, cards::Rank::Rank8)
        );
        assert_eq!( trick.winner, pos::PlayerPos::from_n(1, 5));

        //Higher rank bug wrong color
        trick.play_card(
            pos::PlayerPos::from_n(2, 5),
            cards::Card::new(cards::Suit::Heart, cards::Rank::Rank10)
        );
        assert_eq!( trick.winner, pos::PlayerPos::from_n(1, 5));
    }
}
