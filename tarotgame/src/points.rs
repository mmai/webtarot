//! Manage points and scores

use super::cards;

pub fn score(points: f32, oudlers_count: u8) -> (f32, f32) {
    let raw_points = match oudlers_count {
        3 => points - 36.0,
        2 => points - 41.0,
        1 => points - 51.0,
        _ => points - 56.0,
    };
    let points = if raw_points < 0.0 {
        raw_points - 25.0
    } else {
        raw_points + 25.0
    };
    (raw_points, points)
}

/// Returns the number of points `card` is worth
pub fn points(card: cards::Card) -> f32 {
    match card.rank() {
        cards::Rank::Rank1 if card.suit() == cards::Suit::Trump => 4.5, 
        cards::Rank::Rank21 => 4.5,
        cards::Rank::Rank22 => 4.5,
        cards::Rank::RankJ => 1.5,
        cards::Rank::RankC => 2.5,
        cards::Rank::RankQ => 3.5,
        cards::Rank::RankK => 4.5,
        _ => 0.5,
    }
}

pub fn hand_points(hand: cards::Hand) -> f32 {
    hand.list().iter()
        .map(|c| points(*c))
        .sum()
}

/// Returns the strength of `card`
pub fn strength(card: cards::Card) -> i32 {
    let rank = card.rank();
    if card.suit() == cards::Suit::Trump {
        match rank {
            cards::Rank::Rank1  => 21,
            cards::Rank::Rank2  => 22,
            cards::Rank::Rank3  => 23,
            cards::Rank::Rank4  => 24,
            cards::Rank::Rank5  => 25,
            cards::Rank::Rank6  => 26,
            cards::Rank::Rank7  => 27,
            cards::Rank::Rank8  => 28,
            cards::Rank::Rank9  => 29,
            cards::Rank::Rank10 => 30,
            cards::Rank::Rank11 => 31,
            cards::Rank::Rank12 => 32,
            cards::Rank::Rank13 => 33,
            cards::Rank::Rank14 => 34,
            cards::Rank::Rank15 => 35,
            cards::Rank::Rank16 => 36,
            cards::Rank::Rank17 => 37,
            cards::Rank::Rank18 => 38,
            cards::Rank::Rank19 => 39,
            cards::Rank::Rank20 => 40,
            cards::Rank::Rank21 => 41,
            cards::Rank::Rank22 => 0,
            _ => 0
        }
    } else {
        match rank {
            cards::Rank::Rank1  => 1,
            cards::Rank::Rank2  => 2,
            cards::Rank::Rank3  => 3,
            cards::Rank::Rank4  => 4,
            cards::Rank::Rank5  => 5,
            cards::Rank::Rank6  => 6,
            cards::Rank::Rank7  => 7,
            cards::Rank::Rank8  => 8,
            cards::Rank::Rank9  => 9,
            cards::Rank::Rank10 => 10,
            cards::Rank::RankJ  => 11,
            cards::Rank::RankC  => 12,
            cards::Rank::RankQ  => 13,
            cards::Rank::RankK  => 14,
            _ => 0
        }
    }
}
