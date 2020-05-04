use std::collections::BTreeMap;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::protocol::{
    GameInfo, GamePlayerState, GameStateSnapshot, DealSnapshot, Message, PlayerDisconnectedMessage, PlayerRole,
    Deal, Turn
};
use crate::universe::Universe;
use tarotgame::{bid, cards, pos, deal, trick};

pub struct GameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
    deal: Deal,
    first: pos::PlayerPos,
    scores: [i32; 2],
}

impl GameState {
    fn position_taken(&self, position: &pos::PlayerPos) -> bool {
        self.players.iter().find(|(_uuid, player)| &player.pos == position) != None
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
}

pub struct Game {
    id: Uuid,
    join_code: String,
    universe: Weak<Universe>,
    game_state: Arc<Mutex<GameState>>,
}

impl Game {
    pub fn new(join_code: String, universe: Arc<Universe>) -> Game {
        let deal = Deal::new(pos::PlayerPos::P0);
        log::debug!("new deal: {:?}", deal.hands());
        Game {
            id: Uuid::new_v4(),
            join_code,
            universe: Arc::downgrade(&universe),
            game_state: Arc::new(Mutex::new(GameState {
                players: BTreeMap::new(),
                turn: Turn::Pregame,

                deal,
                first: pos::PlayerPos::P0,
                scores: [0; 2],
            })),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn join_code(&self) -> &str {
        &self.join_code
    }

    pub fn game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.id,
            join_code: self.join_code.to_string(),
        }
    }

    pub async fn is_joinable(&self) -> bool {
        self.game_state.lock().await.turn == Turn::Pregame
    }

    pub fn universe(&self) -> Arc<Universe> {
        self.universe.upgrade().unwrap()
    }

    pub async fn add_player(&self, player_id: Uuid) {
        let universe = self.universe();
        if !universe
            .set_player_game_id(player_id, Some(self.id()))
            .await
        {
            return;
        }

        let mut game_state = self.game_state.lock().await;
        if game_state.players.contains_key(&player_id) {
            return;
        }

        // TODO: `set_player_game_id` also looks up.
        let player_info = match universe.get_player_info(player_id).await {
            Some(player_info) => player_info,
            None => return,
        };

        //Default pos
        let nb_players = game_state.players.len();
        let mut newpos = pos::PlayerPos::from_n(nb_players);

        //TODO rendre générique
        for p in &[ pos::PlayerPos::P0,
        pos::PlayerPos::P1,
        pos::PlayerPos::P2,
        pos::PlayerPos::P3,
        ] {
            if (!game_state.position_taken(p)){
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
        game_state.players.insert(state.player.id, state.clone());

        drop(game_state);
        self.broadcast(&Message::PlayerConnected(state)).await;
    }

    pub async fn remove_player(&self, player_id: Uuid) {
        self.universe().set_player_game_id(player_id, None).await;

        let mut game_state = self.game_state.lock().await;
        if game_state.players.remove(&player_id).is_some() {
            drop(game_state);
            self.broadcast(&Message::PlayerDisconnected(PlayerDisconnectedMessage {
                player_id,
            }))
            .await;
        }

        if self.is_empty().await {
            self.universe().remove_game(self.id()).await;
        }
    }

    pub async fn set_player_role(&self, player_id: Uuid, role: PlayerRole) {
        let mut game_state = self.game_state.lock().await;
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.role = role;
            player_state.ready = false;
        }
    }

    pub async fn mark_player_ready(&self, player_id: Uuid) {
        let mut game_state = self.game_state.lock().await;
        let turn = game_state.turn.clone();
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.ready = true;
            if turn == Turn::Intertrick {
                game_state.update_turn();
            } else {
                player_state.role = PlayerRole::PreDeal;

                // Check if we start the next deal
                let mut count = 0;
                for player in game_state.players.values() {
                    if player.role == PlayerRole::PreDeal {
                        count = count + 1;
                    }
                }
                if count == 4 {
                    if game_state.turn == Turn::Interdeal { // ongoing game
                        Self::next_deal(&mut game_state);
                        game_state.update_turn();
                    } else { // new game
                        game_state.turn = Turn::Bidding((bid::AuctionState::Bidding, pos::PlayerPos::P0));
                    }
                }

            }
        }

    }

    pub async fn broadcast(&self, message: &Message) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.players.keys().copied() {
            universe.send(player_id, message).await;
        }
    }

    pub async fn broadcast_state(&self) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        let contract = game_state.deal.deal_auction().and_then(|auction| auction.current_contract());
        log::debug!("contract broadcasted {:?}", contract);
        log::debug!("turn broadcasted {:?}", game_state.turn);
        for player_id in game_state.players.keys().copied() {
            log::debug!("broadcast game state to {}", player_id);
            let mut players = vec![];
            for (&other_player_id, player_state) in game_state.players.iter() {
                players.push(player_state.clone());
            }
            players.sort_by(|a, b| a.pos.to_n().cmp(&b.pos.to_n()));
            // log::debug!("with sorted players {:?}", players);
            let pos = game_state.players[&player_id].pos;
            let contract = contract.cloned();
            let deal = match game_state.deal.deal_state() {
                Some(state) => { // In Playing phase
                    let points =  match state.get_deal_result() {
                        deal::DealResult::Nothing => [0; 2],
                        deal::DealResult::GameOver {points, winners, scores } => points
                    };
                    let last_trick = if game_state.turn == Turn::Intertrick {
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
                    hand: game_state.deal.hands()[pos as usize],
                    current: game_state.deal.next_player(),
                    contract,
                    points: [0;2],
                    last_trick: trick::Trick::default(),
                }
            };
            universe
                .send(
                    player_id,
                    &Message::GameStateSnapshot(GameStateSnapshot {
                        players,
                        turn: game_state.turn,
                        deal
                    }),
                )
                .await;
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.game_state.lock().await.players.is_empty()
    }


    pub async fn set_bid(&self, pid: Uuid, target: bid::Target, trump: cards::Suit){
        let mut game_state = self.game_state.lock().await;
        let pos = game_state.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = game_state.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.bid(pos, trump, target) {
            Self::complete_auction(&mut game_state);
        }
        game_state.update_turn();
    }

    pub async fn set_pass(&self, pid: Uuid){
        let mut game_state = self.game_state.lock().await;
        let pos = game_state.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = game_state.deal.deal_auction_mut().unwrap();
        let pass_result = auction.pass(pos);
        if Ok(bid::AuctionState::Over) == pass_result  {
            Self::complete_auction(&mut game_state);
        }
        game_state.update_turn();
    }

    pub async fn set_coinche(&self, pid: Uuid){
        let mut game_state = self.game_state.lock().await;
        let pos = game_state.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let auction = game_state.deal.deal_auction_mut().unwrap();
        if Ok(bid::AuctionState::Over) == auction.coinche(pos) {
            Self::complete_auction(&mut game_state);
        }
        game_state.update_turn();
    }

    pub async fn set_play(&self, pid: Uuid, card: cards::Card){
        let mut game_state = self.game_state.lock().await;
        let pos = game_state.players.get(&pid).map(|p| p.pos).unwrap();// TODO -> Result<..>
        let state = game_state.deal.deal_state_mut().unwrap();
        match state.play_card(pos, card) { // TODO -> RESULT
            Err(e) => {
                log::debug!("erreur play card: {:?}", e);
                ()
            },
            Ok(deal::TrickResult::Nothing) => (),
            Ok(deal::TrickResult::TrickOver(winner, game_result)) => {
                match game_result {
                    deal::DealResult::Nothing => Self::end_trick(&mut game_state),
                    deal::DealResult::GameOver{points, winners, scores} => {

                        log::debug!("results: {:?} {:?}", points, winners);
                        for i in 0..2 {
                            game_state.scores[i] += scores[i];
                        }
                        Self::end_deal(&mut game_state);
                    }
                }
            }
        }
        game_state.update_turn();
    }

    fn complete_auction(game_state: &mut GameState) {
        log::info!("auction complete");
        let deal_state = match &mut game_state.deal {
            &mut Deal::Playing(_) => unreachable!(),
            &mut Deal::Bidding(ref mut auction) => {
                match auction.complete() {
                    Ok(deal_state) => deal_state,
                    Err(err) => panic!(err)
                }
            }
        };
        game_state.deal = Deal::Playing(deal_state);
    }

    fn end_trick(game_state: &mut GameState) {
        for player in game_state.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
            }
        }
    }

    fn end_deal(game_state: &mut GameState) {
        game_state.turn = Turn::Interdeal;
        for player in game_state.players.values_mut() {
            if player.role != PlayerRole::Spectator {
                player.ready = false;
                player.role = PlayerRole::Unknown;
            }
        }
    }

    fn next_deal(game_state: &mut GameState) {
        // TODO: Maybe keep the current game in the history?
        let auction = bid::Auction::new(game_state.first);
        game_state.first = game_state.first.next();
        game_state.deal = Deal::Bidding(auction);
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::universe::Universe;
    // use tarotgame::{bid, cards, pos, deal, trick};

    #[test]
    fn test_init_game() {
        assert_eq!("a", "a");
    }
}
