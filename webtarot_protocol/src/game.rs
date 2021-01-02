use std::collections::BTreeMap;
use std::rc::Weak;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tarotgame::{bid, cards, pos, deal, trick, Announce};
use webgame_protocol::{GameState, PlayerInfo, ProtocolErrorKind, GameManager};
pub use webgame_protocol::GameEventsListener;
use crate::{ ProtocolError };

use crate::turn::Turn;
use crate::deal::{Deal, DealSnapshot};
use crate::player::{PlayerRole, GamePlayerState};
use crate::message::{TarotVariant, DebugOperation};

#[derive(Clone)]
pub struct TarotGameState {
    nb_players: u8,
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
    deal: Deal,
    first: pos::PlayerPos,
    scores: Vec<Vec<f32>>,
}
//
// pub struct TarotGameManager {
//     state: TarotGameState,
// }
//
// impl GameEventsListener<PlayEvent> for TarotGameManager {
//     fn notify(&mut self, event: &PlayEvent) {
//         println!("Notify called with {:?}", event);
//     }
// }

pub struct TarotGameManager<'a, Listener: GameEventsListener<(PlayEvent, TarotGameState)>> {
    state: &'a mut TarotGameState,
    listeners: Vec<&'a mut Listener>,
}

impl<'a, Listener: GameEventsListener<(PlayEvent, TarotGameState)> + PartialEq> TarotGameManager<'a, Listener>  {
    pub fn new(state: &'a mut TarotGameState) -> TarotGameManager<'a, Listener> {
        TarotGameManager {
            state,
            listeners: Vec::new(),
        }
    }

    pub fn set_play(&mut self, pid: Uuid, card: cards::Card) -> Result<(), ProtocolError> {
        let pos = self.state.players.get(&pid).map(|p| p.pos).unwrap();
        let state = self.state.deal.deal_state_mut().ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "Unknown deal state")
        )?;
        match state.play_card(pos, card)? {
            deal::TrickResult::Nothing => {},
            deal::TrickResult::TrickOver(_winner, deal::DealResult::Nothing) => {
                // XXX ugly hack to get the correct played cards as the new trick has already
                // been initiated in the play_card() function 
                let mut state_snapshot = self.state.clone();
                state_snapshot
                    .get_deal_mut()
                    .deal_state_mut().unwrap()
                    .revert_trick();
                self.emit((PlayEvent::EndTrick, state_snapshot));
            },
            deal::TrickResult::TrickOver(_winner, deal::DealResult::GameOver{points: _, taker_diff: _, scores}) => {
                self.state.scores.push(scores);
                let state_snapshot = self.state.clone();
                self.state.end_last_trick();
                self.emit((PlayEvent::EndDeal, state_snapshot));
                self.state.next_deal();
            }
        };
        self.state.update_turn();
        Ok(())
    }
}

impl<'a, Listener: GameEventsListener<(PlayEvent, TarotGameState)> + PartialEq> GameManager<'a, Listener> for TarotGameManager<'a, Listener>  
{
    type Event = (PlayEvent, TarotGameState);

    fn register_listener(&mut self, listener: &'a mut Listener){
        self.listeners.push(listener);
    }

    fn unregister_listener(&mut self, listener: &'a Listener){
        if let Some(idx) = self.listeners.iter().position(|x| *x == listener){
            self.listeners.remove(idx);
        }
    }

    fn emit(&mut self, event: Self::Event) {
        self.listeners.iter_mut().for_each(|listener| { 
            listener.notify(&event)
        });
    }
}

impl Default for TarotGameState {
    fn default() -> TarotGameState {
        TarotGameState {
            nb_players: 5,
            players: BTreeMap::new(),
            turn: Turn::Pregame,
            deal: Deal::new(pos::PlayerPos::from_n(0, 5)),
            first: pos::PlayerPos::from_n(0, 5),
            scores: vec![],
        }
    }
}

impl GameState for TarotGameState {
    type PlayerPos = pos::PlayerPos;
    type PlayerRole = PlayerRole;

    type GamePlayerState = GamePlayerState;
    type Snapshot = GameStateSnapshot;
    type Operation = DebugOperation;
    type VariantParameters = VariantSettings;

    fn set_variant(&mut self, variant: TarotVariant) {
        self.nb_players = variant.parameters.nb_players;
        self.deal = Deal::new(pos::PlayerPos::from_n(0, self.nb_players));
        self.first = pos::PlayerPos::from_n(0, self.nb_players);
    }
    
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
        let mut newpos = pos::PlayerPos::from_n(nb_players, self.nb_players);

        for p in 0..self.nb_players {
            let position = pos::PlayerPos::from_n(p as usize, self.nb_players);
            if !self.position_taken(position){
                newpos = position.clone();
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
        let mut scores = vec![0.0; self.nb_players as usize];
        let mut dog = cards::Hand::new();
        let mut taker_diff = 0.0;
        let mut announces = vec![vec![]; self.nb_players as usize];
        let mut trick_count = 0;
        let deal = match self.deal.deal_state() {
            Some(state) => { // In Playing phase
                announces = state.announces.clone();
                trick_count = state.get_tricks_count();
                if let deal::DealResult::GameOver {points: _, taker_diff: diff, scores: lscores } = state.get_deal_result() {
                     scores = lscores;
                     taker_diff = diff;
                     dog = state.dog();
                };
                let last_trick = state.current_trick().clone();
                let initial_dog = if self.turn == Turn::MakingDog {
                    state.dog()
                } else { cards::Hand::new() };

                //When the dog is done
                // if self.turn == Turn::Intertrick || matches!(self.turn, Turn::Playing(_x)) {
                if matches!(self.turn, Turn::Playing(_x)) {
                    //We check if there are cards to show
                    let to_show: Vec<cards::Card> = state.dog().list()
                        .into_iter()
                        .filter(|c| c.suit() == cards::Suit::Trump || c.rank() == cards::Rank::RankK)
                        .collect();
                    for c in to_show {
                        dog.add(c);
                    }
                }

                DealSnapshot {
                    hand: state.hands()[pos.pos as usize],
                    current: state.next_player(),
                    contract,
                    king: state.king(),
                    scores,
                    // last_trick: state.tricks.last().unwrap_or(trick::Trick::default()),
                    last_trick,
                    trick_count,
                    initial_dog,
                    dog,
                    taker_diff,
                    announces,
                }
            },
            None => DealSnapshot { // In bidding phase
                hand: self.deal.hands()[pos.pos as usize],
                current: self.deal.next_player(),
                contract,
                king: None,
                scores: vec![0.0;self.nb_players as usize],
                last_trick: trick::Trick::default(),
                trick_count,
                initial_dog: cards::Hand::new(),
                dog,
                taker_diff,
                announces,
            }
        };
        GameStateSnapshot {
            players,
            scores: self.scores.clone(),
            turn: self.turn,
            deal
        }
    }

    fn set_player_ready(&mut self, player_id: Uuid) -> bool {
        let turn = self.turn.clone();
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.ready = true;
            // println!("set_player_ready, turn = {}", turn.to_string());
                player_state.role = PlayerRole::PreDeal;

                // Check if we start the next deal
                let mut count = 0;
                for player in self.players.values() {
                    if player.role == PlayerRole::PreDeal {
                        count = count + 1;
                    }
                }
                // println!("set_player_ready, count = {} ; nb_players = {}", count, self.nb_players);
                if count == self.nb_players {
                    self.turn = Turn::Bidding((bid::AuctionState::Bidding, pos::PlayerPos::from_n(0, count)));
                    return true
                }
            // }
        }
        false
    }

    fn set_player_not_ready(&mut self, player_id: Uuid) {
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.ready = false;
        }
    }

    fn update_init_state(&mut self) -> bool {
        // self.next();
        false
    }

    fn manage_operation(&mut self, operation: Self::Operation) {
        match operation {
            Self::Operation::SetSeed(seed) => {
                let (hands, dog) = tarotgame::deal_seeded_hands(seed, self.nb_players as usize);
                self.deal.deal_auction_mut().map(|auction| auction.set_hands(hands, dog));
            }
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
        self.turn = Turn::from_deal(&self.deal)
    }

    pub fn set_bid(&mut self, pid: Uuid, target: bid::Target, slam: bool) -> Result<(), ProtocolError>{
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = self.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.bid(pos, target, slam) {
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

    pub fn make_dog(&mut self, pid: Uuid, cards: cards::Hand, slam: bool) -> Result<(), ProtocolError> {
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();
        self.deal.deal_state_mut().unwrap().make_dog(pos, cards, slam)?;
        self.turn = Turn::from_deal(&self.deal);
        Ok(())
    }

    pub fn set_announce(&mut self, pid: Uuid, announce: Announce) -> Result<(), ProtocolError>{
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();
        let state = self.deal.deal_state_mut().ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "Unknown deal state")
        )?;
        state.announce(pos, announce)?;
        Ok(())
    }

    // XXX obsolete ? (cf. manager set_play)
    pub fn set_play(&mut self, pid: Uuid, card: cards::Card) -> Result<Option<PlayEvent>, ProtocolError> {
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();
        let state = self.deal.deal_state_mut().ok_or(
            ProtocolError::new(ProtocolErrorKind::InternalError, "Unknown deal state")
        )?;
        let result = match state.play_card(pos, card)? {
            deal::TrickResult::Nothing => None,
            deal::TrickResult::TrickOver(_winner, deal::DealResult::Nothing) => Some(PlayEvent::EndTrick),
            deal::TrickResult::TrickOver(_winner, deal::DealResult::GameOver{points: _, taker_diff: _, scores}) => {
                self.scores.push(scores);
                self.end_last_trick();
                // self.next_deal();
                Some(PlayEvent::EndDeal)
            }
        };
        // self.update_turn();
        Ok(result)
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
        self.turn = if self.nb_players == 5 {
            Turn::CallingKing
        } else {
            let target = self.deal.deal_contract().unwrap().target;
            if target == bid::Target::GardeSans || target == bid::Target::GardeContre {
                //No dog
                Turn::from_deal(&self.deal)
            } else {
                //Dog
                Turn::MakingDog
            }
        };
        Ok(())
    }

    fn end_last_trick(&mut self) {
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.role = PlayerRole::Unknown;
            }
        }
    }

    pub fn next_deal(&mut self) {
        self.first = self.first.next();
        let auction = bid::Auction::new(self.first);
        self.deal = Deal::Bidding(auction);
    }

    pub fn get_deal(&self) -> &Deal {
        &self.deal
    }
    pub fn get_deal_mut(&mut self) -> &mut Deal {
        &mut self.deal
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayEvent {
    Play( Uuid, cards::Card),
    Announce( Uuid, Announce ),
    EndTrick,
    EndDeal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VariantSettings {
    pub nb_players: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameStateSnapshot {
    pub players: Vec<GamePlayerState>,
    pub turn: Turn,
    pub deal: DealSnapshot,
    pub scores: Vec<Vec<f32>>,
}

impl webgame_protocol::GameStateSnapshot for GameStateSnapshot { }

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
        let pos = pos::PlayerPos::from_n(0, 5); // could be anything
        GameStateSnapshot {
            players: vec![],
            scores: vec![],
            turn: Turn::Pregame,
            deal: DealSnapshot {
                hand: cards::Hand::new(),
                current: pos,
                contract: None,
                king: None,
                scores: vec![],
                last_trick: trick::Trick::new(pos),
                trick_count: 0,
                initial_dog: cards::Hand::new(),
                dog: cards::Hand::new(),
                taker_diff: 0.0,
                announces: vec![],
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
        // assert_ne!(game.deal.hands(), hands_deal1);
    }

    #[test]
    fn test_garde_contre() {
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

        let _hands_deal1 = game.deal.hands();

        game.set_bid(id0, bid::Target::GardeContre, false).unwrap();
        assert_eq!(game.get_turn(), Turn::CallingKing);
        assert_eq!(game.player_by_pos(pos0).unwrap().role, PlayerRole::Taker);
        game.call_king(id0, cards::Card::new(cards::Suit::Club, cards::Rank::RankK));

        // Garde contre : no dog creation step
        assert_eq!(game.get_turn(), Turn::Playing(pos0));
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
        let (hands, dog) = deal_seeded_hands(seed, 5);
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

        game.set_bid(id0, bid::Target::Garde, false).unwrap();
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
        game.make_dog(id0, dog, false);
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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);

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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
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
        // assert_eq!(game.get_turn(), Turn::Intertrick);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
        //
        // assert_eq!(game.get_turn(), Turn::Interdeal);
        // game.set_player_ready(id0);
        // game.set_player_ready(id1);
        // game.set_player_ready(id2);
        // game.set_player_ready(id3);
        // game.set_player_ready(id4);
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos1)));
        // println!("scores: {:?}", game.scores);
    }

    #[test]
    fn test_generic_game() {
        // let variant: usize = 4;
        let variant: usize = 5;
        let mut game = TarotGameState {
                nb_players: variant as u8,
                players: BTreeMap::new(),
                turn: Turn::Pregame,
                deal: Deal::new(pos::PlayerPos::from_n(0, variant as u8)),
                first: pos::PlayerPos::from_n(0, variant as u8),
                scores: vec![],
            };

        for v in 0..variant {
            let pos = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from(format!("player{:?}", v))});
            game.set_player_ready(game.player_by_pos(pos).unwrap().player.id);
        }

        assert_eq!(false, game.is_joinable());

        let seed = [7, 32, 3, 32, 54, 1, 84, 3, 32, 54, 1, 84, 3, 32, 65, 1, 84, 3, 32, 64, 1, 44, 3, 32, 54, 1, 84, 3, 32, 65, 1, 44];
        let (hands, dog) = deal_seeded_hands(seed, variant);

        println!("{}", dog.to_string());
        // for hand in hands.iter() {
        //     println!("{}", hand.to_string()); // `cargo test -- --nocapture` to view output
        // }
        // println!("-------------------");

        let auction = game.deal.deal_auction_mut().unwrap();
        auction.set_hands(hands, dog);

        let pos0 = pos::PlayerPos::from_n(0, variant as u8);
        let pos1 = pos::PlayerPos::from_n(1, variant as u8);
        let id0 = game.player_by_pos(pos0).unwrap().player.id;
        game.set_bid(id0, bid::Target::Garde, false).unwrap();
        for v in 1..variant {
            let pos = pos::PlayerPos::from_n(v, variant as u8);
            game.set_pass(game.player_by_pos(pos).unwrap().player.id).unwrap();
        }
        if variant == 5 {
            assert_eq!(game.get_turn(), Turn::CallingKing);
            game.call_king(id0, cards::Card::new(cards::Suit::Club, cards::Rank::RankK));
        }
        assert_eq!(game.get_turn(), Turn::MakingDog);
        game.make_dog(id0, dog, false);// keep initial dog
        assert_eq!(true, game.players_ready());
        assert_eq!(game.get_turn(), Turn::Playing(pos0));

        let deal_size = game.deal.hands()[0].size();
        for _id in 0..deal_size {
            // println!("-------------------------");
            // for hand in game.deal.hands().iter() {
            //     println!("{}", hand.to_string()); // `cargo test -- --nocapture` to view output
            // }

            let mut pos = game.deal.next_player();
            for _v in 0..variant {
                let player_id = game.player_by_pos(pos).unwrap().player.id;
                let hand = game.deal.hands()[pos.to_n()];
                // let mut found = false;
                for card in hand.list() { // Try to play a card until one is correct
                    if !game.set_play(player_id, card).is_err() {
                        // println!("player {:?} played {}", pos, card.to_string());
                        // found = true;
                        break;
                    }
                }
                // if !found {
                //     println!("no playable cards ?? {}", hand.to_string());
                // }
                pos = pos.next();
            }
            // for v in 0..variant {
            //     let pos = pos::PlayerPos::from_n(v, variant as u8);
            //     game.set_player_ready(game.player_by_pos(pos).unwrap().player.id);
            // }
        }
        // assert_eq!(game.get_turn(), Turn::Interdeal);

        // for v in 0..variant {
        //     let pos = pos::PlayerPos::from_n(v, variant as u8);
        //     game.set_player_ready(game.player_by_pos(pos).unwrap().player.id);
        // }
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos1)));

    }

    #[test]
    fn test_garde_contre_4players() {
        let variant: usize = 4;
        let mut game = TarotGameState {
                nb_players: variant as u8,
                players: BTreeMap::new(),
                turn: Turn::Pregame,
                deal: Deal::new(pos::PlayerPos::from_n(0, variant as u8)),
                first: pos::PlayerPos::from_n(0, variant as u8),
                scores: vec![],
            };

        for v in 0..variant {
            let pos = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from(format!("player{:?}", v))});
            game.set_player_ready(game.player_by_pos(pos).unwrap().player.id);
        }

        let seed = [7, 32, 3, 32, 54, 1, 84, 3, 32, 54, 1, 84, 3, 32, 65, 1, 84, 3, 32, 64, 1, 44, 3, 32, 54, 1, 84, 3, 32, 65, 1, 44];
        let (hands, dog) = deal_seeded_hands(seed, variant);

        let auction = game.deal.deal_auction_mut().unwrap();
        auction.set_hands(hands, dog);

        let pos0 = pos::PlayerPos::from_n(0, variant as u8);
        let id0 = game.player_by_pos(pos0).unwrap().player.id;
        game.set_bid(id0, bid::Target::GardeContre, false).unwrap();

        assert_eq!(game.get_turn(), Turn::Playing(pos0));



        let deal_size = game.deal.hands()[0].size();
        for _id in 0..deal_size {
            // println!("-------------------------");
            // for hand in game.deal.hands().iter() {
            //     println!("{}", hand.to_string()); // `cargo test -- --nocapture` to view output
            // }

            let mut pos = game.deal.next_player();
            for _v in 0..variant {
                let player_id = game.player_by_pos(pos).unwrap().player.id;
                let hand = game.deal.hands()[pos.to_n()];
                // let mut found = false;
                for card in hand.list() { // Try to play a card until one is correct
                    if !game.set_play(player_id, card).is_err() {
                        // println!("player {:?} played {}", pos, card.to_string());
                        // found = true;
                        break;
                    }
                }
                // if !found {
                //     println!("no playable cards ?? {}", hand.to_string());
                // }
                pos = pos.next();
            }
            // for v in 0..variant {
            //     let pos = pos::PlayerPos::from_n(v, variant as u8);
            //     game.set_player_ready(game.player_by_pos(pos).unwrap().player.id);
            // }
        }
        // assert_eq!(game.get_turn(), Turn::Interdeal);
        // println!("scores: {:?}", game.scores);
    }

}
