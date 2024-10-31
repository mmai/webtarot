use std::fs::File;
use std::io::Read;
use std::path::Path;

use tarotgame::{cards::Hand, deal_seeded_hands};
use webtarot_protocol::TarotGameState;

pub fn read_json(json_path: &str) {
    let path = Path::new(json_path);

    let mut file = File::open(path).expect("Impossible d'ouvrir le fichier");
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)
        .expect("Erreur de lecture du fichier");

    let game: TarotGameState = serde_json::from_str(&json_str).expect("Error parsing JSON");
    println!("{}", game);
}

pub fn find_decks() {
    let mut found = false;

    let mut seed = [0; 32];

    let mut deal = deal_seeded_hands(seed, 5);
    let mut optseed = Some(seed);
    while !found && optseed.is_some() {
        seed = optseed.unwrap();
        deal = deal_seeded_hands(seed, 5);
        found = check_deal(&deal);
        optseed = incr_seed(seed);
    }

    if found {
        for hand in deal.0 {
            println!("{}", hand.to_string());
        }

        println!("{:?}", seed);
    }
}

fn check_deal(deal: &(Vec<Hand>, Hand)) -> bool {
    // Une main avec au moins 10 atouts
    deal.0.iter().find(|h| h.trumps_count() > 9).is_some()
}

// XXX non exhaustif
fn incr_seed(seed: [u8; 32]) -> Option<[u8; 32]> {
    let mut result = seed.clone();
    seed.iter()
        .enumerate()
        .find(|(i, n)| **n < 255)
        .map(|(i, n)| {
            result[i] = *n + 1;
            result
        })
}
