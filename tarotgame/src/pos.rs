//! Player position in the table

use serde::{Deserialize, Serialize};

/// A position in the table
#[derive(PartialEq,Clone,Copy,Debug,Serialize,Deserialize)]
pub enum PlayerPos {
    /// Player 0
    P0,
    /// Player 1
    P1,
    /// Player 2
    P2,
    /// Player 3
    P3,
    /// Player 4
    P4,
}

/// Iterates on players
pub struct PlayerIterator {
    current: PlayerPos,
    remaining: usize,
}

impl Iterator for PlayerIterator {
    type Item = PlayerPos;

    fn next(&mut self) -> Option<PlayerPos> {
        if self.remaining == 0 {
            return None;
        }

        let r = self.current;
        self.current = self.current.next();
        self.remaining -= 1;
        Some(r)
    }
}

impl PlayerPos {
    /// Returns the position corresponding to the number (0 => P0, ...).
    ///
    /// Panics if `n > 4`.
    pub fn from_n(n: usize) -> Self {
        match n {
            0 => PlayerPos::P0,
            1 => PlayerPos::P1,
            2 => PlayerPos::P2,
            3 => PlayerPos::P3,
            4 => PlayerPos::P4,
            other => panic!("invalid pos: {}", other),
        }
    }

    /// Returns the number corresponding to the position.
    ///
    pub fn to_n(self) -> usize {
        match self {
            PlayerPos::P0 => 0,
            PlayerPos::P1 => 1,
            PlayerPos::P2 => 2,
            PlayerPos::P3 => 3,
            PlayerPos::P4 => 4,
        }
    }

    /// Returns the next player in line
    pub fn next(self) -> PlayerPos {
        match self {
            PlayerPos::P0 => PlayerPos::P1,
            PlayerPos::P1 => PlayerPos::P2,
            PlayerPos::P2 => PlayerPos::P3,
            PlayerPos::P3 => PlayerPos::P4,
            PlayerPos::P4 => PlayerPos::P0,
        }
    }

    /// Returns the player `n` seats further
    pub fn next_n(self, n: usize) -> PlayerPos {
        if n == 0 {
            self
        } else {
            PlayerPos::from_n((self as usize + n) % 5)
        }
    }

    /// Returns the previous player.
    pub fn prev(self) -> PlayerPos {
        match self {
            PlayerPos::P0 => PlayerPos::P4,
            PlayerPos::P1 => PlayerPos::P0,
            PlayerPos::P2 => PlayerPos::P1,
            PlayerPos::P3 => PlayerPos::P2,
            PlayerPos::P4 => PlayerPos::P3,
        }
    }

    /// Returns an iterator that iterates on `n` players, including this one.
    pub fn until_n(self, n: usize) -> PlayerIterator {
        PlayerIterator {
            current: self,
            remaining: n,
        }
    }

    /// Returns the number of turns after `self` to reach `other`.
    pub fn distance_until(self, other: PlayerPos) -> usize {
        (4 + other as usize - self as usize) % 5 + 1
    }

    /// Returns an iterator until the given player (`self` included, `other` excluded)
    pub fn until(self, other: PlayerPos) -> PlayerIterator {
        let d = self.distance_until(other);
        self.until_n(d)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_pos() {
        let mut count = [0; 5];
        for i in 0..5 {
            for pos in PlayerPos::from_n(i).until(PlayerPos::from_n(0)) {
                count[pos as usize] += 1;
            }
            for pos in PlayerPos::from_n(0).until(PlayerPos::from_n(i)) {
                count[pos as usize] += 1;
            }
        }

        for c in count.iter() {
            assert!(*c == 6);
        }

        for i in 0..5 {
            assert!(PlayerPos::from_n(i).next() == PlayerPos::from_n((i + 1) % 5));
            assert!(PlayerPos::from_n(i) == PlayerPos::from_n((i + 1) % 5).prev());
            assert!(PlayerPos::from_n(i).next().prev() == PlayerPos::from_n(i));
        }
    }
}
