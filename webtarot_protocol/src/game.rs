use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tarotgame::{NB_PLAYERS, bid, cards, pos, deal, trick};
use webgame_protocol::{GameState, PlayerInfo, ProtocolErrorKind};
use crate::{ ProtocolError };

use crate::turn::Turn;
use crate::deal::{Deal, DealSnapshot};
use crate::player::{PlayerRole, GamePlayerState};

pub struct TarotGameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
    deal: Deal,
    first: pos::PlayerPos,
    scores: Vec<[f32; NB_PLAYERS]>,
}

impl Default for TarotGameState {
    fn default() -> TarotGameState {
        TarotGameState {
            players: BTreeMap::new(),
            turn: Turn::Pregame,
            deal: Deal::new(pos::PlayerPos::P0),
            first: pos::PlayerPos::P0,
            scores: vec![],
        }
    }
}

impl GameState< GamePlayerState, GameStateSnapshot> for TarotGameState {
    type PlayerPos = pos::PlayerPos;
    type PlayerRole = PlayerRole;

    fn is_joinable(&self) -> bool {
        self.turn == Turn::Pregame
    }
    
    fn get_players(&self) -> &BTreeMap<Uuid, GamePlayerState> {
        &self.players
    }

    fn add_player(&mut self, player_info: PlayerInfo) -> pos::PlayerPos {
        if self.players.contains_key(&player_info.id) {
            return self.players.get(&player_info.id).unwrap().pos;
        }

        //Default pos
        let nb_players = self.players.len();
        let mut newpos = pos::PlayerPos::from_n(nb_players);

        //TODO rendre générique
        for p in &[ pos::PlayerPos::P0,
        pos::PlayerPos::P1,
        pos::PlayerPos::P2,
        pos::PlayerPos::P3,
        ] {
            if !self.position_taken(*p){
                newpos = p.clone();
                break;
            }
        }

        let state = GamePlayerState {
            player: player_info,
            // pos: pos::PlayerPos::from_n(nb_players),
            pos: newpos,
            role: PlayerRole::Spectator,
            ready: false,
        };
        self.players.insert(state.player.id, state.clone());
        newpos
    }

    fn remove_player(&mut self, player_id: Uuid) -> bool {
        self.players.remove(&player_id).is_some()
    }

    fn set_player_role(&mut self, player_id: Uuid, role: PlayerRole) {
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.role = role;
        }
    }

    fn player_by_pos(&self, position: pos::PlayerPos) -> Option<&GamePlayerState> {
        self.players.iter().find(|(_uuid, player)| player.pos == position).map(|p| p.1)
    }

    // Creates a view of the game for a player
    fn make_snapshot(&self, player_id: Uuid) -> GameStateSnapshot {
        let contract = self.deal.deal_contract().cloned();
        let mut players = vec![];
        for (&_other_player_id, player_state) in self.players.iter() {
            players.push(player_state.clone());
        }
        players.sort_by(|a, b| a.pos.to_n().cmp(&b.pos.to_n()));
        let pos = self.players[&player_id].pos;
        let mut scores = [0.0; 5];
        let mut dog = cards::Hand::new();
        let mut taker_diff = 0.0;
        let deal = match self.deal.deal_state() {
            Some(state) => { // In Playing phase
                if let deal::DealResult::GameOver {points: _, taker_diff: diff, scores: lscores } = state.get_deal_result() {
                     scores = lscores;
                     taker_diff = diff;
                     dog = state.dog();
                };
                let last_trick = if self.turn == Turn::Intertrick && !self.was_last_trick() {
                    // intertrick : there is at least a trick done
                    state.last_trick().unwrap().clone()
                } else {
                    state.current_trick().clone()
                };
                // log::debug!("trick {:?}", last_trick.cards);
                let initial_dog = if self.turn == Turn::MakingDog {
                    state.dog()
                } else { cards::Hand::new() };
                DealSnapshot {
                    hand: state.hands()[pos as usize],
                    current: state.next_player(),
                    contract,
                    king: state.king(),
                    scores,
                    // last_trick: state.tricks.last().unwrap_or(trick::Trick::default()),
                    last_trick,
                    initial_dog,
                    dog,
                    taker_diff,
                }
            },
            None => DealSnapshot { // In bidding phase
                hand: self.deal.hands()[pos as usize],
                current: self.deal.next_player(),
                contract,
                king: None,
                scores: [0.0;NB_PLAYERS],
                last_trick: trick::Trick::default(),
                initial_dog: cards::Hand::new(),
                dog,
                taker_diff,
            }
        };
        GameStateSnapshot {
            players,
            scores: self.scores.clone(),
            turn: self.turn,
            deal
        }
    }

    fn set_player_ready(&mut self, player_id: Uuid){
        let turn = self.turn.clone();
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.ready = true;
            if turn == Turn::Intertrick {
                self.update_turn();
            } else {
                player_state.role = PlayerRole::PreDeal;

                // Check if we start the next deal
                let mut count = 0;
                for player in self.players.values() {
                    if player.role == PlayerRole::PreDeal {
                        count = count + 1;
                    }
                }
                if count == NB_PLAYERS {
                    if self.turn == Turn::Interdeal { // ongoing game
                        self.update_turn();
                    } else { // new game
                        self.turn = Turn::Bidding((bid::AuctionState::Bidding, pos::PlayerPos::P0));
                    }
                }

            }
        }
    }

    fn set_player_not_ready(&mut self, player_id: Uuid) {
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.ready = false;
        }
    }

}

impl TarotGameState {
    pub fn get_turn(&self) -> Turn {
        self.turn
    }

    fn position_taken(&self, position: pos::PlayerPos) -> bool {
        self.player_by_pos(position) != None
    }

    pub fn players_ready(&self) -> bool {
        !(self.players.iter().find(|(_, player)| player.ready == false) != None)
    }

    pub fn update_turn(&mut self){
        if self.turn == Turn::CallingKing || self.turn == Turn::MakingDog {
            return ();
        }
        self.turn = if !self.players_ready() {
            Turn::Intertrick
        } else if self.was_last_trick() {
            self.end_deal();
            Turn::Interdeal
        } else {
            if self.turn == Turn::Interdeal {
                self.next_deal();
            }
            Turn::from_deal(&self.deal)
        }
    }

    fn was_last_trick(&self) -> bool {
        let p0 = self.player_by_pos(pos::PlayerPos::P0).unwrap();
        self.turn == Turn::Intertrick && p0.role == PlayerRole::Unknown
    }

    pub fn set_bid(&mut self, pid: Uuid, target: bid::Target) -> Result<(), ProtocolError>{
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = self.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.bid(pos, target) {
            self.complete_auction()?;
        }
        self.update_turn();
        Ok(())
    }

    pub fn set_pass(&mut self, pid: Uuid) -> Result<(), ProtocolError> {
        let pos = self.players.get(&pid).map(|p| p.pos).ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "unknown position")
            )?;
        let auction = self.deal.deal_auction_mut().ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "unknown auction")
        )?;
        let pass_result = auction.pass(pos);
        match pass_result {
            Ok(bid::AuctionState::Over) => self.complete_auction()?,
            Ok(bid::AuctionState::Cancelled) => self.next_deal(),
            _ => ()
        };
        self.update_turn();
        Ok(())
    }

    pub fn call_king(&mut self, pid: Uuid, card: cards::Card){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let deal_state = self.deal.deal_state_mut().unwrap();
        if deal_state.call_king(pos, card) {
            // Next step : do we need to make a dog ?
            let target = self.deal.deal_contract().unwrap().target;
            if target == bid::Target::GardeSans || target == bid::Target::GardeContre {
                //No dog
                self.turn = Turn::from_deal(&self.deal);
            } else {
                //Dog
                self.turn = Turn::MakingDog;
            }
        }
    }

    pub fn make_dog(&mut self, pid: Uuid, cards: cards::Hand){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        if self.deal.deal_state_mut().unwrap().make_dog(pos, cards) {
            self.turn = Turn::from_deal(&self.deal);
        }
    }

    pub fn set_play(&mut self, pid: Uuid, card: cards::Card) -> Result<(), ProtocolError> {
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();
        let state = self.deal.deal_state_mut().ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "Unknown deal state")
        )?;
        match state.play_card(pos, card)? {
            deal::TrickResult::Nothing => (),
            deal::TrickResult::TrickOver(_winner, deal::DealResult::Nothing) => self.end_trick(),
            deal::TrickResult::TrickOver(_winner, deal::DealResult::GameOver{points: _, taker_diff: _, scores}) => {
                self.scores.push(scores);
                self.end_last_trick();
            }
        }
        self.update_turn();
        Ok(())
    }

    fn complete_auction(&mut self)  -> Result<(), ProtocolError>{
        let deal_state = match &mut self.deal {
            &mut Deal::Playing(_) => unreachable!(),
            &mut Deal::Bidding(ref mut auction) => auction.complete()?
        };
        self.deal = Deal::Playing(deal_state);

        //Set taker role
        let taker_pos = self.deal.deal_contract().unwrap().author;
        let taker_id = self.player_by_pos(taker_pos).unwrap().player.id;
        self.set_player_role( taker_id, PlayerRole::Taker);

        //Update turn
        self.turn = if NB_PLAYERS == 5 {
            Turn::CallingKing
        } else {
            Turn::MakingDog
        };
        Ok(())
    }

    fn end_trick(&mut self) {
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
            }
        }
    }

    fn end_last_trick(&mut self) {
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
                player.role = PlayerRole::Unknown;
            }
        }
    }

    fn end_deal(&mut self) {
        self.turn = Turn::Interdeal;
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
            }
        }
    }

    fn next_deal(&mut self) {
        self.first = self.first.next();
        let auction = bid::Auction::new(self.first);
        self.deal = Deal::Bidding(auction);
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayEvent {
    Play( Uuid, cards::Card)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameStateSnapshot {
    pub players: Vec<GamePlayerState>,
    pub turn: Turn,
    pub deal: DealSnapshot,
    pub scores: Vec<[f32; NB_PLAYERS]>,
}

impl webgame_protocol::GameStateSnapshot for GameStateSnapshot {

}

impl GameStateSnapshot {
    // pub fn get_current_player(self) -> Option<PlayerInfo> {
    //     let player_info;
    //     match self.turn {
    //         Turn::Playing(pos) => player_info = Some(self.players[pos.to_n()].player.clone()),
    //         Turn::Bidding((_, pos)) => player_info = Some(self.players[pos.to_n()].player.clone()),
    //         _ => player_info = None
    //     }
    //     player_info
    // }
    pub fn get_playing_pos(&self) -> Option<pos::PlayerPos> {
        match self.turn {
            Turn::Playing(pos) => Some(pos),
            Turn::Bidding((_, pos)) => Some(pos),
            _ => None
        }
    }

    pub fn pos_player_name(&self, pos: pos::PlayerPos) -> String {
        self.players.iter()
            .find(|p| p.pos == pos)
            .map(|found| &found.player.nickname)
            .unwrap() // panic on invalid pos
            .into()
    }

    pub fn current_player_name(&self) -> String {
        let found_name = self.get_playing_pos().map(|pos| {
            self.pos_player_name(pos)
        });

        if let Some(name) = found_name {
            format!("{}", name)
        } else {
            "".into()
        }
    }

    // pub fn get_playing_player(&self) -> Option<&GamePlayerState> {
    //     self.get_playing_player().map(|pos| {
    //         self.players.iter()
    //             .find(|p| p.pos == pos)
    //             .map(|found| &found)
    //     }).flatten()
    // }

}

impl Default for GameStateSnapshot {
    fn default() -> GameStateSnapshot {
        let pos = pos::PlayerPos::P0; // could be anything
        GameStateSnapshot {
            players: vec![],
            scores: vec![],
            turn: Turn::Pregame,
            deal: DealSnapshot {
                hand: cards::Hand::new(),
                current: pos,
                contract: None,
                king: None,
                scores: [0.0;NB_PLAYERS],
                last_trick: trick::Trick::new(pos),
                initial_dog: cards::Hand::new(),
                dog: cards::Hand::new(),
                taker_diff: 0.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tarotgame::{deal_hands, deal_seeded_hands, bid, cards};
    // use tarotgame::{bid, cards, pos, deal, trick};

    #[test]
    fn test_all_pass() {
        let mut game = TarotGameState::default();
        let pos0 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player0")});
        let pos1 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player1")});
        let pos2 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player2")});
        let pos3 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player3")});
        let pos4 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player4")});

        assert_eq!(true, game.is_joinable());

        let id0 = game.player_by_pos(pos0).unwrap().player.id;
        let id1 = game.player_by_pos(pos1).unwrap().player.id;
        let id2 = game.player_by_pos(pos2).unwrap().player.id;
        let id3 = game.player_by_pos(pos3).unwrap().player.id;
        let id4 = game.player_by_pos(pos4).unwrap().player.id;
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(false, game.is_joinable());
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos0)));

        let hands_deal1 = game.deal.hands();

        game.set_pass(id0).unwrap();
        game.set_pass(id1).unwrap();
        game.set_pass(id2).unwrap();
        game.set_pass(id3).unwrap();
        game.set_pass(id4).unwrap();

        //5 passes : should start a new deal
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos1)));
        assert_ne!(game.deal.hands(), hands_deal1);
    }

    #[test]
    fn test_game() {
        let mut game = TarotGameState::default();
        let pos0 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player0")});
        let pos1 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player1")});
        let pos2 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player2")});
        let pos3 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player3")});
        let pos4 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player4")});

        assert_eq!(true, game.is_joinable());

        let id0 = game.player_by_pos(pos0).unwrap().player.id;
        let id1 = game.player_by_pos(pos1).unwrap().player.id;
        let id2 = game.player_by_pos(pos2).unwrap().player.id;
        let id3 = game.player_by_pos(pos3).unwrap().player.id;
        let id4 = game.player_by_pos(pos4).unwrap().player.id;
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(false, game.is_joinable());
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos0)));

        let seed = [3, 32, 3, 32, 54, 1, 84, 3, 32, 54, 1, 84, 3, 32, 65, 1, 84, 3, 32, 64, 1, 44, 3, 32, 54, 1, 84, 3, 32, 65, 1, 44];
        let (hands, dog) = deal_seeded_hands(seed);
        // println!("{}", _dog.to_string());
        // for hand in hands.iter() {
        //     println!("{}", hand.to_string()); // `cargo test -- --nocapture` to view output
        // }
        // println!("-------------------");

        // Dog : 4♥,      10♠,Q♠,
        // [C♥,Q♥,        2♠,C♠,        2♦,3♦,7♦,        5♣,8♣,10♣,       3T,8T,11T,12T,21T,]
        // [6♥,K♥,        9♠,           5♦,6♦,J♦,        4♣,              1T,5T,7T,9T,16T,17T,20T,ET,]
        // [1♥,2♥,8♥,9♥,  4♠,8♠,K♠,     10♦,             3♣,J♣,           10T,14T,15T,18T,19T,]
        // [7♥,J♥,        1♠,3♠,5♠,J♠,  1♦,9♦,           1♣,2♣,6♣,9♣,Q♣,  2T,4T,]
        // [3♥,5♥,10♥,    6♠,7♠,        4♦,8♦,C♦,Q♦,K♦,  7♣,C♣,K♣,        6T,13T,]

        let auction = game.deal.deal_auction_mut().unwrap();
        auction.set_hands(hands, dog);

        game.set_bid(id0, bid::Target::Garde).unwrap();
        game.set_pass(id1).unwrap();
        game.set_pass(id2).unwrap();
        game.set_pass(id3).unwrap();
        game.set_pass(id4).unwrap();
        assert_eq!(game.get_turn(), Turn::CallingKing);
        assert_eq!(game.player_by_pos(pos0).unwrap().role, PlayerRole::Taker);
        game.call_king(id0, cards::Card::new(cards::Suit::Club, cards::Rank::RankK));
        assert_eq!(game.get_turn(), Turn::MakingDog);
        let mut dog = cards::Hand::new();
        dog.add(cards::Card::new(cards::Suit::Heart, cards::Rank::RankC));
        dog.add(cards::Card::new(cards::Suit::Spade, cards::Rank::Rank2));
        dog.add(cards::Card::new(cards::Suit::Spade, cards::Rank::RankQ));
        game.make_dog(id0, dog);
        assert_eq!(true, game.players_ready());
        assert_eq!(game.get_turn(), Turn::Playing(pos0));

        // Dog : C♥,      2♠,Q♠,
        // [4♥,Q♥,        10♠,C♠,        2♦,3♦,7♦,        5♣,8♣,10♣,       3T,8T,11T,12T,21T,]
        // [6♥,K♥,        9♠,           5♦,6♦,J♦,        4♣,              1T,5T,7T,9T,16T,17T,20T,ET,]
        // [1♥,2♥,8♥,9♥,  4♠,8♠,K♠,     10♦,             3♣,J♣,           10T,14T,15T,18T,19T,]
        // [7♥,J♥,        1♠,3♠,5♠,J♠,  1♦,9♦,           1♣,2♣,6♣,9♣,Q♣,  2T,4T,]
        // [3♥,5♥,10♥,    6♠,7♠,        4♦,8♦,C♦,Q♦,K♦,  7♣,C♣,K♣,        6T,13T,]
        game.set_play(id0, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank2)).unwrap();
        assert_eq!(game.get_turn(), Turn::Playing(pos1));
        game.set_play(id1, cards::Card::new(cards::Suit::Diamond, cards::Rank::RankJ)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank10)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank9)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank8)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);

        // [4♥,Q♥,        10♠,C♠,        3♦,7♦,        5♣,8♣,10♣,       3T,8T,11T,12T,21T,]        
        // [6♥,K♥,        9♠,           5♦,6♦,        4♣,              1T,5T,7T,9T,16T,17T,20T,ET]
        // [1♥,2♥,8♥,9♥,  4♠,8♠,K♠,     ,             3♣,J♣,           10T,14T,15T,18T,19T,]
        // [7♥,J♥,        1♠,3♠,5♠,J♠,  1♦,           1♣,2♣,6♣,9♣,Q♣,  2T,4T,]
        // [3♥,5♥,10♥,    6♠,7♠,        4♦,C♦,Q♦,K♦,  7♣,C♣,K♣,        6T,13T,]
        game.set_play(id1, cards::Card::new(cards::Suit::Club, cards::Rank::Rank4)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Club, cards::Rank::Rank3)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Club, cards::Rank::Rank1)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Club, cards::Rank::RankC)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Club, cards::Rank::Rank5)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos4));

        // [4♥,Q♥,        10♠,C♠,        3♦,7♦,        8♣,10♣,       3T,8T,11T,12T,21T,]        
        // [6♥,K♥,        9♠,           5♦,6♦,                      1T,5T,7T,9T,16T,17T,20T,ET]
        // [1♥,2♥,8♥,9♥,  4♠,8♠,K♠,     ,             J♣,           10T,14T,15T,18T,19T,]       
        // [7♥,J♥,        1♠,3♠,5♠,J♠,  1♦,           2♣,6♣,9♣,Q♣,  2T,4T,]                    
        // [3♥,5♥,10♥,    6♠,7♠,        4♦,C♦,Q♦,K♦,  7♣,K♣,        6T,13T,]                   
        game.set_play(id4, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank6)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Spade, cards::Rank::RankC)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank9)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank8)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank5)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos0));

        // [4♥,Q♥,        10♠,        3♦,7♦,        8♣,10♣,       3T,8T,11T,12T,21T,]        
        // [6♥,K♥,                   5♦,6♦,                      1T,5T,7T,9T,16T,17T,20T,ET]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,             J♣,           10T,14T,15T,18T,19T,]       
        // [7♥,J♥,        1♠,3♠,J♠,  1♦,           2♣,6♣,9♣,Q♣,  2T,4T,]                    
        // [3♥,5♥,10♥,    7♠,        4♦,C♦,Q♦,K♦,  7♣,K♣,        6T,13T,]                   
        game.set_play(id0, cards::Card::new(cards::Suit::Club, cards::Rank::Rank8)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank1)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Club, cards::Rank::RankJ)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Club, cards::Rank::Rank2)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Club, cards::Rank::Rank7)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [4♥,Q♥,        10♠,        3♦,7♦,        10♣,       3T,8T,11T,12T,21T,]     
        // [6♥,K♥,                   5♦,6♦,                   5T,7T,9T,16T,17T,20T,ET]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                        10T,14T,15T,18T,19T,]    
        // [7♥,J♥,        1♠,3♠,J♠,  1♦,           6♣,9♣,Q♣,  2T,4T,]                 
        // [3♥,5♥,10♥,    7♠,        4♦,C♦,Q♦,K♦,  K♣,        6T,13T,]                
        game.set_play(id1, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank6)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank10)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank1)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank4)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank7)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos2));

        // [4♥,Q♥,        10♠,        3♦,        10♣,       3T,8T,11T,12T,21T,]     
        // [6♥,K♥,                   5♦,                   5T,7T,9T,16T,17T,20T,ET]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                     14T,15T,18T,19T,]
        // [7♥,J♥,        1♠,3♠,J♠,             6♣,9♣,Q♣,  2T,4T,]                
        // [3♥,5♥,10♥,    7♠,        C♦,Q♦,K♦,  K♣,        6T,13T,]               
        game.set_play(id2, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank14)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank2)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank6)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank21)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank22)).unwrap();
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos0));

        // [4♥,Q♥,        10♠,        3♦,        10♣,       3T,8T,11T,12T,]     
        // [6♥,K♥,                   5♦,                   5T,7T,9T,16T,17T,20T]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                     15T,18T,19T,]
        // [7♥,J♥,        1♠,3♠,J♠,             6♣,9♣,Q♣,  4T,]                
        // [3♥,5♥,10♥,    7♠,        C♦,Q♦,K♦,  K♣,        13T,]               
        game.set_play(id0, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank3)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank5)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank15)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank4)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank13)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos2));

        // [4♥,Q♥,        10♠,        3♦,        10♣,       8T,11T,12T,]     
        // [6♥,K♥,                   5♦,                   7T,9T,16T,17T,20T]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                     18T,19T,]
        // [7♥,J♥,        1♠,3♠,J♠,             6♣,9♣,Q♣,  ]                
        // [3♥,5♥,10♥,    7♠,        C♦,Q♦,K♦,  K♣,        ]               
        game.set_play(id2, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank19)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Club, cards::Rank::RankQ)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank7)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank12)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank20)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [4♥,Q♥,        10♠,        3♦,        10♣,    8T,11T,]     
        // [6♥,K♥,                   5♦,                7T,9T,16T,17T]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                  18T,]
        // [7♥,J♥,        1♠,3♠,J♠,             6♣,9♣,  ]                
        // [3♥,5♥,10♥,               C♦,Q♦,K♦,  K♣,        ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank5)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank18)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Club, cards::Rank::Rank9)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Diamond, cards::Rank::RankK)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Diamond, cards::Rank::Rank3)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos2));

        // [4♥,Q♥,        10♠,                10♣,    8T,11T,]     
        // [6♥,K♥,                                   7T,9T,16T,17T]
        // [1♥,2♥,8♥,9♥,  4♠,K♠,     ,                  ]
        // [7♥,J♥,        1♠,3♠,J♠,          6♣, ]                
        // [3♥,5♥,10♥,               C♦,Q♦,  K♣,        ]               
        game.set_play(id2, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank9)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Heart, cards::Rank::RankJ)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank10)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Heart, cards::Rank::RankQ)).unwrap();
        game.set_play(id1, cards::Card::new(cards::Suit::Heart, cards::Rank::RankK)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [4♥,        10♠,                10♣,    8T,11T,]     
        // [6♥,                                   7T,9T,16T,17T]
        // [1♥,2♥,8♥,  4♠,K♠,     ,                  ]
        // [7♥,        1♠,3♠,J♠,          6♣, ]                
        // [3♥,5♥,               C♦,Q♦,  K♣,        ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank17)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank1)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank7)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank3)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank11)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [4♥,        10♠,                10♣,    8T,
        // [6♥,                                   7T,9T,16T
        // [2♥,8♥,  4♠,K♠,     ,                  ]
        // [        1♠,3♠,J♠,          6♣, ]                
        // [5♥,               C♦,Q♦,  K♣,        ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank16)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank2)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Club, cards::Rank::Rank6)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank5)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank8)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [4♥,        10♠,                10♣,    
        // [6♥,                                   7T,9T
        // [8♥,  4♠,K♠,     ,                  ]
        // [        1♠,3♠,J♠,           ]                
        // [               C♦,Q♦,  K♣,        ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank9)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank8)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank1)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Club, cards::Rank::RankK)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank4)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [        10♠,                10♣,    
        // [6♥,                                   7T
        // [  4♠,K♠,     ,                  ]
        // [        3♠,J♠,           ]                
        // [               C♦,Q♦,          ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Trump, cards::Rank::Rank7)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank4)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank3)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Diamond, cards::Rank::RankC)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Spade, cards::Rank::Rank10)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Playing(pos1));

        // [                        10♣,    
        // [6♥,     
        // [  K♠,     ,                  ]
        // [        J♠,           ]                
        // [               Q♦,          ]               
        game.set_play(id1, cards::Card::new(cards::Suit::Heart, cards::Rank::Rank6)).unwrap();
        game.set_play(id2, cards::Card::new(cards::Suit::Spade, cards::Rank::RankK)).unwrap();
        game.set_play(id3, cards::Card::new(cards::Suit::Spade, cards::Rank::RankJ)).unwrap();
        game.set_play(id4, cards::Card::new(cards::Suit::Diamond, cards::Rank::RankQ)).unwrap();
        game.set_play(id0, cards::Card::new(cards::Suit::Club, cards::Rank::Rank10)).unwrap();
        assert_eq!(game.get_turn(), Turn::Intertrick);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);

        assert_eq!(game.get_turn(), Turn::Interdeal);
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos1)));
        println!("scores: {:?}", game.scores);
    }
}
