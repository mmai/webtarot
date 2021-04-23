use std::{thread, time};
// use std::rc::Rc;
use std::collections::HashMap;
 
use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::{Deck, Card, Hand, Suit, Rank}, deal::can_play, bid::Target, points::strength, pos::PlayerPos, trick::Trick};
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState, BidCommand, CallKingCommand, MakeDogCommand, Turn, PlayerRole, PlayEvent, TarotVariant, VariantSettings};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

pub trait InOut {
    fn read(&mut self) -> Message;
    fn send(&mut self, command: &Command) -> Result<()>;
    fn close(&mut self);
}

#[derive(Clone)]
struct DealStats {
    pub players: Vec<PlayerStats>,
    pub suit_left: HashMap<Suit, Hand>,
    pub suit_played: [bool;5],
    pub teams_known: bool,
    pub teams_known_by_all: bool,
    // pub count: usize,
}

impl DealStats {
    fn new() -> Self {
        DealStats { 
            players: vec![],
            suit_left: HashMap::default(),
            suit_played: [false; 5],
            teams_known: false,
            teams_known_by_all: false,
            // count: 0,
        }
    }

    fn init_state(&mut self, nb_players: usize, hand: Hand) {
        self.players = vec![ PlayerStats::new() ; nb_players];
        self.suit_played = [false; 5];
        self.teams_known = false;
        self.teams_known_by_all = false;
        // self.count = self.count + 1;
        // println!("---------------------------- DEAL {} --------------------", self.count);

        //reset cards left
        let deck = Deck::new();
        let hearts = deck.get_suit_cards(Suit::Heart).into();
        let clubs = deck.get_suit_cards(Suit::Club).into();
        let spades = deck.get_suit_cards(Suit::Spade).into();
        let diamonds = deck.get_suit_cards(Suit::Diamond).into(); 
        let trumps = deck.get_suit_cards(Suit::Trump).into();
        self.suit_left = [
                (Suit::Heart, hearts),
                (Suit::Spade, spades),
                (Suit::Diamond, diamonds),
                (Suit::Club, clubs),
                (Suit::Trump, trumps)
            ].iter().cloned().collect();

        //remove own cards
        for suit in &[Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart, Suit::Trump] {
            for card in hand.get_suit_cards(suit) {
                (*self.suit_left.get_mut(suit).unwrap()).remove(card);
            }
        }
    }

    fn partner_is_after(self, trick: &Trick, mepos: usize ) -> Option<bool> {
        let me = &self.players[mepos];
        if self.teams_known {
            let is_after = self.players.iter().enumerate().any(|(npos, player)| {
                let pos = PlayerPos::from_n(npos, self.players.len() as u8);
                player.is_partner(me) == Some(true) && !trick.clone().player_already_played(pos,)
            });
            return Some(is_after);
        } else if self.is_trick_last_player(trick, mepos){
            return Some(false);
        }
        None
    }

    fn opponent_is_after(self, trick: &Trick, mepos: usize) -> Option<bool> {
        let me = &self.players[mepos];
        if self.teams_known {
            // print!(" i know the teams.. ");
            let is_after = self.players.iter().enumerate().any(|(npos, player)| {
                let pos = PlayerPos::from_n(npos, self.players.len() as u8);
                player.is_partner(me) == Some(false) && !trick.clone().player_already_played(pos)
            });
            return Some(is_after);
        } else if self.is_trick_last_player(trick, mepos){
            // print!(" i am the last so no opponent after me.. ");
            return Some(false);
        }
        None
    }

    //Last player of the trick ?
    fn is_trick_last_player(&self, trick: &Trick, mepos: usize) -> bool {
        trick.first.prev().to_n() == mepos
    }

    fn suit_is_cut(&self, suit: Suit) -> bool {
        if self.suit_left[&Suit::Trump].size() == 0 { return false; } // No more trumps
        self.players.iter().any(|player| 
            player.suits_available.get(&suit).unwrap() == &Some(false) &&
            player.suits_available.get(&Suit::Trump).unwrap() != &Some(false)
            )
    }

    fn suit_already_played(&self, suit: Suit) -> bool {
        self.suit_played[suit.to_n()]
    }

    fn set_suit_played(&mut self, suit: &Suit) {
        self.suit_played[suit.to_n()] = true;
    }
}


#[derive(Clone, Debug)]
struct PlayerStats {
    played: Hand,
    is_taker: bool,
    in_taker_team: Option<bool>,
    suits_available: HashMap<Suit, Option<bool>>,
}

impl PlayerStats {
    fn new() -> Self {
        PlayerStats {
            played: Hand::new(), 
            is_taker: false,
            in_taker_team: None,
            suits_available: [
                (Suit::Heart, None),
                (Suit::Spade, None),
                (Suit::Diamond, None),
                (Suit::Club, None),
                (Suit::Trump, None)
            ].iter().cloned().collect(),
        } 
    }

    fn is_partner(&self, player: &PlayerStats) -> Option<bool> {
        if player.in_taker_team.is_none() || self.in_taker_team.is_none() {
            None
        } else {
            Some(player.in_taker_team == self.in_taker_team)
        }
    }
}

pub struct Player {
    delay: time::Duration,
    in_out: Box<dyn InOut>,
    join_code: String,
    game_state: GameStateSnapshot,
    player_info: PlayerInfo,
    stats: DealStats,
    // stats: Rc<DealStats>,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.in_out.close();
    }
}

impl Player {
    pub fn new(in_out: Box<dyn InOut>, join_code: String, nickname: String, delay: time::Duration) -> Self {
        Player { 
            delay,
            in_out,
            join_code,
            game_state: GameStateSnapshot::default(),
            player_info: PlayerInfo { id: Uuid::default(), nickname } ,
            stats: DealStats::new(),
            // stats: Rc::new(DealStats::new()),
        }
    }
    
    pub fn play(&mut self){
        self.in_out.send(&Command::Authenticate(AuthenticateCommand { nickname: self.player_info.nickname.clone() }));
        loop {
            let message = self.in_out.read();
            if self.handle_server_message(message) {
                break
            }
        }

    }

    fn update_stats(&mut self) {
        let deal = &self.game_state.deal;
        //My stats
        let my_idx = self.my_state().pos.to_n();
        if self.stats.players.len() > 0 {
            for suit in &[Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart] {
                *self.stats.players[my_idx].suits_available.get_mut(suit).unwrap() = Some(deal.hand.get_suit_cards(suit).len() > 0);
            }

            let trick_suit = deal.last_trick.suit();
            let cards = deal.last_trick.cards;
            cards.iter().enumerate().for_each(|(pos, card)| {
                card.map(|c| {
                    self.stats.players[pos].played.add(c);
                    (*self.stats.suit_left.get_mut(&c.suit()).unwrap()).remove(c);
                    if Some(c.suit()) != trick_suit && c != Card::excuse() {
                        *self.stats.players[pos].suits_available.get_mut(&trick_suit.unwrap()).unwrap() = Some(false);
                    }
                });
            });

            if let Turn::Playing(_) = self.game_state.turn {
                self.update_partners();
            }
            // self.stats.players.iter().for_each(|p| println!("{:?} {:?}", p.played.to_string(), p.in_taker_team));
            // println!("suits cards not played for {}:", self.player_info.nickname);
            // self.stats.suit_left.iter().for_each(|(s, h)| println!("{:?} {}", s, h.to_string()));
        }
    }

    fn get_taker_pos(&self) -> Option<u8> {
        let taker_opt = self.game_state.players.iter()
            .find(|p| p.role == PlayerRole::Taker);
        if let Some(taker) = taker_opt {
            return Some(taker.pos.to_n() as u8);
        }
        None
    }

    fn update_partners(&mut self) {
        let deal = &self.game_state.deal;
        if self.stats.teams_known_by_all { return () }
        self.stats.teams_known_by_all = deal.king.map(|king| !self.stats.suit_left[&king.suit()].has(king)).unwrap_or(true);
        if self.stats.teams_known { return () }
        let me_pos = self.my_state().pos.to_n();
        let mystats = &self.stats.players[me_pos];

        let taker_opt = self.game_state.players.iter()
            .find(|p| p.role == PlayerRole::Taker);
        if let Some(taker) = taker_opt {
            // print!(" taker.. ");
            self.stats.players[taker.pos.to_n()].is_taker = true;
            self.stats.players[taker.pos.to_n()].in_taker_team = Some(true);
        }

        if let Some(king) = deal.king {
            if deal.hand.has(king) { // I have the king
                // print!("teams : i have king.. ");

                for (idx, pstat) in  self.stats.players.iter_mut().enumerate() {
                    pstat.in_taker_team = Some(pstat.is_taker || idx == me_pos);
                }
                self.stats.teams_known = true;
            } else { //I do not have the king
                // print!("teams : i don't have king.. ");
                if self.stats.players[me_pos].in_taker_team.is_none() {
                    self.stats.players[me_pos].in_taker_team = Some(false);
                }

                // Do other players played the king ?
                let partner_pos = self.stats.players.iter()
                    .enumerate()
                    .find(|(idx, pstat)| pstat.played.has(king))
                    .map(|(idx, pstat)| idx);
                if let Some(pos) = partner_pos { // yes, king played
                    // print!(" teams : king was played.. ");
                    for (idx, pstat) in  self.stats.players.iter_mut().enumerate() {
                        pstat.in_taker_team = Some(pstat.is_taker || idx == pos);
                    }
                    self.stats.teams_known = true;
                    self.stats.teams_known_by_all = true;
                } else { // King not played
                    // print!(" teams : king was not played.. ");
                    //Check players who don't have cards of the king's suit
                    for pstat in  self.stats.players.iter_mut() {
                        if pstat.in_taker_team.is_none() && Some(false) == pstat.suits_available[&king.suit()] {
                            pstat.in_taker_team = Some(false);
                        }
                    }
                }
            }
        } else { // Aucun roi appelé : le preneur est seul 
            self.stats.players.iter_mut().map(|pstat|
                pstat.in_taker_team = Some(pstat.is_taker)
            );
            self.stats.teams_known = true;
            self.stats.teams_known_by_all = true;
        }
        let mystats = &self.stats.players[me_pos];
    }

    fn my_state(&self) -> &GamePlayerState {
        self.game_state
            .players
            .iter()
            .find(|state| state.player.id == self.player_info.id)
            .unwrap()
    }

    // Returns a bool : do we exit ?
    fn handle_server_message(&mut self, msg: Message) -> bool {
        match msg {
            Message::Authenticated(player_info) => {
                self.player_info = player_info;
                // println!("Authenticated with id {}", self.player_info.id);
                if self.join_code != "" {
                    self.in_out.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                } else {
                    //Create game
                    // println!("creating game");
                    let variant = TarotVariant {
                        parameters: VariantSettings { nb_players: 5 }
                    };
                    self.in_out.send(&Command::NewGame(variant));
                }
            }
            Message::GameJoined(game_info) => {
                // println!("Game joined: {:?}", game_info);
                if self.join_code == "" {
                    // println!("need to invite other bots");
                    self.join_code = game_info.join_code.clone();
                    self.in_out.send(&Command::InviteBot);
                    self.in_out.send(&Command::InviteBot);
                    self.in_out.send(&Command::InviteBot);
                    self.in_out.send(&Command::InviteBot);
                }
                self.in_out.send(&Command::MarkReady);
            }
            Message::GameStateSnapshot(game_state) => {
                if game_state != self.game_state {
                    self.game_state = game_state;
                    self.handle_new_state();
                }
            }
            Message::PlayEvent(play_event) => {
                // println!("play event for {}: {:?}", self.player_info.nickname, play_event);
            }
            Message::Chat(_) => {}
            Message::Pong => {
                // println!("Received a pong !!");
            }
            Message::ServerStatus(server_status) => {
                server_status.games.iter()
                    .next()
                    .map(|g| {
                        // println!("Found a game with code {}", g.game.join_code);
                        self.join_code = g.game.join_code.clone();
                        self.in_out.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                    });
            }
            Message::PlayerDisconnected(_) => {
                // println!("A player disconnected, I disconnect too");
                return true; //We exit
            }
            _ => {
                // println!("Unmanaged server message for {}: {:?}", self.player_info.nickname, msg);
            }
        }
        false // we don't exit
    }

    fn handle_new_state(&mut self){
        self.update_stats();
        let my_state = self.my_state();
        let player_action = my_state.get_turn_player_action(self.game_state.turn);
        // let mypos = my_state.pos.to_n();
        // let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        match player_action {
            Some(PlayerAction::Bid) => {
                //deal has started, we can init its state
                self.stats.init_state(self.game_state.nb_players as usize, self.game_state.deal.hand);
                if let Some(target) = self.guess_bid() {
                    self.in_out.send(&Command::GamePlay(GamePlayCommand::Bid(BidCommand { target, slam: false })));
                } else {
                    self.in_out.send(&Command::GamePlay(GamePlayCommand::Pass));
                }
            }
            Some(PlayerAction::CallKing) => {
                self.in_out.send(&Command::GamePlay(GamePlayCommand::CallKing(CallKingCommand { card: self.call_king() })));
            }
            Some(PlayerAction::MakeDog) => {
                let dog  = self.make_dog();
                for card in dog.list() {
                    (*self.stats.suit_left.get_mut(&card.suit()).unwrap()).remove(card);
                }
                self.in_out.send(&Command::GamePlay(GamePlayCommand::MakeDog(MakeDogCommand { cards: dog, slam: false })));
            }
            Some(PlayerAction::Play) => {
                let card_played = self.game_state.deal.last_trick.card_played(my_state.pos);
                if card_played.is_none() {
                    let mystate = self.my_state().clone();
                    self.choose_card()
                        .or_else(|| {
                            // println!("\n============================ No card played !!");
                            // println!("state : {:?}", mystate);
                            None
                        })
                        .map(|card| {
                            // println!(">> {} is playing {}...\n\n", self.player_info.nickname, card.to_string());
                            self.stats.set_suit_played(&card.suit());
                            self.in_out.send(&Command::GamePlay(GamePlayCommand::Play(PlayCommand { card })));
                        })
                    ;
                }
            }
            _ => {}
        }
    }

    fn guess_bid(&self) -> Option<Target>{
        let curr_target = &self.game_state.deal.contract_target();

        let points = self.evaluate_hand();
        let candidate = if points < 46 {
            None
        } else if points < 58 {
            Some(Target::Prise)
        } else if points < 75 {
            Some(Target::Garde)
        } else if points < 85 {
            Some(Target::GardeSans)
        } else {
            Some(Target::GardeContre)
        };
        // println!("points : {}", points);
        candidate.filter(|bidtarget| curr_target.lt(&Some(*bidtarget)))
    }

    // cf. https://www.le-tarot.fr/quel-contrat-choisir/
    fn evaluate_hand(&self) -> usize {
        let mut points = match self.stats.players.len() {
            5 => 14,
            4 => 7,
            3 => 0,
            _ => 0,
        };

        let deal = &self.game_state.deal;
        let hand = deal.hand;
        let trumps = hand.trumps();
        let trumps_count = trumps.size();

        // oudlers
        let t21 = Card::new(Suit::Trump, Rank::Rank21);
        let excuse = Card::new(Suit::Trump, Rank::Rank22);
        let petit = Card::new(Suit::Trump, Rank::Rank1);
        if hand.has(t21) { points += 10 }
        if hand.has(excuse) { points += 7 }
        if hand.has(petit) { 
            points += match trumps_count {
                n if 7 < n => 8,
                6 => 7,
                5 => 6,
                _ => 0
            }
        }
        // trumps
        points += trumps_count * 2;
        let trump15 = Card::new(Suit::Trump, Rank::Rank15);
        let big_trumps: Vec<Card>= trumps.into_iter().filter(|c| strength(*c) > strength(trump15) && c.rank() != Rank::Rank22).collect();
        let big_trumps = big_trumps.len();
        points += big_trumps * 2;
        if big_trumps > 4 { points += big_trumps }
        // Honours
        for suit in &[Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart] {
            let suit_cards: Vec<Card> = hand.into_iter().filter(|c| &c.suit() == suit).collect();
            let suit_count = suit_cards.len();
            let has_king = hand.has(Card::new(*suit, Rank::RankK));
            let has_queen = hand.has(Card::new(*suit, Rank::RankQ));
            let has_cavale = hand.has(Card::new(*suit, Rank::RankC));
            let has_jack = hand.has(Card::new(*suit, Rank::RankJ));

            if has_king { points += if has_queen { 7 } else { 6 } }
            if has_queen { points += 3 }
            if has_cavale { points += 2 }
            if has_jack { points += 1 }

            //Coupe
            if suit_count == 0 { points += 5 }
            //Singlette
            if suit_count == 1 { points += 3 }
            //Longue
            if suit_count > 4 {
                points += 5 + (suit_count - 5) * 2
            }
        }
        points
    }

    fn call_king(&self) -> Card {
        let deal = &self.game_state.deal;
        let hand = deal.hand;
        let rank = if hand.has_all_rank(Rank::RankK) {
            if hand.has_all_rank(Rank::RankQ) { Rank::RankC } else { Rank::RankQ } 
        } else {
            Rank::RankK
        };

        let mut suits = [Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart];
        suits.sort_by(|a, b| {
                let a_cards: Vec<Card> = hand.into_iter().filter(|c| &c.suit() == a).collect();
                let b_cards: Vec<Card> = hand.into_iter().filter(|c| &c.suit() == b).collect();
                a_cards.len().cmp(&b_cards.len())
            } );

        let mut candidates: Vec<Card> = suits.into_iter()
            .filter( |suit| !hand.has(Card::new(**suit, rank)) )
            .map(|suit| Card::new(*suit, rank))
            .collect();
        candidates.pop().unwrap()
    }

    fn make_dog(&mut self) -> Hand {
        //Let the players see the initial dog
        let now = time::Instant::now();
        thread::sleep(self.delay);

        let mut dog = Hand::new();
        let deal = &self.game_state.deal;

        //remove dog cards from suit_left
        for card in deal.initial_dog {
            (*self.stats.suit_left.get_mut(&card.suit()).unwrap()).remove(card);
        }

        let dog_size = deal.initial_dog.size();
        let mut hand_all = deal.hand.clone();
        hand_all.merge(deal.initial_dog);
        let mut hand = hand_all.no_trumps();
        //Check if we can make a cut
        for suit in &[Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart] {
            let suit_cards: Vec<Card> = hand.into_iter().filter(|c| &c.suit() == suit).collect();
            let king = Card::new(*suit, Rank::RankK);
            let suit_count = suit_cards.len();
            if suit_count <= (dog_size - dog.size()) && !hand.has(king) {
                for card in suit_cards {
                    dog.add(card);
                    hand.remove(card);
                } 
            }
            hand.remove(king); // kings not allowed in dog
        }
        //Put points 
        let mut queens: Vec<Card> = hand.into_iter().filter(|c| c.rank() == Rank::RankQ).collect(); 
        let mut cavales: Vec<Card> = hand.into_iter().filter(|c| c.rank() == Rank::RankC).collect(); 
        let mut candidates: Vec<Card> = hand.into_iter().filter(|c| c.rank() == Rank::RankJ).collect(); 
        candidates.append(& mut cavales);
        candidates.append(& mut queens);
        let mut candidate = candidates.pop();
        while dog.size() < dog_size  && candidate.is_some() {
            let card = candidate.unwrap();
            dog.add(card);
            hand.remove(card);
            candidate = candidates.pop();
        }
        //other cards
        //(we assume there is enough non trumps cards : 
        //if not, the contract should have been "garde sans" or "garde contre")
        let mut cards = hand.list();
        let mut card = cards.pop();
        while dog.size() < dog_size && card.is_some() {
            dog.add(card.unwrap());
            card = cards.pop();
        }

        assert!(dog.size() == dog_size);
        dog
    }

    fn check_play_excuse_instead_of_trump(&self) -> bool {
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;
        let excuse = Card::new(Suit::Trump, Rank::Rank22);

        let winner_card = trick.cards[trick.winner.pos as usize].unwrap();

        // We do not play the excuse if we are the only one to cut
        if winner_card.suit() != Suit::Trump {
            return false
        }

        if hand.has(excuse) {
            let mut without_excuse = hand.clone();
            without_excuse.remove(Card::new(Suit::Trump, Rank::Rank22));
            if let Some(high_trump) = without_excuse.suit_highest(Suit::Trump) {
                //we can't win, or there is no points to gain
                if winner_card.rank() > high_trump.rank() || trick.points() <= 2.0 {
                    return true;
                }
            } else { // we do not have any other trump
                return true;
            }
        }
        false
    }

    fn choose_card(&self) -> Option<Card>{
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;
        let excuse = Card::new(Suit::Trump, Rank::Rank22);
        let petit = Card::new(Suit::Trump, Rank::Rank1);

        //Play the excuse before the last trick
        if hand.size() == 2 && hand.has(excuse) {
            return Some(excuse);
        }

        let mepos = self.my_state().pos.to_n();
        let me = &self.stats.players[mepos];

        let danger = self.stats.clone().opponent_is_after(trick, mepos) != Some(false);

        if let Some(starting_suit) = trick.suit() { // Not the first to play
            // print!("not the first to play..  ");
            let winner_card = trick.cards[trick.winner.pos as usize].unwrap();

            if starting_suit == Suit::Trump {
                // print!("trump asked..  ");
                // Try to save partner's petit
                let found = self.play_try_save_petit(false);
                if found.is_some() { return found };

                // Try to save own's petit
                let found = self.play_try_own_petit();
                if found.is_some() { return found };

                //Play the excuse if there is no points to gain
                if self.check_play_excuse_instead_of_trump() {
                    return Some(excuse);
                }

                // Must be higher trump if exists
                if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, winner_card){
                    return Some(mylowest);
                }
                // or cut with smallest (except petit)
                if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, petit){
                    return Some(mylowest);
                }

            } else {
                // print!("no trump asked..  ");
                let my_highest = hand.suit_highest(starting_suit);
                let highest_left = self.stats.suit_left.get(&starting_suit).unwrap().suit_highest(starting_suit);
                if my_highest.is_some() {
                    // print!("i have color..  ");
                    let mut myhighest = my_highest.unwrap();
                    if myhighest > winner_card && winner_card.suit() != Suit::Trump { // I can win the trick
                        // print!("i can win..  ");
                        // If not points, take the lowest still winning  
                        if myhighest.rank() < Rank::RankJ {
                            // print!("with a no points card..  ");
                            return hand.suit_lowest_over_card(starting_suit, winner_card);
                        }

                        if highest_left.is_some() {
                            // print!("highest_left: {}..  ", highest_left.unwrap().to_string());
                            let is_cut = self.stats.suit_is_cut(starting_suit);
                            // if is_cut { println!("I am cut"); } else { println!("I am not cut"); }
                            // if me.in_taker_team == Some(true) { println!("in taker team"); } else { println!("not in taker team"); }
                            // if danger { println!("danger"); } else { println!("no danger"); }

                            let taker_pos = PlayerPos::from_n(self.get_taker_pos().unwrap() as usize, self.stats.players.len() as u8);
                            let taker_already_played = trick.clone().player_already_played(taker_pos);
                            if  myhighest > highest_left.unwrap() && !is_cut &&
                                ( me.in_taker_team == Some(true) || !danger || (self.is_first_time_suit_played(&starting_suit) && taker_already_played)) {
                                return Some(myhighest);
                            }
                        } else {
                            // print!("i will be cut..  ");
                            if let Some(mylowest) = hand.suit_lowest(starting_suit){
                                return Some(mylowest);
                            }

                        }


                    } else { // I can't win the trick
                        // print!("i can't win..  ");
                        // Give points if my parter win the trick
                        if self.stats.players[trick.winner.pos.to_n()].is_partner(me) == Some(true)
                           && myhighest.rank() >= Rank::RankJ
                           && ( !danger || 
                               ( me.in_taker_team == Some(true) && (winner_card.suit() == Suit::Trump || highest_left.map(|c|c.rank() > winner_card.rank()) != Some(true) ) )
                               ) {
                            return Some(myhighest);
                        } else {
                            return hand.suit_lowest(starting_suit);
                        }
                    }

                } else { // I must cut or piss
                    // print!("i have not the color..  ");
                    // Try to save own's petit
                    let found = self.play_try_own_petit();
                    if found.is_some() { return found };

                    // If there is points to save and i am not the last to play and it is not the
                    // first time we play this color: play high
                    if !self.is_first_time_suit_played(&starting_suit) && trick.points() > 3.0 && self.stats.clone().opponent_is_after(trick, mepos) != Some(false) {
                        if let Some(highest) = hand.trump_highest(){
                            if highest.rank() > winner_card.rank() {
                                return Some(highest);
                            }
                        }
                    }

                    // Must be higher trump than other cuts
                    if winner_card.suit() == Suit::Trump { 
                        if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, winner_card){
                            return Some(mylowest);
                        }
                    }
                    // or cut with smallest (except petit)
                    if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, petit){
                        return Some(mylowest);
                    }

                    //Play the excuse if there is no points to gain
                    if self.check_play_excuse_instead_of_trump() {
                        return Some(excuse);
                    }

                }
            }
            
        } else {//First to play 
            // print!("i am first to play..  ");
            let found = self.play_try_save_petit(true);
            if found.is_some() { return found };

            // I am the taker and it is not the first trick :
            // play for the king I called if the suit has not been already played
            if me.is_taker && deal.trick_count > 1 {
                // print!("i am the taker..  ");
                if let Some(king) = deal.king {
                    // print!("i called the {} king..  ", king.to_string());
                    if !self.stats.suit_already_played(king.suit()) {
                        // print!("whose color has not been played..  ");
                        let card = hand.suit_highest(king.suit());//We give points
                        if card.is_some() {
                            return card;
                        }
                    }
                }
            } 

            // In  taker team & have king & suit not cut by opponents : play king
            if me.in_taker_team == Some(true) {
                // print!("i am in the taker team..  ");
                let kings: Vec<Card> = hand.list().into_iter()
                    .filter(|card| { card.rank() == Rank::RankK })
                    .collect();
                for king in kings {
                    if !self.stats.suit_is_cut(king.suit()) {
                        return Some(king);
                    // } else { print!("my king {:?} would be cut.. ", king); 
                    }
                }
            }

            //Remove cards of the called king if it's the first trick
            let forbidden_suit: Option<Suit> = if deal.trick_count == 1 {
                deal.king.map(|k| k.suit())
            } else {
                None
            };
            let mut playable_suits: Vec<Suit> = vec![Suit::Heart, Suit::Spade, Suit::Diamond, Suit::Club]
                .into_iter().filter(|s| { Some(*s) != forbidden_suit && hand.has_any(*s) })
                .collect();

            // Should we do an opening ? (partner is last and i have an unplayed color)
            let nb_players = self.stats.players.len();
            let last_pos = (mepos + nb_players - 1) % nb_players;
            if self.stats.players[last_pos].is_partner(me) == Some(true) { //We should do an opening
                let unplayed_suits: Vec<&Suit> = playable_suits.iter()
                    .filter(|s| !self.stats.suit_already_played(**s))
                    .collect();
                if let Some(unplayed_suit) = unplayed_suits.last() {
                    if let Some(lowest) = hand.suit_lowest(**unplayed_suit) {
                        if lowest.rank() < Rank::RankJ { // we don't play points
                            // print!("an opening.. ");
                            return Some(lowest);
                        }
                    } 
                }
            } else { //We shouldn't do an opening
                let played_suits: Vec<&Suit> = playable_suits.iter()
                    .filter(|s| self.stats.suit_already_played(**s))
                    .collect();
                if let Some(played_suit) = played_suits.last() {
                    if let Some(lowest) = hand.suit_lowest(**played_suit) {
                        if lowest.rank() < Rank::RankJ { // we don't play points
                            // print!("not an opening.. ");
                            return Some(lowest);
                        }
                    } 
                }
            }
            
            //Sort by number of cards so we can get the long suit
            playable_suits.sort_by(|a, b| {
                hand.get_suit_cards(&a).len().cmp(&hand.get_suit_cards(&b).len())
            });

            // Play small card of long suit
            if let Some(long_suit) = playable_suits.last() {
                // print!("small card of long suite.. ");
                let lowest = hand.suit_lowest(*long_suit).unwrap(); // this card exists because we previously filtered suits with hand.has_any()
                if lowest.rank() < Rank::RankJ { // we don't play points
                    return Some(lowest);
                }
            }

            // if we are here, we should only have trumps left (or points we don't want to give), play the highest
            let card = hand.suit_highest(Suit::Trump);
            if card.is_some() {
                return card;
            }
        }

        // print!("default play..  ");
        //Low playable card
        let mut playable: Vec<Card> = hand.list().into_iter().filter(|card| {
            can_play(self.my_state().pos, *card, hand, &deal.last_trick, deal.king, deal.trick_count == 1).is_ok()
        })
        .collect();
        playable.sort_by(|a, b| a.rank().cmp(& b.rank()));
        playable.first().map(|c| *c)
    }

    fn play_try_save_petit(&self, is_first_player: bool) -> Option<Card> {
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;

        let vingtetun = Card::new(Suit::Trump, Rank::Rank21);
        let petit = Card::new(Suit::Trump, Rank::Rank1);

        let table_layout_ok = (self.stats.teams_known_by_all && self.stats.clone().partner_is_after(trick, self.my_state().pos.to_n()) == Some(true))
                || (self.my_state().role == PlayerRole::Taker && is_first_player );

        let petit_not_played = self.stats.suit_left.get(&Suit::Trump).unwrap().has(petit);

        // TODO : cas du petit montré dans une poignée
        //
        if petit_not_played && 
           hand.has(vingtetun) && 
           table_layout_ok
           { Some(vingtetun) } else { None }
    }

    fn play_try_own_petit(&self) -> Option<Card> {
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;

        let vingtetun = Card::new(Suit::Trump, Rank::Rank21);
        let petit = Card::new(Suit::Trump, Rank::Rank1);

        if !hand.has(petit) || trick.suit().is_none() {
            return None;
        } 

        let mepos = self.my_state().pos.to_n();
        let me = &self.stats.players[mepos];


        let win_pos = trick.winner.pos as usize;
        let winner_card = trick.cards[win_pos].unwrap();

        if winner_card.suit() == Suit::Trump {
            // If I have a higher trump than the winner card, I must play it
            if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, winner_card){
                return None;
            }
        }

        //My partner has played the highest trump
        if self.stats.players[win_pos].is_partner(me) == Some(true) {
            let trumps_left_highest = self.stats.suit_left[&Suit::Trump].suit_highest(Suit::Trump);
            if let Some(highest_left) = trumps_left_highest {
                if highest_left.rank() < winner_card.rank() {
                    return Some(petit)
                }
            } else { // No trumps left
                return Some(petit)
            }
        }

        if winner_card.suit() != Suit::Trump { // No trump has been played
            if self.stats.is_trick_last_player(trick, mepos) {
                return Some(petit)
            }

            let nb_players = self.stats.players.len();
            // This is the first time we play this suit 
            // and I am in the taker team or the taker already played 
            if self.is_first_time_suit_played(&winner_card.suit()) &&
               ( me.in_taker_team == Some(true) || 
                 trick.clone().player_already_played(PlayerPos::from_n(self.get_taker_pos().unwrap() as usize, nb_players as u8)))  {
                   return Some(petit);
            }
        }
        None
    }

    fn is_first_time_suit_played(&self, suit: &Suit) -> bool {
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;
        let card_played_count = trick.cards.iter()
            .map(|oc| oc.map(|c| c.suit()) )
            .filter(|s| s == &Some(*suit))
            .collect::<Vec<Option<Suit>>>()
            .len();
        let suit_left_count = self.stats.suit_left[suit].size();
        let my_count = hand.get_suit_cards(suit).len();

        suit_left_count + card_played_count + my_count == 14
    }
}
