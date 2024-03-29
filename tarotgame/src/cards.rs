//! This module represents a basic, rule-agnostic 78-cards system.

use rand::{thread_rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use std::num::Wrapping;
use std::str::FromStr;
use std::string::ToString;

/// One of the four Suits: Heart, Spade, Diamond, Club.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize, Eq, Hash)]
#[repr(u64)]
pub enum Suit {
    /// The suit of hearts.
    Heart = 1,
    /// The suit of spades.
    Spade = 1 << 14,
    /// The suit of diamonds.
    Diamond = 1 << 28,
    /// The suit of clubs.
    Club = 1 << 42,
    /// Trumps
    Trump = 1 << 56,
}

impl Suit {
    /// Returns the suit corresponding to the number:
    ///
    /// * `0` -> Heart
    /// * `1` -> Spade
    /// * `2` -> Diamond
    /// * `3` -> Club
    /// * `4` -> Trump
    ///
    /// # Panics
    ///
    /// If `n >= 5`.
    pub fn from_n(n: u32) -> Self {
        match n {
            0 => Suit::Heart,
            1 => Suit::Spade,
            2 => Suit::Diamond,
            3 => Suit::Club,
            4 => Suit::Trump,
            other => panic!("bad suit number: {}", other),
        }
    }

    pub fn to_n(self) -> usize {
        match self {
            Suit::Heart => 0,
            Suit::Spade => 1,
            Suit::Diamond => 2,
            Suit::Club => 3,
            Suit::Trump => 4,
        }.to_owned()
    }

    /// Returns a UTF-8 character representing the suit (♥, ♠, ♦ or ♣).
    pub fn to_string(self) -> String {
        match self {
            Suit::Heart => "♥",
            Suit::Spade => "♠",
            Suit::Diamond => "♦",
            Suit::Club => "♣",
            Suit::Trump => "T",
        }.to_owned()
    }

    /// Returns a character representing the given suit.
    pub fn to_locale_string(self, locale: &str) -> String {
        match locale {
            "fr" => {
                match self {
                    Suit::Trump => "A".to_owned(),
                    _ => self.to_string()
                }
            },
            _ => self.to_string()
        }
    }

    /// Returns a character representing the suit (H, S, D or C).
    pub fn to_safe_string(self) -> String {
        match self {
            Suit::Heart => "H",
            Suit::Spade => "S",
            Suit::Diamond => "D",
            Suit::Club => "C",
            Suit::Trump => "T",
        }.to_owned()
    }
}

impl FromStr for Suit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "♥" | "H" | "h" | "heart" | "Suit::Heart" | "Heart" => Ok(Suit::Heart),
            "♣" | "C" | "c" | "club" | "Suit::Club" | "Club" => Ok(Suit::Club),
            "♠" | "S" | "s" | "spade" | "Suit::Spade" | "Spade" => Ok(Suit::Spade),
            "♦" | "D" | "d" | "diamond" | "Suit::Diamond" | "Diamond" => Ok(Suit::Diamond),
            "T" | "A" | "t" | "trump" | "Suit::Trump" | "Trump" => Ok(Suit::Trump),
            _ => Err(format!("invalid suit: {}", s)),
        }
    }
}

/// Rank of a card in a suit.
#[derive(PartialEq, Clone, Copy, Debug, PartialOrd, Eq, Ord)]
#[repr(u64)]
pub enum Rank {
    Rank1 = 1,
    Rank2 = 1 << 1,
    Rank3 = 1 << 2,
    Rank4 = 1 << 3,
    Rank5 = 1 << 4,
    Rank6 = 1 << 5,
    Rank7 = 1 << 6,
    Rank8 = 1 << 7,
    Rank9 = 1 << 8,
    Rank10 = 1 << 9,
    RankJ = 1 << 10,
    RankC = 1 << 11,
    RankQ = 1 << 12,
    RankK = 1 << 13,

    Rank11 = 1 << 14,
    Rank12 = 1 << 15,
    Rank13 = 1 << 16,
    Rank14 = 1 << 17,
    Rank15 = 1 << 18,
    Rank16 = 1 << 19,
    Rank17 = 1 << 20,
    Rank18 = 1 << 21,
    Rank19 = 1 << 22,
    Rank20 = 1 << 23,
    Rank21 = 1 << 24,
    Rank22 = 1 << 25,
}

/// Bit RANK_MASK over all ranks for non trump suits.
const RANK_MASK: u64 = 16383; 

impl Rank {
    /// Returns the rank corresponding to the given number:
    ///
    ///
    /// # Panics
    ///
    /// If `n > 25`.
    pub fn from_n(n: u32) -> Self {
        match n {
            0 => Rank::Rank1,
            1 => Rank::Rank2,
            2 => Rank::Rank3,
            3 => Rank::Rank4,
            4 => Rank::Rank5,
            5 => Rank::Rank6,
            6 => Rank::Rank7,
            7 => Rank::Rank8,
            8 => Rank::Rank9,
            9 => Rank::Rank10,
            10 => Rank::RankJ,
            11 => Rank::RankC,
            12 => Rank::RankQ,
            13 => Rank::RankK,

            14 => Rank::Rank11,
            15 => Rank::Rank12,
            16 => Rank::Rank13,
            17 => Rank::Rank14,
            18 => Rank::Rank15,
            19 => Rank::Rank16,
            20 => Rank::Rank17,
            21 => Rank::Rank18,
            22 => Rank::Rank19,
            23 => Rank::Rank20,
            24 => Rank::Rank21,
            25 => Rank::Rank22,
            other => panic!("invalid rank number: {}", other),
        }
    }

    // Return the enum by its discriminant.
    fn from_discriminant(rank: u64) -> Self {
        match rank {
            1 => Rank::Rank1,
            2 => Rank::Rank2,
            4 => Rank::Rank3,
            8 => Rank::Rank4,
            16 => Rank::Rank5,
            32 => Rank::Rank6,
            64 => Rank::Rank7,
            128 => Rank::Rank8,
            256 => Rank::Rank9,
            512 => Rank::Rank10,

            1024 => Rank::RankJ,
            2048 => Rank::RankC,
            4096 => Rank::RankQ,
            8192 => Rank::RankK,

            16384 => Rank::Rank11,
            32768 => Rank::Rank12,
            65536 => Rank::Rank13,
            131072 => Rank::Rank14,
            262144 => Rank::Rank15,
            524288 => Rank::Rank16,
            1048576 => Rank::Rank17,
            2097152 => Rank::Rank18,
            4194304 => Rank::Rank19,
            8388608 => Rank::Rank20,
            16777216 => Rank::Rank21,
            33554432 => Rank::Rank22,
            other => panic!("invalid rank discrimant: {}", other),
        }
    }

    /// Returns a character representing the given rank.
    pub fn to_string(self) -> String {
        match self {
            Rank::Rank1 => "1",
            Rank::Rank2 => "2",
            Rank::Rank3 => "3",
            Rank::Rank4 => "4",
            Rank::Rank5 => "5",
            Rank::Rank6 => "6",
            Rank::Rank7 => "7",
            Rank::Rank8 => "8",
            Rank::Rank9 => "9",
            Rank::Rank10 => "10",
            Rank::RankJ => "J",
            Rank::RankC => "C",
            Rank::RankQ => "Q",
            Rank::RankK => "K",

            Rank::Rank11 => "11",
            Rank::Rank12 => "12",
            Rank::Rank13 => "13",
            Rank::Rank14 => "14",
            Rank::Rank15 => "15",
            Rank::Rank16 => "16",
            Rank::Rank17 => "17",
            Rank::Rank18 => "18",
            Rank::Rank19 => "19",
            Rank::Rank20 => "20",
            Rank::Rank21 => "21",
            Rank::Rank22 => "E",
        }.to_owned()
    }

    /// Returns a character representing the given rank.
    pub fn to_locale_string(self, locale: &str) -> String {
        match locale {
            "fr" => {
                match self {
                    Rank::RankQ => String::from("D"),
                    Rank::RankK => String::from("R"),
                    _ => self.to_string()
                }
            },
            _ => self.to_string()
        }
    }
}

impl FromStr for Rank {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "1" => Ok(Rank::Rank1),
            "2" => Ok(Rank::Rank2),
            "3" => Ok(Rank::Rank3),
            "4" => Ok(Rank::Rank4),
            "5" => Ok(Rank::Rank5),
            "6" => Ok(Rank::Rank6),
            "7" => Ok(Rank::Rank7),
            "8" => Ok(Rank::Rank8),
            "9" => Ok(Rank::Rank9),
            "10" => Ok(Rank::Rank10),
            "J" => Ok(Rank::RankJ),
            "C" => Ok(Rank::RankC),
            "Q" => Ok(Rank::RankQ),
            "K" => Ok(Rank::RankK),
            "11" => Ok(Rank::Rank11),
            "12" => Ok(Rank::Rank12),
            "13" => Ok(Rank::Rank13),
            "14" => Ok(Rank::Rank14),
            "15" => Ok(Rank::Rank15),
            "16" => Ok(Rank::Rank16),
            "17" => Ok(Rank::Rank17),
            "18" => Ok(Rank::Rank18),
            "19" => Ok(Rank::Rank19),
            "20" => Ok(Rank::Rank20),
            "21" => Ok(Rank::Rank21),
            "22" => Ok(Rank::Rank22),
            _ => Err(format!("invalid rank: {}", s)),
        }
    }
}


/// Represents a single card.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Card(u64, u32); // (pips & courts , trumps)


impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank().partial_cmp(&other.rank())
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.chars().collect::<Vec<_>>();
        let len = s.len();

        if len < 2 {
            return Err(format!("invalid card: {:?}", s));
        } 

        let str_suit: String = s[(len - 1)..].iter().collect();
        let str_rank: String = s[..(len - 1)].iter().collect();
        let suit = Suit::from_str(&str_suit)?;
        let rank = Rank::from_str(&str_rank)?;

        Ok(Card::new(suit, rank))
    }
}

impl Card {
    /// Returns the card id (from 0 to 77).
    pub fn id(self) -> u32 {
        let mut i = 0;
        let Card(mut v, mut t) = self;
        if t == 0 { //pips & courts
            while v != 0 {
                i += 1;
                v >>= 1;
            }
            i - 1
        } else { //trumps
            while t != 0 {
                i += 1;
                t >>= 1;
            }
            55 + i
        }
    }

    /// Returns the card corresponding to the given id.
    ///
    /// # Panics
    ///
    /// If `id >= 82` or 65 < id < 70
    pub fn from_id(id: u32) -> Self {
        if id < 56 {
            Card(1 << id, 0)
        } else if id < 66 {
            Card(0, 1 << (id - 56))
        } else if id < 70 {
            panic!("invalid card id");
        } else if id < 82 {
            Card(0, 1 << (id - 56))
        } else {
            panic!("invalid card id");
        }
    }

    /// Returns the card's rank.
    pub fn rank(self) -> Rank {
        let Card(v, t) = self;
        let suit = self.suit();
        if suit == Suit::Trump {
            Rank::from_discriminant(t as u64)
        } else {
            Rank::from_discriminant(v / suit as u64)
        }
    }

    /// Returns the card's suit.
    pub fn suit(self) -> Suit {
        let Card(n, t) = self;
        if t > 0 {
            Suit::Trump
        } else if n < Suit::Spade as u64 {
            Suit::Heart
        } else if n < Suit::Diamond as u64 {
            Suit::Spade
        } else if n < Suit::Club as u64 {
            Suit::Diamond
        } else {
            Suit::Club
        }
    }

    /// Returns a string representation of the card (ex: "7♦").
    pub fn to_string(self) -> String {
        let r = self.rank();
        let s = self.suit();
        r.to_string() + &s.to_string()
    }

    /// Returns a locale string representation of the card (ex: "7♦").
    pub fn to_locale_string(self, locale: &str) -> String {
        let r = self.rank();
        let s = self.suit();
        r.to_locale_string(locale) + &s.to_locale_string(locale)
    }

    /// Creates a card from the given suit and rank.
    pub fn new(suit: Suit, rank: Rank) -> Self {
        if suit == Suit::Trump {
            Card(0, rank as u32)
        } else {
            Card(suit as u64 * rank as u64, 0)
        }
    }

    pub fn is_oudler(self) -> bool {
        self.suit() == Suit::Trump && [Rank::Rank1, Rank::Rank21, Rank::Rank22].contains(&self.rank())
    }

    pub fn excuse() -> Card {
        Card::new(Suit::Trump, Rank::Rank22)
    }

}

/// Represents an unordered set of cards.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct Hand(u64, u32);

impl From<Vec<Card>> for Hand {
    fn from(cards: Vec<Card>) -> Hand {
        let mut hand = Self::new();
        for card in cards.into_iter() {
            hand.add(card);
        }
        hand
    }
}

impl Hand {
    /// Returns an empty hand.
    pub fn new() -> Self {
        Hand(0, 0)
    }

    /// Add `cards` to `self`.
    ///
    pub fn merge(&mut self, cards: Hand) -> &mut Hand {
        self.0 |= cards.0;
        self.1 |= cards.1;
        self
    }

    /// Add `card` to `self`.
    ///
    /// No effect if `self` already contains `card`.
    pub fn add(&mut self, card: Card) -> &mut Hand {
        self.0 |= card.0;
        self.1 |= card.1;
        self
    }

    /// Removes `card` from `self`.
    ///
    /// No effect if `self` does not contains `card`.
    pub fn remove(&mut self, card: Card) {
        self.0 &= !card.0;
        self.1 &= !card.1;
    }

    /// Remove all cards from `self`.
    pub fn clean(&mut self) {
        *self = Hand::new();
    }

    /// Returns `true` if `self` contains `card`.
    pub fn has(self, card: Card) -> bool {
        (self.0 & card.0) != 0 || (self.1 & card.1) != 0
    }

    /// Returns `true` if `self` contains all cards from a given rank
    pub fn has_all_rank(self, rank: Rank) -> bool {
        self.has(Card::new(Suit::Club, rank))
            && self.has(Card::new(Suit::Heart, rank))
            && self.has(Card::new(Suit::Diamond, rank))
            && self.has(Card::new(Suit::Spade, rank))
    }

    /// Returns `true` if the hand contains any card of the given suit.
    pub fn has_any(self, suit: Suit) -> bool {
        if suit == Suit::Trump {
            //We do not consider the excuse as a trump
            let mut without_excuse = self.clone();
            without_excuse.remove(Card::new(Suit::Trump, Rank::Rank22));
            without_excuse.1 != 0
        } else {
            self.0 & (RANK_MASK * suit as u64) != 0
        }
    }

    /// Returns `true` if `self` contains no card.
    pub fn is_empty(self) -> bool {
        self.0 == 0 && self.1 == 0
    }

    /// Returns a card from `self`.
    ///
    /// Returns an invalid card if `self` is empty.
    pub fn get_card(self) -> Card {
        if self.is_empty() {
            return Card(0, 0);
        }

        let Hand(h, t) = self;
        if h > 0 { // pips & courts
            // Finds the rightmost bit, shifted to the left by 1.
            // let n = 1 << (h.trailing_zeroes());
            let n = Wrapping(h ^ (h - 1)) + Wrapping(1);
            if n.0 == 0 {
                // We got an overflow. This means the desired bit it the leftmost one.
                Card::from_id(55)
            } else {
                // We just need to shift it back.
                Card(n.0 >> 1, 0)
            }
        } else { //Trumps
            let n = Wrapping(t ^ (t - 1)) + Wrapping(1);
            if n.0 == 0 {
                // We got an overflow. This means the desired bit it the leftmost one.
                Card::from_id(81)
            } else {
                // We just need to shift it back.
                Card(0, n.0 >> 1)
            }
        }
    }

    pub fn get_suit_cards(self, suit: &Suit) -> Vec<Card> {
        self.into_iter().filter(|c| &c.suit() == suit).collect()
    }

    /// Returns the cards contained in `self` as a `Vec`.
    pub fn list(self) -> Vec<Card> {
        let mut cards = Vec::new();
        let mut h = self;

        while !h.is_empty() {
            let c = h.get_card();
            h.remove(c);
            cards.push(c);
        }

        cards
    }

    pub fn suit_highest(self, suit: Suit) -> Option<Card>{
        let mut suit_cards = self.get_suit_cards(&suit);
        suit_cards.sort();
        suit_cards.last().map(|c| *c)
    }

    pub fn trump_highest(self) -> Option<Card> {
        //We do not consider the excuse as a trump
        let mut without_excuse = self.clone();
        without_excuse.remove(Card::new(Suit::Trump, Rank::Rank22));
        let mut suit_cards = without_excuse.get_suit_cards(&Suit::Trump);
        suit_cards.sort();
        suit_cards.last().map(|c| *c)
    }

    pub fn suit_lowest(self, suit: Suit) -> Option<Card>{
        let mut suit_cards = self.get_suit_cards(&suit);
        suit_cards.sort();
        suit_cards.first().map(|c| *c)
    }

    // given a card, returns the lowest card in hand beating it for the suit
    pub fn suit_lowest_over_card(self, suit: Suit, card: Card) -> Option<Card>{
        let mut suit_cards: Vec<Card> = self.get_suit_cards(&suit).into_iter().filter(|c| c.rank() > card.rank()).collect();
        suit_cards.sort();
        suit_cards.first().map(|c| *c)
    }

    /// Check if the hand has the "petit sec" (one of trump with no other trump nor the excuse)
    pub fn has_petit_sec(self) -> bool {
        let petit = Card::new(Suit::Trump, Rank::Rank1);
        self.has(petit) && self.trumps_count() == 1
    }

    pub fn count_oudlers(self) -> u8 {
        let petit = Card::new(Suit::Trump, Rank::Rank1);
        let vingtetun = Card::new(Suit::Trump, Rank::Rank21);
        let excuse = Card::new(Suit::Trump, Rank::Rank22);
        let mut count = 0;
        if self.has(petit){
            count = count + 1;
        }
        if self.has(vingtetun){
            count = count + 1;
        }
        if self.has(excuse){
            count = count + 1;
        }
        count
    }

    /// Returns the trumps in `self`.
    pub fn trumps(self) -> Self {
        Self(0, self.1)
    }

    /// Returns the non trumps in `self`.
    pub fn no_trumps(self) -> Self {
        Self(self.0, 0)
    }

    /// Returns the number of trumps in `self`.
    pub fn trumps_count(self) -> usize {
        let only_trumps = Self(0, self.1);
        only_trumps.size()
    }

    /// Returns the number of cards in `self`.
    pub fn size(self) -> usize {
        self.list().len()
    }
}

impl IntoIterator for Hand {
    type Item = Card;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().into_iter()
    }
}

impl ToString for Hand {
    /// Returns a string representation of `self`.
    fn to_string(&self) -> String {
        let str_cards: Vec<String> = self.list().iter()
            .map(|c| c.to_string())
            .collect();
        str_cards.join(", ")
        // let mut s = "[".to_owned();
        //
        // for c in &(*self).list() {
        //     s += &c.to_string();
        //     s += ",";
        // }
        //
        // s + "]"
    }
}

/// A deck of cards.
pub struct Deck {
    cards: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        Deck::new()
    }
}

impl Deck {
    /// Returns a full, sorted deck of 78 cards.
    pub fn new() -> Self {
        let mut d = Deck {
            cards: Vec::with_capacity(78),
        };

        for i in 0..66 {
            d.cards.push(Card::from_id(i));
        }
        for i in 70..82 {
            d.cards.push(Card::from_id(i));
        }

        d
    }

    pub fn get_suit_cards(&self, suit: Suit) -> Vec<Card> {
        self.cards.clone().into_iter().filter(|c| c.suit() == suit).collect()
    }

    /// Shuffle this deck.
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards[..].shuffle(&mut rng);
    }

    /// Shuffle this deck with the given random seed.
    ///
    /// Result is determined by the seed.
    pub fn shuffle_seeded(&mut self, seed: [u8; 32]) {
        let mut rng = StdRng::from_seed(seed);
        self.cards[..].shuffle(&mut rng);
    }

    /// Draw the top card from the deck.
    ///
    /// # Panics
    /// If `self` is empty.
    pub fn draw(&mut self) -> Card {
        self.cards.pop().expect("deck is empty")
    }

    /// Returns `true` if this deck is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Returns the number of cards left in this deck.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Deal `n` cards to each hand.
    ///
    /// # Panics
    /// If `self.len() < 5 * n`
    pub fn deal_each(&mut self, hands: &mut Vec<Hand>, n: usize) {
        if self.len() < hands.len() * n {
            panic!("Deck has too few cards!");
        }

        for hand in hands.iter_mut() {
            for _ in 0..n {
                hand.add(self.draw());
            }
        }
    }
}

impl ToString for Deck {
    fn to_string(&self) -> String {
        let mut s = "[".to_owned();

        for c in &self.cards {
            s += &c.to_string();
            s += ",";
        }

        s + "]"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suit_translation() {
        assert!(Rank::RankQ.to_locale_string("fr") == "D");
        assert!(Rank::RankQ.to_locale_string("es") == "Q");
        assert!(Rank::Rank2.to_locale_string("fr") == "2");
    }

    #[test]
    fn test_card_translation() {
        let card = Card::new(Suit::Heart, Rank::RankK);
        assert!(card.to_locale_string("fr") == "R♥");
    }

    #[test]
    fn test_card_from_str() {
        let card = Card::from_str("11T");
        assert!(card.is_ok());
        assert!(card.unwrap().to_locale_string("fr") == "11A");
    }

    #[test]
    fn test_card() {
        let card = Card::new(Suit::Trump, Rank::Rank22);
        assert!(81 == card.id());

        let card = Card::from_id(81);
        assert!(Card::new(Suit::Trump, Rank::Rank22) == card);
    }

    #[test]
    fn test_cards() {
        for i in 0..66 {
            let card = Card::from_id(i);
            assert!(i == card.id());
        }
        for i in 70..82 {
            let card = Card::from_id(i);
            assert!(i == card.id());
        }

        for s in 0..3 {
            let suit = Suit::from_n(s);
            for r in 0..13 {
                let rank = Rank::from_n(r);
                let card = Card::new(suit, rank);
                assert!(card.rank() == rank);
                assert!(card.suit() == suit);
            }
        }

        let suit = Suit::Trump;
        for r in 0..9 {
            let rank = Rank::from_n(r);
            let card = Card::new(suit, rank);
            assert!(card.rank() == rank);
            assert!(card.suit() == suit);
        }
        for r in 14..25 {
            let rank = Rank::from_n(r);
            let card = Card::new(suit, rank);
            assert!(card.rank() == rank);
            assert!(card.suit() == suit);
        }
    }

    #[test]
    fn test_hand() {
        let mut hand = Hand::new();

        let cards: Vec<Card> = vec![
            Card::new(Suit::Heart, Rank::Rank2),
            Card::new(Suit::Heart, Rank::Rank3),
            Card::new(Suit::Heart, Rank::Rank4),
            Card::new(Suit::Heart, Rank::Rank7),
            Card::new(Suit::Heart, Rank::Rank8),
            Card::new(Suit::Spade, Rank::Rank9),
            Card::new(Suit::Spade, Rank::RankJ),
            Card::new(Suit::Club, Rank::RankQ),
            Card::new(Suit::Club, Rank::RankK),
            Card::new(Suit::Diamond, Rank::Rank10),
            Card::new(Suit::Diamond, Rank::Rank8),
            Card::new(Suit::Diamond, Rank::Rank9),
            Card::new(Suit::Trump, Rank::Rank15),
            Card::new(Suit::Trump, Rank::Rank7),
            Card::new(Suit::Trump, Rank::Rank21),
        ];

        assert!(hand.is_empty());

        for card in cards.iter() {
            assert!(!hand.has(*card));
            hand.add(*card);
            assert!(hand.has(*card));
        }

        assert!(hand.size() == cards.len());

        for card in cards.iter() {
            assert!(hand.has(*card));
            hand.remove(*card);
            assert!(!hand.has(*card));
        }
    }

    #[test]
    fn test_has_any() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::Spade, Rank::Rank1));
        hand.add(Card::new(Suit::Spade, Rank::Rank10));
        hand.add(Card::new(Suit::Diamond, Rank::Rank4));
        hand.add(Card::new(Suit::Diamond, Rank::Rank5));
        hand.add(Card::new(Suit::Club, Rank::Rank6));
        hand.add(Card::new(Suit::Club, Rank::Rank9));
        hand.add(Card::new(Suit::Club, Rank::Rank10));
        hand.add(Card::new(Suit::Trump, Rank::Rank4));
        hand.add(Card::new(Suit::Trump, Rank::Rank6));
        hand.add(Card::new(Suit::Trump, Rank::Rank7));
        hand.add(Card::new(Suit::Trump, Rank::Rank9));
        hand.add(Card::new(Suit::Trump, Rank::Rank12));
        hand.add(Card::new(Suit::Trump, Rank::Rank13));
        hand.add(Card::new(Suit::Trump, Rank::Rank18));

        assert!(!hand.has_any(Suit::Heart));
        assert!(hand.has_any(Suit::Club));
        assert!(hand.has_any(Suit::Trump));
    }

    #[test]
    fn test_petit_sec() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::Spade, Rank::Rank1));
        hand.add(Card::new(Suit::Spade, Rank::Rank10));
        hand.add(Card::new(Suit::Diamond, Rank::Rank4));
        hand.add(Card::new(Suit::Diamond, Rank::Rank5));
        hand.add(Card::new(Suit::Club, Rank::Rank6));
        hand.add(Card::new(Suit::Club, Rank::Rank9));
        hand.add(Card::new(Suit::Club, Rank::Rank10));
        assert!(!hand.has_petit_sec());

        hand.add(Card::new(Suit::Trump, Rank::Rank1));
        assert!(hand.has_petit_sec());

        hand.add(Card::new(Suit::Trump, Rank::Rank22));
        assert!(!hand.has_petit_sec());
    }

    #[test]
    fn test_deck() {
        let mut deck = Deck::new();
        deck.shuffle();
        assert!(deck.len() == 78);

        let mut count = [0; 78];
        while !deck.is_empty() {
            let card = deck.draw();
            count[idx_from_id(card.id()) as usize] += 1;
        }

        for c in count.iter() {
            assert!(*c == 1);
        }
    }

    fn idx_from_id(id: u32) -> u32 {
        if id < 66 {
            id
        } else {
            id - 4
        }
    }
}

#[cfg(feature = "use_bench")]
mod benchs {
    use deal_seeded_hands;
    use test::Bencher;

    #[bench]
    fn bench_deal(b: &mut Bencher) {
        let seed = &[1, 2, 3, 4, 5];
        b.iter(|| {
            deal_seeded_hands(seed);
        });
    }

    #[bench]
    fn bench_list_hand(b: &mut Bencher) {
        let seed = &[1, 2, 3, 4, 5];
        let hands = deal_seeded_hands(seed);
        b.iter(|| {
            for hand in hands.iter() {
                hand.list().len();
            }
        });
    }

    #[bench]
    fn bench_del_add_check(b: &mut Bencher) {
        let seed = &[1, 2, 3, 4, 5];
        let hands = deal_seeded_hands(seed);
        let cards: Vec<_> = hands.iter().map(|h| h.list()).collect();
        b.iter(|| {
            let mut hands = hands.clone();
            for (hand, cards) in hands.iter_mut().zip(cards.iter()) {
                for c in cards.iter() {
                    hand.remove(*c);
                }
            }
            for (hand, cards) in hands.iter_mut().zip(cards.iter()) {
                for c in cards.iter() {
                    hand.add(*c);
                }
            }

            for (hand, cards) in hands.iter_mut().zip(cards.iter()) {
                for c in cards.iter() {
                    if !hand.has(*c) {
                        panic!("Error!");
                    }
                }
            }
        });
    }
}
