//! Player position in the table

use serde::{Deserialize, Serialize};

/// A position in the table
#[derive(PartialEq,Clone,Copy,Debug,Serialize,Deserialize)]
pub struct PlayerPos {
    pub pos: AbsolutePos,
    pub count: u8,
}

/// A position in the table
#[derive(PartialEq,Clone,Copy,Debug,Serialize,Deserialize)]
pub enum AbsolutePos {
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
    pub fn new(pos: AbsolutePos, count: u8) -> Self {
        PlayerPos { pos, count}
    } 

    /// Returns the position corresponding to the number (0 => P0, ...).
    ///
    /// Panics if `n > 4`.
    pub fn from_n(n: usize, count: u8) -> Self {
        let pos = match n {
            0 => AbsolutePos::P0, 
            1 => AbsolutePos::P1,
            2 => AbsolutePos::P2,
            3 => AbsolutePos::P3,
            4 => AbsolutePos::P4,
            other => panic!("invalid pos: {}", other),
        };
        PlayerPos { pos, count}
    }

    /// Returns the number corresponding to the position.
    ///
    pub fn to_n(self) -> usize {
        match self.pos {
            AbsolutePos::P0 => 0,
            AbsolutePos::P1 => 1,
            AbsolutePos::P2 => 2,
            AbsolutePos::P3 => 3,
            AbsolutePos::P4 => 4,
        }
    }

    /// Returns the next player in line
    pub fn next(self) -> PlayerPos {
        PlayerPos::from_n((self.to_n() as usize + 1) % self.count as usize, self.count)
    }

    /// Returns the player `n` seats further
    pub fn next_n(self, n: usize) -> PlayerPos {
        if n == 0 {
            self
        } else {
            PlayerPos::from_n((self.to_n() as usize + n) % self.count as usize, self.count)
        }
    }

    /// Returns the previous player.
    pub fn prev(self) -> PlayerPos {
        PlayerPos::from_n((self.to_n() as usize - 1) % self.count as usize, self.count)
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
        (self.count as usize - 1 + other.to_n() as usize - self.to_n() as usize) % self.count as usize + 1
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
        let count: u8 = 5;
        let mut counts = [0; 5];
        for i in 0..count {
            for pos in PlayerPos::from_n(i as usize, count).until(PlayerPos::from_n(0, count)) {
                counts[pos.pos as usize] += 1;
            }
            for pos in PlayerPos::from_n(0, count).until(PlayerPos::from_n(i as usize, count)) {
                counts[pos.pos as usize] += 1;
            }
        }

        for c in counts.iter() {
            assert!(*c == 6);
        }

        for i in 0..count {
            assert!(PlayerPos::from_n(i as usize, count).next() == PlayerPos::from_n((i as usize + 1) % count as usize, count));
            assert!(PlayerPos::from_n(i as usize, count) == PlayerPos::from_n((i as usize + 1) % count as usize, count).prev());
            assert!(PlayerPos::from_n(i as usize, count).next().prev() == PlayerPos::from_n(i as usize, count));
        }
    }
}
