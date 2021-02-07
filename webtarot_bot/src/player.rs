use tungstenite::Message as TMessage;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;


use std::{thread, time};

use rayon::prelude::*;

use uuid::Uuid;
use url::Url;
use serde_json::Result;

use tarotgame::{deal_seeded_hands, cards::{Card, Hand, Suit, Rank}, deal::can_play, bid::Target, points::strength} ;
use webtarot_protocol::{Message, Command, GameStateSnapshot, PlayerAction, GamePlayCommand, PlayCommand, GamePlayerState, BidCommand, CallKingCommand, MakeDogCommand};
use webgame_protocol::{AuthenticateCommand, JoinGameCommand, PlayerInfo};

type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

pub struct SocketPlayer {
    socket: TarotSocket,
    join_code: String,
    game_state: GameStateSnapshot,
    player_info: PlayerInfo,
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
            player_info: PlayerInfo { id: Uuid::default(), nickname } 
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

    pub fn my_state(&self) -> &GamePlayerState {
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
        let semi_second = time::Duration::from_millis(500);
        let now = time::Instant::now();
        thread::sleep(semi_second);

        let my_state = self.my_state();
        // let card_played = self.game_state.deal.last_trick.card_played(my_state.pos);
        let player_action = my_state.get_turn_player_action(self.game_state.turn);
        // let mypos = my_state.pos.to_n();
        // let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        match player_action {
            Some(PlayerAction::Bid) => {
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
        let points = self.evaluate_hand();
        if points < 40 {
            None
        } else if points < 56 {
            Some(Target::Prise)
        } else if points < 71 {
            Some(Target::Garde)
        } else if points < 81 {
            Some(Target::GardeSans)
        } else {
            Some(Target::GardeContre)
        }
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
        //dummy
        Card::new(Suit::Heart, rank)
    }

    fn make_dog(&self) -> Hand {
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
        let mut cards = hand.list();
        let mut card = cards.pop();
        while dog.size() < dog_size && card.is_some() {
            dog.add(card.unwrap());
            card = cards.pop();
        }

        assert!(dog.size() == dog_size);

        //dummy
        dog
    }

    fn choose_card(&self) -> Option<Card>{
        let deal = &self.game_state.deal;
        let hand = deal.hand;
        hand.list().iter().find(|card| {
            can_play(self.my_state().pos, **card, hand, &deal.last_trick, deal.king, deal.trick_count == 1).is_ok()
        }).map(|c| *c)
    }

}
