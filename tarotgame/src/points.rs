//! Manage points and scores

use super::cards;

/// Returns the number of points `card` is worth, with the current trump suit.
pub fn score(card: cards::Card, trump: cards::Suit) -> i32 {
    let r = card.rank();
    if card.suit() == trump {
        trump_score(r)
    } else {
        usual_score(r)
    }
}

/// Returns the strength of `card`, with the current trump suit.
pub fn strength(card: cards::Card, trump: cards::Suit) -> i32 {
    let r = card.rank();
    if card.suit() == trump {
        8 + trump_strength(r)
    } else {
        usual_strength(r)
    }
}

/// Returns the score for the given rank when it is the trump.
///
/// # Panics
/// If `rank` is invalid.
pub fn trump_score(rank: cards::Rank) -> i32 {
    match rank {
        cards::Rank::RankJ => 20,
        cards::Rank::Rank9 => 14,
        _ => usual_score(rank),
    }
}

/// Returns the score for the given rank when it is not the trump.
///
/// # Panics
/// If `rank` is invalid.
pub fn usual_score(rank: cards::Rank) -> i32 {
    match rank {
        cards::Rank::RankJ => 2,
        cards::Rank::RankC => 3,
        cards::Rank::RankQ => 4,
        cards::Rank::RankK => 5,
        _ => 0,
    }
}

/// Returns the strength for the given rank when it is the trump.
///
/// # Panics
/// If `rank` is invalid.
pub fn trump_strength(rank: cards::Rank) -> i32 {
    match rank {
        cards::Rank::Rank1 => 5,
        cards::Rank::Rank21 => 5,
        cards::Rank::Rank22 => 5,
        _ => 0,
    }
}

/// Returns the strength for the given rank when it is not the trump.
///
/// # Panics
/// If `rank` is invalid.
pub fn usual_strength(rank: cards::Rank) -> i32 {
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
        cards::Rank::Rank11 => 15,
        cards::Rank::Rank12 => 16,
        cards::Rank::Rank13 => 17,
        cards::Rank::Rank14 => 18,
        cards::Rank::Rank15 => 19,
        cards::Rank::Rank16 => 20,
        cards::Rank::Rank17 => 21,
        cards::Rank::Rank18 => 22,
        cards::Rank::Rank19 => 23,
        cards::Rank::Rank20 => 24,
        cards::Rank::Rank21 => 25,
        cards::Rank::Rank22 => 0,
    }
}
