//! Module for the card deal, after auctions are complete.
use std::fmt;

use super::bid;
use super::cards;
use super::points;
use super::pos;
use super::trick;

/// Describes the state of a coinche deal, ready to play a card.
#[derive(Clone)]
pub struct DealState {
    players: [cards::Hand; super::NB_PLAYERS],
    partner: pos::PlayerPos, 
    called_king: Option<cards::Card>,
    dog: cards::Hand,
    current: pos::PlayerPos,
    contract: bid::Contract,
    points: [f32; super::NB_PLAYERS],
    oudlers_count: u8,
    tricks: Vec<trick::Trick>,
}

/// Result of a deal.
#[derive(PartialEq, Debug)]
pub enum DealResult {
    /// The deal is still playing
    Nothing,

    /// The deal is over
    GameOver {
        /// Worth of won tricks
        points: [f32; super::NB_PLAYERS],
        /// Winning team
        taker_won: bool,
        /// Score for this deal
        scores: [f32; super::NB_PLAYERS],
    },
}

/// Result of a trick
#[derive(PartialEq, Debug)]
pub enum TrickResult {
    Nothing,
    TrickOver(pos::PlayerPos, DealResult),
}

/// Error that can occur during play
#[derive(PartialEq, Debug)]
pub enum PlayError {
    /// A player tried to act before his turn
    TurnError,
    /// A player tried to play a card he doesn't have
    CardMissing,
    /// A player tried to play the wrong suit, while he still have some
    IncorrectSuit,
    /// A player tried to play the wrong suit, while he still have trumps
    InvalidPiss,
    /// A player did not raise on the last played trump
    NonRaisedTrump,

    /// No last trick is available for display
    NoLastTrick,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PlayError::TurnError => write!(f, "invalid turn order"),
            PlayError::CardMissing => write!(f, "you can only play cards you have"),
            PlayError::IncorrectSuit => write!(f, "wrong suit played"),
            PlayError::InvalidPiss => write!(f, "you must use trumps"),
            PlayError::NonRaisedTrump => write!(f, "too weak trump played"),
            PlayError::NoLastTrick => write!(f, "no trick has been played yet"),
        }
    }
}

impl DealState {
    /// Creates a new DealState, with the given cards, first player and contract.
    pub fn new(first: pos::PlayerPos, hands: [cards::Hand; super::NB_PLAYERS], dog: cards::Hand, contract: bid::Contract, partner: pos::PlayerPos) -> Self {
        DealState {
            players: hands,
            partner,
            called_king: None,
            dog,
            current: first,
            contract,
            tricks: vec![trick::Trick::new(first)],
            oudlers_count: 0,
            points: [0.0; 5],
        }
    }

    /// Returns the contract used for this deal
    pub fn contract(&self) -> &bid::Contract {
        &self.contract
    }

    /// Returns the dog
    pub fn dog(&self) -> cards::Hand {
        self.dog
    }

    //TODO return Result instead of bool
    pub fn call_king(&mut self, pos: pos::PlayerPos, card: cards::Card) -> bool {
        if pos != self.contract.author {
            println!("Player {:?} is not the taker", pos);
            return false;
        } 

        let hand = self.players[pos as usize];
        let has_all_kings = hand.has_all_rank(cards::Rank::RankK);
        let has_all_queens = hand.has_all_rank(cards::Rank::RankQ);

        if card.rank() == cards::Rank::RankJ && !(has_all_queens && has_all_kings) {
            println!("Calling a jake not allowed");
            return false;
        }
        if card.rank() == cards::Rank::RankQ && !has_all_kings {
            println!("Calling a queen not allowed");
            return false;
        }

        //Everything ok, now who is the partner ?
        self.partner = pos; // the taker by default (if king is in the dog..) 
        for player_pos in &[
            pos::PlayerPos::P0,
            pos::PlayerPos::P1,
            pos::PlayerPos::P2,
            pos::PlayerPos::P3,
            pos::PlayerPos::P4,
        ] {
            if self.players[*player_pos as usize].has(card) {
                self.partner = *player_pos;
            }
        }

        //King have been called successfully
        self.called_king = Some(card);
        true
    }

    /// Make the dog
    //TODO return Result instead of bool
    pub fn make_dog(&mut self, pos: pos::PlayerPos, cards: cards::Hand) -> bool {
        if pos != self.contract.author {
            println!("Player {:?} is not the taker", pos);
            return false;
        } 
        let cards_list = cards.list();
        if cards_list.len() != super::DOG_SIZE {
            println!("Wrong number of cards: {} instead of {}", cards_list.len(), super::DOG_SIZE);
            return false;
        }

        let mut taker_cards = self.players[pos as usize].clone();
        taker_cards.merge(self.dog);
        let mut new_dog = cards::Hand::new();
        for card in cards_list {
            if new_dog.has(card) {
                println!("Can't put the same card ({}) twice in the dog", card.to_string());
                return false;
            }
            if !taker_cards.has(card) {
                println!("{} is neither in the taker's hand nor in the dog", card.to_string());
                return false;
            }

            taker_cards.remove(card);
            new_dog.add(card);
        }
        //Dog successfully made
        self.dog = new_dog;
        self.players[pos as usize] = taker_cards;
        true
    } 

    /// Try to play a card
    pub fn play_card(
        &mut self,
        player: pos::PlayerPos,
        card: cards::Card,
    ) -> Result<TrickResult, PlayError> {
        if self.current != player {
            return Err(PlayError::TurnError);
        }

        // Is that a valid move?
        can_play(
            player,
            card,
            self.players[player as usize],
            self.current_trick(),
        )?;

        // Play the card
        let trick_over = self.current_trick_mut().play_card(player, card);

        // Remove card from player hand
        self.players[player as usize].remove(card);

        // Is the trick over?
        let result = if trick_over {
            let winner = self.current_trick().winner;

            let points = self.current_trick().points();
            self.points[winner as usize] += points;

            let (has_petit, has_21, has_excuse) = self.current_trick().clone().has_oudlers();
            if self.in_taker_team(winner) && (has_petit || has_21) {
                    self.oudlers_count += 1;
            }

            if has_excuse {
                let excuse = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22);
                let excuse_player = self.current_trick().player_played(excuse).unwrap();
                if self.tricks.len() == super::DEAL_SIZE && !self.is_slam() {
                    //Excuse played in the last trick when not a slam : goes to the other team
                    let excuse_points = points::points(excuse);
                    if self.in_taker_team(excuse_player) {
                        self.points[self.contract.author as usize] -= excuse_points;
                        self.points[*self.get_opponent() as usize] += excuse_points;
                    } else {
                        self.points[excuse_player as usize] -= excuse_points;
                        self.points[self.contract.author as usize] += excuse_points;
                        self.oudlers_count += 1;
                    }

                } else {
                    //player of the excuse keeps it
                      // points
                    let diff_points = points::points(excuse) - 0.5; 
                    self.points[winner as usize] -= diff_points;
                    self.points[excuse_player as usize] += diff_points;
                      // oudlers count
                    if self.in_taker_team(excuse_player) {
                        self.oudlers_count += 1;
                    }
                }
            }

            if self.tricks.len() == super::DEAL_SIZE {
                // TODO petit au bout ? -> maj annonce
            } else {
                self.tricks.push(trick::Trick::new(winner));
            }
            self.current = winner;
            TrickResult::TrickOver(winner, self.get_deal_result())
        } else {
            self.current = self.current.next();
            TrickResult::Nothing
        };

        Ok(result)
    }

    fn in_taker_team(&self, player: pos::PlayerPos) -> bool {
        &player == &self.contract.author || &player == &self.partner
    } 

    fn get_opponent(&self) -> &pos::PlayerPos {
        for position in pos::POSITIONS_LIST.iter() {
            if !self.in_taker_team(*position) {
                return position;
            }
        }
        &pos::PlayerPos::P0
    }

    /// Returns the player expected to play next.
    pub fn next_player(&self) -> pos::PlayerPos {
        self.current
    }

    pub fn get_deal_result(&self) -> DealResult {
        if !self.is_over() {
            return DealResult::Nothing;
        }

        let _slam = self.is_slam();

        let mut taking_points = self.points[self.contract.author as usize];
        if self.partner != self.contract.author {
            taking_points += self.points[self.partner as usize];
        }
        let base_points = self.contract.target.multiplier() as f32 * points::score(taking_points, self.oudlers_count);

        let mut scores = [0.0; super::NB_PLAYERS];
        for position in pos::POSITIONS_LIST.iter() {
            if !self.in_taker_team(*position) {
                scores[*position as usize] -= base_points;
                scores[self.contract.author as usize] += base_points;
            } else if position != &self.contract.author { // Partner
                scores[*position as usize] += base_points;
                scores[self.contract.author as usize] -= base_points;
            }
        }

        DealResult::GameOver {
            points: self.points,
            taker_won: base_points > 0.0,
            scores,
        }
    }

    fn is_slam(&self) -> bool {
        for trick in &self.tricks {
            if !self.in_taker_team(trick.winner) {
                return false;
            }
        }
        true
    }

    /// Returns the cards of all players
    pub fn hands(&self) -> [cards::Hand; super::NB_PLAYERS] {
        self.players
    }

    pub fn is_over(&self) -> bool {
        self.tricks.len() == super::DEAL_SIZE && !self.tricks[super::DEAL_SIZE -1].cards.iter().any(|&c| c.is_none())
    }

    /// Return the last trick, if possible
    pub fn last_trick(&self) -> Result<&trick::Trick, PlayError> {
        if self.tricks.len() == 1 {
            Err(PlayError::NoLastTrick)
        } else {
            let i = self.tricks.len() - 2;
            Ok(&self.tricks[i])
        }
    }

    /// Returns the current trick.
    pub fn current_trick(&self) -> &trick::Trick {
        let i = self.tricks.len() - 1;
        &self.tricks[i]
    }

    fn current_trick_mut(&mut self) -> &mut trick::Trick {
        let i = self.tricks.len() - 1;
        &mut self.tricks[i]
    }
}

/// Returns `true` if the move appear legal.
pub fn can_play(
    p: pos::PlayerPos,
    card: cards::Card,
    hand: cards::Hand,
    trick: &trick::Trick,
) -> Result<(), PlayError> {
    // First, we need the card to be able to play
    if !hand.has(card) {
        return Err(PlayError::CardMissing);
    }

    //Excuse
    if card.rank() == cards::Rank::Rank22 {
        return Ok(());
    }

    if p == trick.first {
        return Ok(());
    }

    let card_suit = card.suit();
    let starting_suit = trick.suit().unwrap();
    if card_suit != starting_suit {
        if hand.has_any(starting_suit) {
            return Err(PlayError::IncorrectSuit);
        }

        if card_suit != cards::Suit::Trump && hand.has_any(cards::Suit::Trump) {
            return Err(PlayError::InvalidPiss);
        }
    }

    // One must raise when playing trump
    if card_suit == cards::Suit::Trump {
        let highest = highest_trump(trick, p);
        if points::strength(card) < highest && has_higher_trump(hand, highest) {
            return Err(PlayError::NonRaisedTrump);
        }
    }

    Ok(())
}

fn has_higher_trump(hand: cards::Hand, strength: i32) -> bool {
    for c in hand.list() {
        if points::strength(c) > strength {
            return true;
        }
    }
    false
}

fn highest_trump(trick: &trick::Trick, player: pos::PlayerPos) -> i32 {
    let mut highest = -1;

    for p in trick.first.until(player) {
        if trick.cards[p as usize].unwrap().suit() == cards::Suit::Trump {
            let str = points::strength(trick.cards[p as usize].unwrap());
            if str > highest {
                highest = str;
            }
        }
    }

    highest
}

#[cfg(test)]
mod tests {
    use super::has_higher_trump;
    use super::*;
    use crate::{NB_PLAYERS, cards, points, pos};

    #[test]
    fn test_play_card() {
        let mut dog = cards::Hand::new();
        dog.add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank16));
        dog.add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank5));
        dog.add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank2));

        let mut hands = [cards::Hand::new(); NB_PLAYERS];
        hands[0].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8));
        hands[0].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank9));
        hands[0].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank7));
        hands[0].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank10));
        hands[0].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank1));
        hands[0].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank8));
        hands[0].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank9));
        hands[0].add(cards::Card::new(cards::Suit::Club, cards::Rank::RankJ));
        hands[0].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank6));
        hands[0].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank6));
        hands[0].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank18));
        hands[0].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank19));
        hands[0].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank20));
        hands[0].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank21));
        hands[0].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22));

        hands[1].add(cards::Card::new(cards::Suit::Club, cards::Rank::RankQ));
        hands[1].add(cards::Card::new(cards::Suit::Club, cards::Rank::RankK));
        hands[1].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank10));
        hands[1].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank1));
        hands[1].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank7));
        hands[1].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank8));
        hands[1].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank9));
        hands[1].add(cards::Card::new(cards::Suit::Spade, cards::Rank::RankJ));
        hands[1].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank2));
        hands[1].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank6));
        hands[1].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank3));
        hands[1].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank13));
        hands[1].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank14));
        hands[1].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank15));
        hands[1].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank17));

        hands[2].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank7));
        hands[2].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank8));
        hands[2].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank9));
        hands[2].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::RankJ));
        hands[2].add(cards::Card::new(cards::Suit::Spade, cards::Rank::RankQ));
        hands[2].add(cards::Card::new(cards::Suit::Spade, cards::Rank::RankK));
        hands[2].add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankQ));
        hands[2].add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankK));
        hands[2].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank3));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank2));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank7));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank9));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank10));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank11));
        hands[2].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank12));

        hands[3].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::RankQ));
        hands[3].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::RankK));
        hands[3].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank10));
        hands[3].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank1));
        hands[3].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank10));
        hands[3].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank1));
        hands[3].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank7));
        hands[3].add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankJ));
        hands[3].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank4));
        hands[3].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank2));
        hands[3].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1));
        hands[3].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank8));
        hands[3].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank3));
        hands[3].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank4));
        hands[3].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank5));

        hands[4].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank2));
        hands[4].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank3));
        hands[4].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank4));
        hands[4].add(cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank5));
        hands[4].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank2));
        hands[4].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank3));
        hands[4].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank2));
        hands[4].add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank3));
        hands[4].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank3));
        hands[4].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank4));
        hands[4].add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank5));
        hands[4].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank5));
        hands[4].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank4));
        hands[4].add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank6));
        hands[4].add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank6));

        let contract = bid::Contract {
            author: pos::PlayerPos::P0,
            target: bid::Target::Prise,
        };

        let mut deal = DealState::new(pos::PlayerPos::P0, hands, dog, contract, pos::PlayerPos::P2);

        // Wrong turn
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P1,
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank10)
            ).err(),
            Some(PlayError::TurnError)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P0,
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank7)
            ).ok(),
            Some(TrickResult::Nothing)
        );
        // Card missing
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P1,
                cards::Card::new(cards::Suit::Heart, cards::Rank::Rank7)
            ).err(),
            Some(PlayError::CardMissing)
        );
        // Wrong color
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P1,
                cards::Card::new(cards::Suit::Spade, cards::Rank::Rank7)
            ).err(),
            Some(PlayError::IncorrectSuit)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P1,
                cards::Card::new(cards::Suit::Club, cards::Rank::RankQ)
            ).ok(),
            Some(TrickResult::Nothing)
        );
        // Invalid piss
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P2,
                cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank7)
            ).err(),
            Some(PlayError::InvalidPiss)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P2,
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank2)
            ).ok(),
            Some(TrickResult::Nothing)
        );
        // UnderTrump
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P3,
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1)
            ).err(),
            Some(PlayError::NonRaisedTrump)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P3,
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank8)
            ).ok(),
            Some(TrickResult::Nothing)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::P4,
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank4)
            ).ok(),
            Some(TrickResult::TrickOver(
                pos::PlayerPos::P3,
                deal.get_deal_result()
            ))
        );
    }

    #[test]
    fn test_has_higher_1() {
        // Simple case
        let mut hand = cards::Hand::new();

        hand.add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8));
        hand.add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank6));
        assert!(has_higher_trump(
            hand,
            points::strength(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank4))
        ));
    }

    #[test]
    fn test_has_higher_2() {
        // Test that we don't mix colors
        let mut hand = cards::Hand::new();

        hand.add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8));
        hand.add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank1));
        assert!(!has_higher_trump(
            hand,
            points::strength(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank5))
        ));
    }

    #[test]
    fn test_has_higher_3() {
        let mut hand = cards::Hand::new();

        hand.add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankJ));
        hand.add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank2));
        assert!(!has_higher_trump(
            hand,
            points::strength(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank3))
        ));
    }

    #[test]
    fn test_has_higher_4() {
        let mut hand = cards::Hand::new();

        hand.add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8));
        hand.add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22));
        assert!(!has_higher_trump(
            hand,
            points::strength(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank3))
        ));
    }

    #[test]
    fn test_has_higher_5() {
        // Test when we have no trump at all
        let mut hand = cards::Hand::new();

        hand.add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankJ));
        hand.add(cards::Card::new(cards::Suit::Diamond, cards::Rank::RankJ));
        hand.add(cards::Card::new(cards::Suit::Spade, cards::Rank::RankJ));
        assert!(!has_higher_trump(
            hand,
            points::strength(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank7))
        ));
    }
}

#[cfg(feature = "use_bench")]
mod benchs {
    use deal_seeded_hands;
    use test::Bencher;

    use super::*;
    use {bid, cards, pos};

    #[bench]
    fn bench_can_play(b: &mut Bencher) {
        fn try_deeper(deal: &DealState, depth: usize) {
            let player = deal.next_player();
            for c in deal.hands()[player as usize].list() {
                let mut new_deal = deal.clone();
                match new_deal.play_card(player, c) {
                    Ok(_) => {
                        if depth > 0 {
                            try_deeper(&new_deal, depth - 1);
                        }
                    }
                    _ => (),
                };
            }
        }

        let seed = &[3, 32, 654, 1, 844];
        let hands = deal_seeded_hands(seed);
        let deal = DealState::new(
            pos::PlayerPos::P0,
            hands,
            bid::Contract {
                author: pos::PlayerPos::P0,
                trump: cards::Suit::Heart,
                target: bid::Target::Contract80,
                coinche_level: 0,
            },
        );
        b.iter(|| try_deeper(&deal, 4));
    }
}
