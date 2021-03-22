use tungstenite::Message as TMessage;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;


use std::{thread, time};
use std::collections::HashMap;
 
use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::{Deck, Card, Hand, Suit, Rank}, deal::can_play, bid::Target, points::strength, pos::PlayerPos, trick::Trick};
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState, BidCommand, CallKingCommand, MakeDogCommand, Turn, PlayerRole};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

struct DealStats {
    pub players: Vec<PlayerStats>,
    pub suit_left: HashMap<Suit, Hand>,
    pub teams_known: bool,
}

impl DealStats {
    fn new() -> Self {
        DealStats { 
            players: vec![],
            suit_left: HashMap::default(),
            teams_known: false
        }
    }

    fn init_state(&mut self, nb_players: usize, hand: Hand) {
        self.players = vec![ PlayerStats::new() ; nb_players];
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

    fn partner_is_after(self, trick: Trick, me: PlayerStats ) -> Option<bool> {
        if self.teams_known {
            let is_after = self.players.iter().any(|player| {
                player.is_partner(&me) == Some(true) && !trick.player_already_played(player.get_playing_pos().to_n())
            });
            Some(is_after)
        } else if (self.is_trick_last_player(trick, me)){
            Some(false)
        }
        None
    }

    fn opponent_is_after(self, trick: Trick, me: PlayerStats ) -> Option<bool> {
        if self.teams_known {
            let is_after = self.players.iter().any(|player| {
                !player.is_partner(me) && !trick.player_already_played(player.get_playing_pos().to_n())
            });
            Some(is_after)
        } else if (self.is_trick_last_player(trick, me)){
            Some(false)
        }
        None
    }

    //Last player of the trick ?
    fn is_trick_last_player(&self, trick: Trick, me: PlayerStats) -> bool {
        trick.first.prev() == me.pos
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
        if (player.in_taker_team.is_none() || self.in_taker_team.is_none() ) {
            None
        } else {
            Some(player.in_taker_team == self.in_taker_team)
        }
    }
}

pub struct SocketPlayer {
    socket: TarotSocket,
    join_code: String,
    game_state: GameStateSnapshot,
    player_info: PlayerInfo,
    stats: DealStats,
}

impl Drop for SocketPlayer {
    fn drop(&mut self) {
        self.socket.close(None);
    }
}

impl SocketPlayer {
    pub fn new(socket: TarotSocket, join_code: String, nickname: String) -> Self {
        SocketPlayer { 
            socket,
            join_code,
            game_state: GameStateSnapshot::default(),
            player_info: PlayerInfo { id: Uuid::default(), nickname } ,
            stats: DealStats::new(),
        }
    }
    
    pub fn play(&mut self){
        self.send(&Command::Authenticate(AuthenticateCommand { nickname: self.player_info.nickname.clone() }));
        loop {
            let msg = self.socket.read_message().expect("Error reading message");
            let msg = match msg {
                tungstenite::Message::Text(s) => { s }
                _ => { panic!() }
            };

            let message: Message = serde_json::from_str(&msg).expect("Can't parse JSON");
            self.handle_server_message(message);
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
            self.stats.players.iter().for_each(|p| println!("{:?} {:?}", p.played.to_string(), p.in_taker_team));
            println!("suits cards not played for {}:", self.player_info.nickname);
            self.stats.suit_left.iter().for_each(|(s, h)| println!("{:?} {}", s, h.to_string()));
        }
    }

    fn update_partners(&mut self) {
        let deal = &self.game_state.deal;
        if self.stats.teams_known { println!("teams known"); return ()};
        let taker_opt = self.game_state.players.iter()
            .find(|p| p.role == PlayerRole::Taker);
        if let Some(taker) = taker_opt {
            self.stats.players[taker.pos.to_n()].is_taker = true;
            self.stats.players[taker.pos.to_n()].in_taker_team = Some(true);
        }

        if let Some(king) = deal.king {
            let partner_pos: Option<usize> = if deal.hand.has(king) {
                    // I have king
                    Some(self.my_state().pos.to_n())
                } else {
                    // searching king in players played cards
                    self.stats.players.iter()
                        .enumerate()
                        .find(|(idx, pstat)| pstat.played.has(king))
                        .map(|(idx, pstat)| idx)
                };
            if let Some(pos) = partner_pos {
                for (idx, pstat) in  self.stats.players.iter_mut().enumerate() {
                    pstat.in_taker_team = Some(pstat.is_taker || idx == pos);
                }
                self.stats.teams_known = true;
            } else { // King not played
                //Check players who don't have cards of the king's suit
                for pstat in  self.stats.players.iter_mut() {
                    if Some(false) == pstat.suits_available[&king.suit()] {
                        pstat.in_taker_team = Some(false);
                    }
                }
            }

        } else { // Aucun roi appelé : le preneur est seul 
            self.stats.players.iter_mut().map(|pstat|
                pstat.in_taker_team = Some(pstat.is_taker)
            );
            self.stats.teams_known = true;
        }
    }

    fn my_state(&self) -> &GamePlayerState {
        self.game_state
            .players
            .iter()
            .find(|state| state.player.id == self.player_info.id)
            .unwrap()
    }

    fn send(&mut self, command: &Command) -> Result<()> {
        let json = serde_json::to_string(command)?;
        self.socket.write_message(TMessage::Text(json)).unwrap();
        Ok(())
    }

    fn handle_server_message(&mut self, msg: Message){
        match msg {
            Message::Authenticated(player_info) => {
                self.player_info = player_info;
                println!("Authenticated with id {}", self.player_info.id);
                // if self.check_join_code() {
                    self.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                // }
            }
            Message::GameJoined(game_info) => {
                println!("Game joined: {}", game_info.game_id);
                self.send(&Command::MarkReady);
            }
            Message::GameStateSnapshot(game_state) => {
                self.game_state = game_state;
                self.handle_new_state();
            }
            Message::Chat(_) => {}
            Message::Pong => {
                println!("Received a pong !!");
            }
            Message::ServerStatus(server_status) => {
                server_status.games.iter()
                    .next()
                    .map(|g| {
                        println!("Found a game with code {}", g.game.join_code);
                        self.join_code = g.game.join_code.clone();
                        self.send(&Command::JoinGame(JoinGameCommand { join_code: self.join_code.clone(), }));
                    });
            }
            _ => {
                println!("Unmanaged server message for {}: {:?}", self.player_info.nickname, msg);
            }
        }
    }

    fn handle_new_state(&mut self){
        self.update_stats();
        let my_state = self.my_state();
        // let card_played = self.game_state.deal.last_trick.card_played(my_state.pos);
        let player_action = my_state.get_turn_player_action(self.game_state.turn);
        // let mypos = my_state.pos.to_n();
        // let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        match player_action {
            Some(PlayerAction::Bid) => {
                //deal has started, we can init its state
                self.stats.init_state(self.game_state.nb_players as usize, self.game_state.deal.hand);
                if let Some(target) = self.guess_bid() {
                    self.send(&Command::GamePlay(GamePlayCommand::Bid(BidCommand { target, slam: false })));
                } else {
                    self.send(&Command::GamePlay(GamePlayCommand::Pass));
                }
            }
            Some(PlayerAction::CallKing) => {
                self.send(&Command::GamePlay(GamePlayCommand::CallKing(CallKingCommand { card: self.call_king() })));
            }
            Some(PlayerAction::MakeDog) => {
                let dog  = self.make_dog();
                for card in dog.list() {
                    (*self.stats.suit_left.get_mut(&card.suit()).unwrap()).remove(card);
                }
                self.send(&Command::GamePlay(GamePlayCommand::MakeDog(MakeDogCommand { cards: dog, slam: false })));
            }
            Some(PlayerAction::Play) => {
                self.choose_card().map(|card| {
                    println!("{} is playing {}...", self.player_info.nickname, card.to_string());
                    self.send(&Command::GamePlay(GamePlayCommand::Play(PlayCommand { card })));
                });
            }
            _ => {}
        }
    }

    fn guess_bid(&self) -> Option<Target>{
        let curr_target = &self.game_state.deal.contract_target();

        let points = self.evaluate_hand();
        let candidate = if points < 40 {
            None
        } else if points < 56 {
            Some(Target::Prise)
        } else if points < 71 {
            Some(Target::Garde)
        } else if points < 81 {
            Some(Target::GardeSans)
        } else {
            Some(Target::GardeContre)
        };
        candidate.filter(|bidtarget| curr_target.lt(&Some(*bidtarget)))
    }

    // cf. https://www.le-tarot.fr/quel-contrat-choisir/
    fn evaluate_hand(&self) -> usize {
        let mut points = 0;

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

    fn make_dog(&self) -> Hand {
        //Let the players see the initial dog
        let delay = time::Duration::from_millis(3000);
        let now = time::Instant::now();
        thread::sleep(delay);

        let mut dog = Hand::new();
        let deal = &self.game_state.deal;
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

        // TODO Play the excuse if trumps and no points for me

        let danger = self.stats.opponent_is_after(trick, self.my_state().pos) != Some(false);

        if let Some(starting_suit) = trick.suit() { // Not the first to play
            let winner_card = trick.cards[trick.winner.pos as usize].unwrap();

            if starting_suit == Suit::Trump {
                // Try to save the petit
                let found = self.play_try_save_petit(false);
                if found.is_some() { return found };
            } else {
                // TODO Play the long suit when no oudlers left (or to make adversaries cut)
                
                let my_highest = hand.suit_highest(starting_suit);
                let highest_left = self.stats.suit_left.get(&starting_suit).unwrap().suit_highest(starting_suit);
                if my_highest.is_some() {
                    let mut myhighest = my_highest.unwrap();
                    if myhighest > winner_card { // I can win the trick
                        // If not points, take the lowest still winning  
                        if myhighest.rank() < Rank::RankJ {
                            return hand.suit_lowest_over_card(starting_suit, winner_card).unwrap();
                        }

                        if highest_left.is_some() {
                            if  myhighest > highest_left.unwrap() && !danger{
                                return Some(myhighest);
                            }
                        } else {
                            if let Some(mylowest) = hand.suit_lowest(starting_suit){
                                return mylowest;
                            }
                            // TODO give points if my parter win the trick
                        }


                    }

                } else { // I must cut or piss
                    if let Some(mylowest) = hand.suit_lowest_over_card(Suit::Trump, petit){
                        return mylowest;
                    }
                }
            }
            
        } else {//First to play 
            let found = self.play_try_save_petit(true);
            if found.is_some() { return found };
            // TODO taker_team & have king : play king
            // TODO not taker team : play small card of long suit
            if (deal.trick_count == 1){ // first trick 
            } else { // Not first trick
            }
        }

        //Random playable card
        hand.list().iter().find(|card| {
            can_play(self.my_state().pos, **card, hand, &deal.last_trick, deal.king, deal.trick_count == 1).is_ok()
        }).map(|c| *c)
    }

    fn play_try_save_petit(&self, is_first_player: bool) -> Option<Card> {
        let deal = &self.game_state.deal;
        let trick = &deal.last_trick;
        let hand = deal.hand;

        let vingtetun = Card::new(Suit::Trump, Rank::Rank21);
        let petit = Card::new(Suit::Trump, Rank::Rank1);

        let table_layout_ok = self.stats.partner_is_after(trick, self.my_state().pos) == Some(true)
                || (self.my_state().is_taker && is_first_player );

        let petit_not_played = self.stats.suit_left.get(Suit::Trump).unwrap().has(petit);
        // TODO : je suis le preneur et je débute le trick

        // TODO : cas du petit montré dans une poignée
        //
        if (petit_not_played && 
            hand.has(vingtetun) && 
            table_layout_ok
        ) { Some(vingtetun) } else { None }
    }
}
