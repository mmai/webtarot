use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::board::Board;
use crate::protocol::{
    GameInfo, GamePlayerState, GameStateSnapshot, DealSnapshot, Message, PlayerDisconnectedMessage, PlayerRole,
    Turn,
};
use crate::universe::Universe;
use tarotgame::{bid, cards, pos, game, trick};

pub struct GameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
    board: Board,
    deal: Deal,
    first: pos::PlayerPos,
    scores: [i32; 2],
}

impl GameState {
    fn position_taken(&self, position: &pos::PlayerPos) -> bool {
        self.players.iter().find(|(uuid, player)| &player.pos == position) != None
    }
}


/// Describe a single deal.
pub enum Deal {
    /// The deal is still in the auction phase
    Bidding(bid::Auction),
    /// The deal is in the main playing phase
    Playing(game::GameState),
}

impl Deal {
    fn next_player(&self) -> pos::PlayerPos {
        match self {
            &Deal::Bidding(ref auction) => auction.next_player(),
            &Deal::Playing(ref deal) => deal.next_player(),
        }
    }

    fn hands(&self) -> [cards::Hand; 4] {
        match self {
            &Deal::Bidding(ref auction) => auction.hands(),
            &Deal::Playing(ref deal) => deal.hands(),
        }
    }

    fn deal_state(&self) -> Option<&game::GameState> {
        match self {
            Deal::Bidding(bid) => None,
            Deal::Playing(state) => Some(state),
        }
    }
}

// Creates a new deal, starting with an auction.
// fn make_deal(first: pos::PlayerPos) -> bid::Auction {
fn make_deal(first: pos::PlayerPos) -> Deal {
    let auction = bid::Auction::new(first);
    Deal::Bidding(auction)
}

pub struct Game {
    id: Uuid,
    join_code: String,
    universe: Weak<Universe>,
    game_state: Arc<Mutex<GameState>>,
}

impl Game {
    pub fn new(join_code: String, universe: Arc<Universe>) -> Game {
        let deal = make_deal(pos::PlayerPos::P0);
        log::debug!("new deal: {:?}", deal.hands());
        Game {
            id: Uuid::new_v4(),
            join_code,
            universe: Arc::downgrade(&universe),
            game_state: Arc::new(Mutex::new(GameState {
                players: BTreeMap::new(),
                turn: Turn::Pregame,
                board: Board::new(),

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
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.ready = true;
            player_state.role = PlayerRole::Unknown;
        }

        let mut count = 0;

        for player in game_state.players.values() {
            if player.role != PlayerRole::Spectator {
                count = count + 1;
            }
        }

        if count == 4 {
            game_state.turn = game_state.board.initial_turn();
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
        for player_id in game_state.players.keys().copied() {
            log::debug!("broadcast game state to {}", player_id);
            let mut players = vec![];
            let mut reveal = false;
            for (&other_player_id, player_state) in game_state.players.iter() {
                players.push(player_state.clone());
            }
            log::debug!("with players {:?}", players);
            let pos = game_state.players[&player_id].pos;
            let deal = match game_state.deal.deal_state() {
                Some(state) => {
                    log::debug!("hands: {:?}", state.hands());
                    let points =  match state.get_game_result() {
                        game::GameResult::Nothing => [0; 2],
                        game::GameResult::GameOver {points, winners, scores } => points
                    };
                    DealSnapshot {
                        hand: state.hands()[pos as usize],
                        current: state.next_player(),
                        points
                    }
                },
                None => DealSnapshot {
                    hand: game_state.deal.hands()[pos as usize],
                    current: game_state.deal.next_player(),
                    points: [0;2]
                }
            };
            universe
                .send(
                    player_id,
                    &Message::GameStateSnapshot(GameStateSnapshot {
                        players,
                        tiles: game_state.board.tiles(reveal),
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
}
