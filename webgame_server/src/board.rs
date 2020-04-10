use std::iter;

use lazy_static::lazy_static;
use rand::prelude::*;

use crate::protocol::{Tile, Turn};

pub const SIZE: usize = 5;

lazy_static! {
    static ref WORDS: Vec<String> = include_str!("wordlist.txt")
        .lines()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
}

pub struct Board {
    tiles: Vec<Tile>,
}

impl Board {
    /// Creates a new board.
    pub fn new() -> Board {
        let mut rng = thread_rng();
        let tiles = WORDS
            .choose_multiple(&mut rng, SIZE * SIZE)
            .map(|word| Tile {
                codeword: word.to_string(),
                spotted: false,
            })
            .collect();

        Board {
            tiles,
        }
    }

    /// Returns tiles with non spotted characters hidden.
    pub fn tiles(&self, reveal: bool) -> Vec<Tile> {
        self.tiles
            .iter()
            .map(|tile| {
                let mut tile = tile.clone();
                tile
            })
            .collect()
    }

    /// Returns the initial turn
    pub fn initial_turn(&self) -> Turn {
        Turn::BiddingP0
    }
}
