//! Module for the card deal, after auctions are complete.
use std::fmt;

use serde::{Deserialize, Serialize};

use super::bid;
use super::cards;
use super::points;
use super::pos;
use super::trick;
use super::Announce;
use super::AnnounceType;

/// Describes the state of a coinche deal, ready to play a card.
#[derive(Serialize, Deserialize, Clone)]
pub struct DealState {
    players: Vec<cards::Hand>,
    partner: pos::PlayerPos,
    called_king: Option<cards::Card>,
    dog: cards::Hand,
    current: pos::PlayerPos,
    contract: bid::Contract,
    points: Vec<f32>,
    oudlers_count: u8,
    petit_au_bout: Option<pos::PlayerPos>,
    tricks: Vec<trick::Trick>,
    pub announces: Vec<Vec<AnnounceType>>,
}

impl fmt::Display for DealState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(king) = self.called_king {
            writeln!(f, "called king : {}", king.to_string())?;
        }
        writeln!(f, "dog : {}", self.dog.to_string())?;
        writeln!(f, "taker : {:?}", self.contract().author.pos)?;
        writeln!(f, "partner : {:?}", self.partner.pos)?;
        // for trick in self.tricks {
        //     writeln!(f,  "------------")?;
        // }
        write!(f, "------------")
    }
}

/// Result of a deal.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum DealResult {
    /// The deal is still playing
    Nothing,

    /// The deal is over
    GameOver {
        /// Worth of won tricks
        points: Vec<f32>,
        /// Winning team
        taker_diff: f32,
        oudlers_count: u8,
        petit_bonus: f32,
        multiplier: i32,
        slam_bonus: f32,
        poignees_bonus: f32,
        /// Score for this deal
        scores: Vec<f32>,
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
    /// The first player tried to play the suit of the called king in the first trick
    CallKingSuit,
    /// No last trick is available for display
    NoLastTrick,

    DogNotTaker,
    DogWrongNumberOfCards(usize, usize),
    DogSameCardTwice(cards::Card),
    DogCardNotFound(cards::Card),
    DogOudler(cards::Card),
    DogKing(cards::Card),
    DogTrump(cards::Card),
    InvalidAnnounce,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PlayError::TurnError => write!(f, "invalid turn order"),
            PlayError::CardMissing => write!(f, "you can only play cards you have"),
            PlayError::IncorrectSuit => write!(f, "wrong suit played"),
            PlayError::InvalidPiss => write!(f, "you must use trumps"),
            PlayError::NonRaisedTrump => write!(f, "too weak trump played"),
            PlayError::CallKingSuit => write!(
                f,
                "you cannot play the suit of the called king in the first trick"
            ),
            PlayError::NoLastTrick => write!(f, "no trick has been played yet"),
            PlayError::DogNotTaker => write!(f, "you are not the taker"),
            PlayError::DogWrongNumberOfCards(_wrong, _right) => write!(f, "Wrong number of cards"),
            PlayError::DogSameCardTwice(_card) => {
                write!(f, "Can't put the same card twice in the dog")
            }
            PlayError::DogCardNotFound(_card) => {
                write!(f, "Card neither in the taker's hand nor in the dog")
            }
            PlayError::DogOudler(_card) => write!(f, "Can't put an oudler in the dog"),
            PlayError::DogKing(_card) => write!(f, "Can't put a king in the dog"),
            PlayError::DogTrump(_card) => write!(f, "Can't put a trump in the dog"),
            PlayError::InvalidAnnounce => write!(f, "Invalid announce"),
            // PlayError::DogWrongNumberOfCards(wrong, right) => write!(f, "Wrong number of cards: {} instead of {}", wrong, right),
            // PlayError::DogSameCardTwice(card) => write!(f, "Can't put the same card ({}) twice in the dog", card.to_string()),
            // PlayError::DogCardNotFound(card) => write!(f, "{} is neither in the taker's hand nor in the dog", card.to_string()),
            // PlayError::DogOudler(card) => write!(f, "Can't put an oudler ({}) in the dog", card.to_string()),
            // PlayError::DogKing(card) => write!(f, "Can't put a king ({}) in the dog", card.to_string()),
            // PlayError::DogTrump(card) => write!(f, "Can't put a trump ({}) in the dog", card.to_string()),
        }
    }
}

impl DealState {
    /// Creates a new DealState, with the given cards, first player and contract.
    pub fn new(
        deal_first: pos::PlayerPos,
        hands: Vec<cards::Hand>,
        dog: cards::Hand,
        contract: bid::Contract,
        partner: pos::PlayerPos,
    ) -> Self {
        let count = hands.len();
        let first = if contract.slam
            && (contract.target == bid::Target::GardeContre
                || contract.target == bid::Target::GardeSans)
        {
            contract.author
        } else {
            deal_first // if contract with dog, the slam is checked later and updates the first player with the set_first_player function
        };

        DealState {
            players: hands,
            partner,
            called_king: None,
            dog,
            current: first,
            contract,
            tricks: vec![trick::Trick::new(first)],
            oudlers_count: 0,
            petit_au_bout: None,
            points: vec![0.0; count],
            announces: vec![vec![]; count],
        }
    }

    pub fn get_tricks_count(&self) -> usize {
        self.tricks.len()
    }

    // Set the first player at the beginning of a deal
    // only used when a slam is announced to override default
    pub fn set_first_player(&mut self, first: pos::PlayerPos) {
        self.current = first;
        self.tricks = vec![trick::Trick::new(first)];
    }

    /// Returns the contract used for this deal
    pub fn contract(&self) -> &bid::Contract {
        &self.contract
    }

    /// Returns the dog
    pub fn dog(&self) -> cards::Hand {
        self.dog
    }

    /// Returns the called king
    pub fn king(&self) -> Option<cards::Card> {
        self.called_king
    }

    /// Returns the partner
    pub fn partner(&self) -> pos::PlayerPos {
        self.partner
    }

    //TODO return Result instead of bool
    pub fn call_king(&mut self, pos: pos::PlayerPos, card: cards::Card) -> bool {
        if pos != self.contract.author {
            println!("Player {:?} is not the taker", pos);
            return false;
        }

        let hand = self.players[pos.pos as usize];
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
            pos::PlayerPos::from_n(0, 5),
            pos::PlayerPos::from_n(1, 5),
            pos::PlayerPos::from_n(2, 5),
            pos::PlayerPos::from_n(3, 5),
            pos::PlayerPos::from_n(4, 5),
        ] {
            if self.players[player_pos.pos as usize].has(card) {
                self.partner = *player_pos;
            }
        }

        //King have been called successfully
        self.called_king = Some(card);
        true
    }

    /// Make the dog
    pub fn make_dog(
        &mut self,
        pos: pos::PlayerPos,
        cards: cards::Hand,
        slam: bool,
    ) -> Result<(), PlayError> {
        if pos != self.contract.author {
            return Err(PlayError::DogNotTaker);
        }
        let cards_list = cards.list();
        let dog_size = super::dog_size(self.players.len());
        if cards_list.len() != dog_size {
            return Err(PlayError::DogWrongNumberOfCards(cards_list.len(), dog_size));
        }

        let mut taker_cards = self.players[pos.pos as usize].clone();
        taker_cards.merge(self.dog);
        let mut new_dog = cards::Hand::new();
        for card in cards_list {
            if new_dog.has(card) {
                return Err(PlayError::DogSameCardTwice(card));
            }
            if !taker_cards.has(card) {
                return Err(PlayError::DogCardNotFound(card));
            }
            if card.is_oudler() {
                return Err(PlayError::DogOudler(card));
            }
            if card.rank() == cards::Rank::RankK {
                return Err(PlayError::DogKing(card));
            }
            if card.suit() == cards::Suit::Trump {
                //Check if there is no alternative (taker has only trumps and kings)
                if taker_cards
                    .list()
                    .iter()
                    .filter(|tcard| {
                        tcard.rank() != cards::Rank::RankK && tcard.suit() != cards::Suit::Trump
                    })
                    .peekable()
                    .peek()
                    .is_some()
                {
                    return Err(PlayError::DogTrump(card));
                }
            }
            taker_cards.remove(card);
            new_dog.add(card);
        }
        //Dog successfully made
        self.contract.slam = slam;
        if slam {
            // The taker is the first to play if he asked a slam
            self.set_first_player(self.contract().author);
        }
        self.dog = new_dog;
        self.players[pos.pos as usize] = taker_cards;
        Ok(())
    }

    /// Try to declare an announce
    pub fn announce(
        &mut self,
        player: pos::PlayerPos,
        announce: Announce,
    ) -> Result<(), PlayError> {
        if self.current != player {
            return Err(PlayError::TurnError);
        }
        if let Some(proof) = announce.proof {
            let hand = self.players[player.pos as usize];
            if announce.atype.check(hand, proof) {
                self.announces[player.pos as usize].push(announce.atype);
                return Ok(());
            }
        }
        Err(PlayError::InvalidAnnounce)
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

        let is_first_trick = self.tricks.len() == 1;
        let deal_size = super::deal_size(self.players.len());

        // Is that a valid move?
        can_play(
            player,
            card,
            self.players[player.pos as usize],
            self.current_trick(),
            self.called_king,
            is_first_trick,
        )?;

        // Play the card
        let trick_over = self.current_trick_mut().play_card(player, card);

        // Remove card from player hand
        self.players[player.pos as usize].remove(card);

        let result = if !trick_over {
            //Continue trick
            self.current = self.current.next();
            TrickResult::Nothing
        } else {
            // Trick finished
            let is_last_trick = self.tricks.len() == deal_size;

            let excuse = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22);
            // Special case : this is a slam and the taker played the excuse at the last trick
            let is_excuse_slam = if is_last_trick {
                let won_until_last = self
                    .tricks
                    .split_last()
                    .unwrap() // We can unwrap because tricks have been played (last trick)
                    .1
                    .iter() // get all tricks but the last
                    .filter(|trick| self.in_taker_team(trick.winner))
                    .count();
                won_until_last == deal_size - 1
                    && self.current_trick().player_played(excuse) == Some(self.contract.author.pos)
            } else {
                false
            };
            if is_excuse_slam {
                self.current_trick_mut().winner = self.contract.author;
            }

            let winner = self.current_trick().winner;

            let points = self.current_trick().points();
            self.points[winner.pos as usize] += points;

            let (has_petit, has_21, has_excuse) = self.current_trick().clone().has_oudlers();
            if self.in_taker_team(winner) {
                if has_petit {
                    self.oudlers_count += 1;
                }
                if has_21 {
                    self.oudlers_count += 1;
                }
            }

            if has_excuse {
                let count = self.players.len() as u8;
                let excuse_player = pos::PlayerPos::from_n(
                    self.current_trick().player_played(excuse).unwrap() as usize,
                    count,
                );
                if is_last_trick && !is_excuse_slam {
                    //Excuse played in the last trick when not a slam : goes to the other team
                    let excuse_points = points::points(excuse);
                    if self.in_taker_team(excuse_player) {
                        let opponent_pos = self.get_opponent().pos as usize;
                        self.points[self.contract.author.pos as usize] -= excuse_points;
                        self.points[opponent_pos] += excuse_points;
                    } else {
                        self.points[excuse_player.pos as usize] -= excuse_points;
                        self.points[self.contract.author.pos as usize] += excuse_points;
                        self.oudlers_count += 1;
                    }
                } else {
                    //player of the excuse keeps it
                    // points
                    let diff_points = points::points(excuse) - 0.5; // half a point for the pip exchange card
                    self.points[winner.pos as usize] -= diff_points;
                    self.points[excuse_player.pos as usize] += diff_points;
                    // oudlers count
                    if self.in_taker_team(excuse_player) {
                        self.oudlers_count += 1;
                    }
                }
            }

            if is_last_trick {
                if has_petit {
                    self.petit_au_bout = Some(winner);
                }
                // XXX : ici pour bénéficier de la mutabilité de self
                if self.contract.target == bid::Target::GardeSans {
                    self.oudlers_count += self.dog.count_oudlers();
                }
            } else {
                self.tricks.push(trick::Trick::new(winner));
            }
            self.current = winner;
            TrickResult::TrickOver(winner, self.get_deal_result())
        };

        Ok(result)
    }

    fn in_taker_team(&self, player: pos::PlayerPos) -> bool {
        &player == &self.contract.author || &player == &self.partner
    }

    fn get_opponent(&self) -> pos::PlayerPos {
        let count = self.players.len() as u8;
        for position in 0..=count {
            let candidate = pos::PlayerPos::from_n(position as usize, count);
            if !self.in_taker_team(candidate) {
                return candidate;
            }
        }
        pos::PlayerPos::from_n(0, count)
    }

    /// Returns the player expected to play next.
    pub fn next_player(&self) -> pos::PlayerPos {
        self.current
    }

    pub fn get_deal_result(&self) -> DealResult {
        if !self.is_over() {
            return DealResult::Nothing;
        }

        let mut taking_points = self.points[self.contract.author.pos as usize];
        if self.players.len() == 5 && self.partner != self.contract.author {
            taking_points += self.points[self.partner.pos as usize];
        }
        if self.contract.target != bid::Target::GardeContre {
            taking_points += points::hand_points(self.dog);
        }

        //Score : taker_diff +- 25
        let (taker_diff, score) = points::score(taking_points, self.oudlers_count);
        let taker_won = taker_diff >= 0.0;
        let petit_bonus = self.petit_au_bout_bonus();
        let multiplier = self.contract.target.multiplier();
        let mut base_points = multiplier as f32 * (score + petit_bonus);
        // other bonuses not multiplied by the contract level
        let slam_bonus = self.slam_bonus();
        let poignees_bonus = self.poignees_bonus(taker_won);
        base_points = base_points + slam_bonus + poignees_bonus;

        let count = self.players.len() as u8;
        let mut scores = vec![0.0; count as usize];
        for position in 0..count {
            if !self.in_taker_team(pos::PlayerPos::from_n(position as usize, count)) {
                scores[position as usize] -= base_points;
                scores[self.contract.author.pos as usize] += base_points;
            } else if position != self.contract.author.pos as u8 {
                // Partner
                scores[position as usize] += base_points;
                scores[self.contract.author.pos as usize] -= base_points;
            }
        }

        DealResult::GameOver {
            oudlers_count: self.oudlers_count,
            points: self.points.clone(),
            taker_diff,
            petit_bonus,
            multiplier,
            slam_bonus,
            poignees_bonus,
            scores,
        }
    }

    fn slam_bonus(&self) -> f32 {
        if self.contract.slam {
            // Slam announced
            if self.is_slam() {
                400.0
            } else {
                -200.0
            }
        } else if self.is_slam() {
            200.0 // Slam not announced
        } else {
            0.0
        }
    }

    fn poignees_bonus(&self, taker_won: bool) -> f32 {
        // All announces points go to the deal winner
        let points = self
            .announces
            .iter()
            .flatten()
            .map(|ann| ann.points())
            .sum();
        if taker_won {
            points
        } else {
            0.0 - points
        }
    }

    fn petit_au_bout_bonus(&self) -> f32 {
        if let Some(petit_player) = self.petit_au_bout {
            if self.in_taker_team(petit_player) {
                10.0
            } else {
                -10.0
            }
        } else {
            0.0 //Default : no petit au bout = 0 points
        }
    }

    fn taker_team_won_count(&self) -> usize {
        self.tricks
            .iter()
            .filter(|trick| self.in_taker_team(trick.winner))
            .count()
    }

    fn is_slam(&self) -> bool {
        let deal_size = super::deal_size(self.players.len());
        self.taker_team_won_count() == deal_size
    }

    /// Returns the cards of all players
    pub fn hands(&self) -> &Vec<cards::Hand> {
        &self.players
    }

    pub fn is_over(&self) -> bool {
        let nb_players = self.players.len();
        let deal_size = super::deal_size(nb_players);
        self.tricks.len() == deal_size
            && !self.tricks[deal_size - 1].cards[0..nb_players]
                .iter()
                .any(|&c| c.is_none())
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

    // XXX ugly hack to get the correct played cards for event dispatch after play_card (when the
    // new trick has already been initiated)
    // XXX only use this fuction on cloned states for dispatch !
    pub fn revert_trick(&mut self) {
        self.tricks.pop();
    }
}

/// Returns `true` if the move appear legal.
pub fn can_play(
    p: pos::PlayerPos,
    card: cards::Card,
    hand: cards::Hand,
    trick: &trick::Trick,
    called_king: Option<cards::Card>,
    is_first_trick: bool,
) -> Result<(), PlayError> {
    // First, we need the card to be able to play
    if !hand.has(card) {
        return Err(PlayError::CardMissing);
    }

    //Excuse
    if card.rank() == cards::Rank::Rank22 {
        return Ok(());
    }

    let card_suit = card.suit();
    if let Some(starting_suit) = trick.suit() {
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
    } else {
        //First to play (or second if the first played the Excuse)
        // Everything is accepted except a card in the suit of the called king at the
        // first trick of the deal if it is not the king itself
        if called_king.is_some()                          // A king has been called (5 players variant)
            && is_first_trick                             // and this is the first trick of the deal
            && card.suit() == called_king.unwrap().suit() // and the card played is in the suit of the called king
            && card.rank() != cards::Rank::RankK
        {
            // and this is not the king itself
            return Err(PlayError::CallKingSuit);
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
        if trick.cards[p.pos as usize].unwrap().suit() == cards::Suit::Trump {
            let str = points::strength(trick.cards[p.pos as usize].unwrap());
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
    use crate::{cards, points, pos};

    #[test]
    fn test_play_card() {
        let mut dog = cards::Hand::new();
        dog.add(cards::Card::new(cards::Suit::Trump, cards::Rank::Rank16));
        dog.add(cards::Card::new(cards::Suit::Heart, cards::Rank::Rank5));
        dog.add(cards::Card::new(cards::Suit::Club, cards::Rank::Rank2));

        let mut hands = [cards::Hand::new(); 5];
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
            author: pos::PlayerPos::from_n(0, 5),
            target: bid::Target::Prise,
            slam: false,
        };

        let mut deal = DealState::new(
            pos::PlayerPos::from_n(0, 5),
            hands.to_vec(),
            dog,
            contract,
            pos::PlayerPos::from_n(2, 5),
        );
        deal.call_king(
            pos::PlayerPos::from_n(0, 5),
            cards::Card::new(cards::Suit::Diamond, cards::Rank::RankK),
        );

        // Wrong turn
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(1, 5),
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank10)
            )
            .err(),
            Some(PlayError::TurnError)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(0, 5),
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank7)
            )
            .ok(),
            Some(TrickResult::Nothing)
        );
        // Card missing
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(1, 5),
                cards::Card::new(cards::Suit::Heart, cards::Rank::Rank7)
            )
            .err(),
            Some(PlayError::CardMissing)
        );
        // Wrong color
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(1, 5),
                cards::Card::new(cards::Suit::Spade, cards::Rank::Rank7)
            )
            .err(),
            Some(PlayError::IncorrectSuit)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(1, 5),
                cards::Card::new(cards::Suit::Club, cards::Rank::RankQ)
            )
            .ok(),
            Some(TrickResult::Nothing)
        );
        // Invalid piss
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(2, 5),
                cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank7)
            )
            .err(),
            Some(PlayError::InvalidPiss)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(2, 5),
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank2)
            )
            .ok(),
            Some(TrickResult::Nothing)
        );
        // UnderTrump
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(3, 5),
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1)
            )
            .err(),
            Some(PlayError::NonRaisedTrump)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(3, 5),
                cards::Card::new(cards::Suit::Trump, cards::Rank::Rank8)
            )
            .ok(),
            Some(TrickResult::Nothing)
        );
        assert_eq!(
            deal.play_card(
                pos::PlayerPos::from_n(4, 5),
                cards::Card::new(cards::Suit::Club, cards::Rank::Rank4)
            )
            .ok(),
            Some(TrickResult::TrickOver(
                pos::PlayerPos::from_n(3, 5),
                deal.get_deal_result()
            ))
        );
    }

    #[test]
    fn test_excuse_not_required_after_trump() {
        let first_player = pos::PlayerPos::from_n(1, 5);
        let heart = cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8);
        let excuse = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22);
        let trump = cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1);
        let king = cards::Card::new(cards::Suit::Diamond, cards::Rank::RankK);

        let mut hand = cards::Hand::new();
        hand.add(heart);
        hand.add(excuse);

        let trick = trick::Trick {
            cards: [None, Some(trump), None, None, None],
            first: first_player,
            winner: first_player,
        };

        assert!(!can_play(
            pos::PlayerPos::from_n(2, 5),
            heart,
            hand,
            &trick,
            Some(king),
            false,
        )
        .is_err());
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
