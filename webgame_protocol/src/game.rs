use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::turn::Turn;
use crate::deal::{Deal, DealSnapshot};
use crate::player::{PlayerInfo, PlayerRole, GamePlayerState};
use tarotgame::{bid, cards, pos, deal, trick};

pub struct GameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
    deal: Deal,
    first: pos::PlayerPos,
    scores: [i32; 2],
}

impl GameState {
    pub fn default() -> Self {
        GameState {
            players: BTreeMap::new(),
            turn: Turn::Pregame,
            deal: Deal::new(pos::PlayerPos::P0),
            first: pos::PlayerPos::P0,
            scores: [0; 2],
        }
    }

    pub fn get_turn(&self) -> Turn {
        self.turn
    }

    pub fn is_joinable(&self) -> bool {
        self.turn == Turn::Pregame
    }
    
    pub fn get_players(&self) -> &BTreeMap<Uuid, GamePlayerState> {
        &self.players
    }

    pub fn add_player(&mut self, player_info: PlayerInfo) -> pos::PlayerPos {
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

    pub fn remove_player(&mut self, player_id: Uuid) -> bool {
        self.players.remove(&player_id).is_some()
    }

    pub fn set_player_role(&mut self, player_id: Uuid, role: PlayerRole) {
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.role = role;
            player_state.ready = false;
        }
    }

    fn position_taken(&self, position: pos::PlayerPos) -> bool {
        self.player_by_pos(position) != None
    }

    pub fn player_by_pos(&self, position: pos::PlayerPos) -> Option<&GamePlayerState> {
        self.players.iter().find(|(_uuid, player)| player.pos == position).map(|p| p.1)
    }

    // Creates a view of the game for a player
    pub fn make_snapshot(&self, player_id: Uuid) -> GameStateSnapshot {
        let contract = self.deal.deal_auction().and_then(|auction| auction.current_contract()).cloned();
        let mut players = vec![];
        for (&_other_player_id, player_state) in self.players.iter() {
            players.push(player_state.clone());
        }
        players.sort_by(|a, b| a.pos.to_n().cmp(&b.pos.to_n()));
        let pos = self.players[&player_id].pos;
        let deal = match self.deal.deal_state() {
            Some(state) => { // In Playing phase
                let points =  match state.get_deal_result() {
                    deal::DealResult::Nothing => [0; 2],
                    deal::DealResult::GameOver {points, winners: _, scores: _ } => points
                };
                let last_trick = if self.turn == Turn::Intertrick {
                    // intertrick : there is at least a trick done
                    // (current_trick() returns the new empty one)
                    state.last_trick().unwrap().clone()
                } else {
                    state.current_trick().clone()
                };
                // log::debug!("trick {:?}", last_trick.cards);
                DealSnapshot {
                    hand: state.hands()[pos as usize],
                    current: state.next_player(),
                    contract,
                    points,
                    // last_trick: state.tricks.last().unwrap_or(trick::Trick::default()),
                    last_trick,
                }
            },
            None => DealSnapshot { // In bidding phase
                hand: self.deal.hands()[pos as usize],
                current: self.deal.next_player(),
                contract,
                points: [0;2],
                last_trick: trick::Trick::default(),
            }
        };
        GameStateSnapshot {
            players,
            turn: self.turn,
            deal
        }
    }

    fn players_ready(&self) -> bool {
        !(self.players.iter().find(|(_, player)| player.ready == false) != None)
    }

    pub fn update_turn(&mut self){
        // if !(self.turn == Turn::Interdeal) {
            self.turn = if self.players_ready() {
                Turn::from_deal(&self.deal)
            } else {
                Turn::Intertrick
            }
        // }
    }

    pub fn set_player_ready(&mut self, player_id: Uuid){
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
                if count == 4 {
                    if self.turn == Turn::Interdeal { // ongoing game
                        self.next_deal();
                        self.update_turn();
                    } else { // new game
                        self.turn = Turn::Bidding((bid::AuctionState::Bidding, pos::PlayerPos::P0));
                    }
                }

            }
        }
    }

    pub fn set_bid(&mut self, pid: Uuid, target: bid::Target, trump: cards::Suit){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = self.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.bid(pos, trump, target) {
            self.complete_auction();
        }
        self.update_turn();
    }

    pub fn set_pass(&mut self, pid: Uuid){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = self.deal.deal_auction_mut().unwrap();
        let pass_result = auction.pass(pos);
        if Ok(bid::AuctionState::Over) == pass_result  {
            self.complete_auction();
        }
        self.update_turn();
    }

    pub fn set_coinche(&mut self, pid: Uuid){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = self.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.coinche(pos) {
            self.complete_auction();
        }
        self.update_turn();
    }

    pub fn set_play(&mut self, pid: Uuid, card: cards::Card){
        let pos = self.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let state = self.deal.deal_state_mut().unwrap();
        match state.play_card(pos, card) { // TODO -> RESULT
            Err(_e) => {
                // log::debug!("erreur play card: {:?}", e);
                ()
            },
            Ok(deal::TrickResult::Nothing) => (),
            Ok(deal::TrickResult::TrickOver(_winner, game_result)) => {
                match game_result {
                    deal::DealResult::Nothing => self.end_trick(),
                    deal::DealResult::GameOver{points: _, winners: _, scores} => {
                        // log::debug!("results: {:?} {:?}", points, winners);
                        for i in 0..2 {
                            self.scores[i] += scores[i];
                        }
                        self.end_deal();
                    }
                }
            }
        }
        self.update_turn();
    }

    fn complete_auction(&mut self) {
        let deal_state = match &mut self.deal {
            &mut Deal::Playing(_) => unreachable!(),
            &mut Deal::Bidding(ref mut auction) => {
                match auction.complete() {
                    Ok(deal_state) => deal_state,
                    Err(err) => panic!(err)
                }
            }
        };
        self.deal = Deal::Playing(deal_state);
    }

    fn end_trick(&mut self) {
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
            }
        }
    }

    fn end_deal(&mut self) {
        self.turn = Turn::Interdeal;
        for player in self.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
                player.role = PlayerRole::Unknown;
            }
        }
    }

    fn next_deal(&mut self) {
        let auction = bid::Auction::new(self.first);
        self.first = self.first.next();
        self.deal = Deal::Bidding(auction);
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameStateSnapshot {
    pub players: Vec<GamePlayerState>,
    pub turn: Turn,
    pub deal: DealSnapshot,
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
}

impl Default for GameStateSnapshot {
    fn default() -> GameStateSnapshot {
        let pos = pos::PlayerPos::P0; // could be anything
        GameStateSnapshot {
            players: vec![],
            turn: Turn::Pregame,
            deal: DealSnapshot {
                hand: cards::Hand::new(),
                current: pos,
                contract: None,
                points: [0;2],
                last_trick: trick::Trick::new(pos),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub join_code: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tarotgame::{bid, cards};
    // use tarotgame::{bid, cards, pos, deal, trick};

    #[test]
    fn test_init_game() {
        let mut game = GameState::default();
        let pos0 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player0")});
        let pos1 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player1")});
        let pos2 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player2")});
        let pos3 = game.add_player(PlayerInfo {id: Uuid::new_v4(), nickname: String::from("player3")});

        assert_eq!(true, game.is_joinable());

        let id0 = game.player_by_pos(pos0).unwrap().player.id;
        let id1 = game.player_by_pos(pos1).unwrap().player.id;
        let id2 = game.player_by_pos(pos2).unwrap().player.id;
        let id3 = game.player_by_pos(pos3).unwrap().player.id;
        game.set_player_ready(id0);
        game.set_player_ready(id1);
        game.set_player_ready(id2);
        game.set_player_ready(id3);
        assert_eq!(false, game.is_joinable());
        assert_eq!(game.get_turn(), Turn::Bidding((bid::AuctionState::Bidding, pos0)));

        game.set_bid(id0, bid::Target::Contract80, cards::Suit::Heart);
        game.set_pass(id1);
        game.set_pass(id2);
        game.set_pass(id3);
        assert_eq!(game.get_turn(), Turn::Playing(pos0));


    }
}
