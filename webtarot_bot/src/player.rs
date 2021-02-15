use tungstenite::Message as TMessage;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;


use std::{thread, time};
use std::collections::HashMap;
 
use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::{Card, Hand, Suit, Rank}, deal::can_play, bid::Target, points::strength, pos::PlayerPos};
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState, BidCommand, CallKingCommand, MakeDogCommand, Turn, PlayerRole};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

struct DealStats {
    pub players: Vec<PlayerStats>,
    pub teams_known: bool,
}

impl DealStats {
    fn new() -> Self {
        DealStats { players: vec![], teams_known: false }
    }

    fn init_state(&mut self, nb_players: usize) {
        self.players = vec![ PlayerStats::new() ; nb_players];
    }
}


#[derive(Clone, Debug)]
struct PlayerStats {
    played: Hand,
    is_taker: bool,
    in_taker_team: Option<bool>,
    has_heart: bool,
    has_club: bool,
    has_spade: bool,
    has_diamond: bool,
    has_trump: bool,
}

impl PlayerStats {
    fn new() -> Self {
        PlayerStats {
            played: Hand::new(), 
            is_taker: false,
            in_taker_team: None,
            has_heart: true,
            has_club: true,
            has_spade: true,
            has_diamond: true,
            has_trump: true,
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
        // let cards = self.game_state.deal.last_trick.cards.clone();
        let cards = self.game_state.deal.last_trick.cards;
        cards.iter().enumerate().for_each(|(pos, card)| {
            card.map(|c| self.stats.players[pos].played.add(c));
        });

        if let Turn::Playing(_) = self.game_state.turn {
            self.update_partners();
        }
        self.stats.players.iter().for_each(|p| println!("{:?} {:?}", p.played.to_string(), p.in_taker_team));
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
                println!("i have king ({})", king.to_string());
                Some(self.my_state().pos.to_n())
            } else {
                println!("searching king in played cards");
                self.stats.players.iter()
                    .enumerate()
                    .find(|(idx, pstat)| pstat.played.has(king))
                    .map(|(idx, pstat)| idx)
            };

            if let Some(pos) = partner_pos {
                for (idx, pstat) in  self.stats.players.iter_mut().enumerate() {
                    pstat.in_taker_team = Some(pstat.is_taker || idx == pos);
                    println!("taker {:?}, p{} <> id{}",pstat.is_taker, pos, idx);
                }
                self.stats.teams_known = true;
            }
        } else {
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
                self.stats.init_state(self.game_state.nb_players as usize);
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
                self.send(&Command::GamePlay(GamePlayCommand::MakeDog(MakeDogCommand { cards: self.make_dog(), slam: false })));
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
        let hand = deal.hand;
        let excuse = Card::new(Suit::Trump, Rank::Rank22);

        //Play the excuse before the last trick
        if hand.size() == 2 && hand.has(excuse) {
            return Some(excuse);
        }
        //Play the excuse if trumps and no points for me

        //Random playable card
        hand.list().iter().find(|card| {
            can_play(self.my_state().pos, **card, hand, &deal.last_trick, deal.king, deal.trick_count == 1).is_ok()
        }).map(|c| *c)
    }

}
