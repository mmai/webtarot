use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{GameInfo, GameStateSnapshot, PlayEvent};
use crate::player::{PlayerInfo, GamePlayerState, PlayerRole};
use tarotgame::{cards, bid, deal};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Ping,
    Authenticate(AuthenticateCommand),
    SendText(SendTextCommand),
    ShareCodename(ShareCodenameCommand),
    NewGame,
    JoinGame(JoinGameCommand),
    LeaveGame,
    MarkReady,
    Continue,
    Bid(BidCommand),
    Play(PlayCommand),
    Pass,
    CallKing(CallKingCommand),
    MakeDog(MakeDogCommand),
    SetPlayerRole(SetPlayerRoleCommand),
    DebugUi(DebugUiCommand), // Used to send a custom state to a client, allows to quickly view the UI at a given state of the game without having to play all the hands leading to this state.
    ShowUuid, // get uuid of connected client : for use with debugUi
    ShowServerStatus, // get server infos : active games, players connected...
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorKind {
    /// Client tried to authenticate twice
    AlreadyAuthenticated,
    /// Tried to do something while unauthenticated
    NotAuthenticated,
    /// Client sent in some garbage
    InvalidCommand,
    /// Cannot be done at this time
    BadState,
    /// Something wasn't found
    NotFound,
    /// Invalid input.
    BadInput,
    /// This should never happen.
    InternalError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
    message: String,
}

impl ProtocolError {
    pub fn new<S: Into<String>>(kind: ProtocolErrorKind, s: S) -> ProtocolError {
        ProtocolError {
            kind,
            message: s.into(),
        }
    }

    pub fn kind(&self) -> ProtocolErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<deal::PlayError> for ProtocolError {
    fn from(error: deal::PlayError) -> Self {
        ProtocolError {
            kind: ProtocolErrorKind::BadState,
            message: format!("play error: {}", error),
        }
    }
}

impl From<bid::BidError> for ProtocolError {
    fn from(error: bid::BidError) -> Self {
        ProtocolError {
            kind: ProtocolErrorKind::BadState,
            message: format!("bid error: {}", error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticateCommand {
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DebugUiCommand {
    pub player_id: Uuid,
    pub snapshot: GameStateSnapshot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendTextCommand {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShareCodenameCommand {
    pub codename: String,
    pub number: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinGameCommand {
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetPlayerRoleCommand {
    pub role: PlayerRole,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidCommand {
    pub target: bid::Target,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayCommand {
    pub card: cards::Card,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallKingCommand {
    pub card: cards::Card,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MakeDogCommand {
    pub cards: cards::Hand,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Pong,
    ServerStatus(ServerStatus),
    Chat(ChatMessage),
    PlayerConnected(GamePlayerState),
    PlayerDisconnected(PlayerDisconnectedMessage),
    PregameStarted,
    GameJoined(GameInfo),
    GameLeft,
    Authenticated(PlayerInfo),
    Error(ProtocolError),
    PlayEvent(PlayEvent),
    GameStateSnapshot(GameStateSnapshot),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerStatus {
    pub players: Vec<Uuid>,
    pub games: Vec<GameInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub player_id: Uuid,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerDisconnectedMessage {
    pub player_id: Uuid,
}
