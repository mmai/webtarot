use tarotgame::{deal_seeded_hands, cards::Hand} ;

pub fn find_decks() {
    let mut found = false;

    let mut seed = [0;32];


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
fn incr_seed(seed: [u8;32]) -> Option<[u8;32]> {
    let mut result = seed.clone();
    seed.iter().enumerate()
        .find(|(i, n)| **n < 255)
        .map(|(i, n)| {
            result[i] = *n + 1;
            result
        })
}
