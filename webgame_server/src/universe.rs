use std::collections::HashMap;
use std::sync::Arc;
use std::convert::From;

use serde::Serialize;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use warp::ws;

use crate::game::Game;
use crate::protocol::{Message, PlayerInfo, PlayerState, ProtocolError, ProtocolErrorKind, GameExtendedInfo, GameState, GameStateSnapshot};
use crate::utils::generate_join_code;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub nickname: String,
}

// impl User {
//     pub fn as_player_info(self) -> PlayerInfo {
//         PlayerInfo {
//             id: self.id,
//             nickname: self.nickname
//         }
//     }
// }

impl From<PlayerInfo> for User {
    fn from(player: PlayerInfo) -> Self {
        User { 
            id: player.id,
            nickname: player.nickname
        }
    }
}

impl From<User> for PlayerInfo {
    fn from(user: User) -> Self {
        PlayerInfo {
            id: user.id,
            nickname: user.nickname
        }
    }
}

pub struct UniverseUserState {
    user: User,
    is_authenticated: bool,
    game_id: Option<Uuid>,
    tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
}

pub struct UniverseState<GameStateType: GameState<GamePlayerStateT, GameStateSnapshotT>, GamePlayerStateT: PlayerState, GameStateSnapshotT: GameStateSnapshot, PlayEventT> {
    users: HashMap<Uuid, UniverseUserState>,
    games: HashMap<Uuid, Arc<Game<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>>>,
    joinable_games: HashMap<String, Uuid>,
}

pub struct Universe<
    GameStateType: GameState<GamePlayerStateT, GameStateSnapshotT>,
    GamePlayerStateT: PlayerState,
    GameStateSnapshotT: GameStateSnapshot,
    PlayEventT> {
        state: Arc<RwLock<UniverseState<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>>>,
}

impl<GameStateType: Default+GameState<GamePlayerStateT, GameStateSnapshotT>, GamePlayerStateT: PlayerState, GameStateSnapshotT:GameStateSnapshot, PlayEventT:Serialize+Send> Universe<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT> {
    pub fn new() -> Universe<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT> {
        Universe {
            state: Arc::new(RwLock::new(UniverseState {
                users: HashMap::new(),
                games: HashMap::new(),
                joinable_games: HashMap::new(),
            })),
        }
    }

    /// show all the active games
    pub async fn show_games(self: &Arc<Self>) -> Vec<GameExtendedInfo> {
        let state = self.state.read().await;
        let fgames = state.games.iter()
            .map(|(_uuid, g)| {
                g.game_extended_info()
            } );
        futures::future::join_all(fgames).await
    }

    /// for debug purposes: show all the users connected to the server, except user_id
    pub async fn show_users(self: &Arc<Self>, user_id: Uuid) -> Vec<Uuid> {
        let state = self.state.read().await;
        let uuids:Vec<Uuid> = state.users.keys()
            .filter(|k| *k != &user_id)
            .map(|k| *k )
            .collect();
        uuids
    }

    /// Starts a new game.
    pub async fn new_game(self: &Arc<Self>) -> Arc<Game<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>> {
        let mut universe_state = self.state.write().await;

        loop {
            let join_code = generate_join_code();
            if universe_state.joinable_games.contains_key(&join_code) {
                continue;
            }

            let game = Arc::new(Game::new(join_code, self.clone()));
            universe_state.games.insert(game.id(), game.clone());
            universe_state
                .joinable_games
                .insert(game.join_code().to_string(), game.id());
            return game;
        }
    }

    /// Joins a user into a game by join code.
    pub async fn join_game(
        &self,
        user_id: Uuid,
        join_code: String,
    ) -> Result<Arc<Game<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>>, ProtocolError> {
        // assign to temporary to release lock.
        let game_id = self
            .state
            .read()
            .await
            .joinable_games
            .get(&join_code)
            .copied();

        if let Some(game_id) = game_id {
            if let Some(game) = self.get_game(game_id).await {
                if game.is_joinable().await {
                    game.add_player(user_id).await;
                    return Ok(game);
                } else {
                    return Err(ProtocolError::new(
                        ProtocolErrorKind::InvalidCommand,
                        "game is currently not joinable",
                    ));
                }
            }
        }

        Err(ProtocolError::new(
            ProtocolErrorKind::NotFound,
            "game does not exist",
        ))
    }

    /// Registers a user.
    ///
    /// The user is given a new ID which is returned and starts out without
    /// any associated nickname.
    pub async fn add_user(
        &self,
        tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
        guid: String,
        uuid: String,
    ) -> (User, Option<Uuid>) {
        //Defaults for a new user
        let mut user_id = Uuid::new_v4();
        let mut nickname: String = "anonymous".into();
        let mut game_id: Option<Uuid> = None;
        let mut is_authenticated = false;

        // Check validity of given uuid
        if let (Ok(user_uuid),  Ok(game_uid)) = (Uuid::parse_str(&uuid), Uuid::parse_str(&guid)) {
            //Check if user is in a active game
            if let Some(user) = self.find_user_game(game_uid, user_uuid).await {
                user_id = user_uuid;
                game_id = Some(game_uid);
                is_authenticated = true; 
                nickname = user.nickname; 
            }
        }

        //Register user
        let user = User {
            id: user_id,
            nickname,
        };
        let mut universe_state = self.state.write().await;
        universe_state.users.insert(
            user_id,
            UniverseUserState {
                user: user.clone(),
                game_id,
                is_authenticated,
                tx,
            },
        );
        (user, game_id)
    }

    /// Returns the user.
    pub async fn get_user(&self, user_id: Uuid) -> Option<User> {
        let universe_state = self.state.read().await;
        universe_state
            .users
            .get(&user_id)
            .map(|x| x.user.clone())
    }

    /// Authenticates a user.
    ///
    /// If the user is already authenticated this returns an error
    pub async fn authenticate_user(
        &self,
        user_id: Uuid,
        nickname: String,
    ) -> Result<User, ProtocolError> {
        let mut universe_state = self.state.write().await;
        if let Some(user_state) = universe_state.users.get_mut(&user_id) {
            if user_state.is_authenticated {
                Err(ProtocolError::new(
                    ProtocolErrorKind::AlreadyAuthenticated,
                    "cannot authenticate twice",
                ))
            } else {
                user_state.is_authenticated = true;
                user_state.user.nickname = nickname;
                Ok(user_state.user.clone())
            }
        } else {
            Err(ProtocolError::new(
                ProtocolErrorKind::InternalError,
                "couldn't find user in state",
            ))
        }
    }

    /// Checks if the user is authenticated.
    pub async fn user_is_authenticated(&self, user_id: Uuid) -> bool {
        let universe_state = self.state.read().await;
        if let Some(ref state) = universe_state.users.get(&user_id) {
            state.is_authenticated
        } else {
            false
        }
    }

    /// Unregisters a user.
    pub async fn remove_user(&self, user_id: Uuid) {
        let mut universe_state = self.state.write().await;
        universe_state.users.remove(&user_id);
    }

    /// Sets the current game of a user.
    pub async fn set_user_game_id(&self, user_id: Uuid, game_id: Option<Uuid>) -> bool {
        let mut universe_state = self.state.write().await;
        if let Some(state) = universe_state.users.get_mut(&user_id) {
            state.game_id = game_id;
            true
        } else {
            false
        }
    }

    /// Returns a game by ID
    pub async fn get_game(&self, game_id: Uuid) -> Option<Arc<Game<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>>> {
        let universe_state = self.state.read().await;
        universe_state.games.get(&game_id).cloned()
    }

    /// Removes a game from the universe.
    pub async fn remove_game(&self, game_id: Uuid) -> bool {
        let mut universe_state = self.state.write().await;
        universe_state.games.remove(&game_id).is_some()
    }

    /// Returns the game a user is in.
    pub async fn get_user_game(&self, user_id: Uuid) -> Option<Arc<Game<GameStateType, GamePlayerStateT, GameStateSnapshotT, PlayEventT>>> {
        let universe_state = self.state.read().await;
        universe_state
            .users
            .get(&user_id)
            .and_then(|user| user.game_id)
            .and_then(|game_id| universe_state.games.get(&game_id))
            .cloned()
    }

    /// Find a game with the user
    pub async fn find_user_game(&self, game_id: Uuid, user_id: Uuid) -> Option<User> {
        let universe_state = self.state.read().await;
        let mut player = None;
        if let Some(game) = universe_state.games.get(&game_id) {
            player = game.get_player(&user_id).await;
        }
        player.map(|pl| pl.into())
    }

    /// Makes the user leave the game they are in.
    pub async fn remove_user_from_game(&self, user_id: Uuid) {
        if let Some(game) = self.get_user_game(user_id).await {
            game.remove_user(user_id).await;
        }
    }

    /// Send a message to a single user.
    pub async fn send(&self, user_id: Uuid, message: &Message<GamePlayerStateT, GameStateSnapshotT, PlayEventT>) {
        let universe_state = self.state.write().await;
        if let Some(ref state) = universe_state.users.get(&user_id) {
            let s = serde_json::to_string(message).unwrap();
            if let Err(_disconnected) = state.tx.send(Ok(ws::Message::text(s))) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}
