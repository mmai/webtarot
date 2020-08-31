use serde::{Deserialize, Serialize};
use uuid::Uuid;

use webgame_protocol::{GameInfo, GameExtendedInfo, PlayerInfo, ProtocolErrorKind};
use webgame_protocol::ProtocolError as GenericProtocolError;

use crate::game_messages::GamePlayCommand;
use crate::game::{GameStateSnapshot, PlayEvent};
use crate::player::{GamePlayerState, PlayerRole};

impl From<ProtocolError> for GenericProtocolError {
    fn from(error: ProtocolError) -> Self {
        GenericProtocolError::new(
            error.kind,
            error.message      
       )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Ping,
    Authenticate(AuthenticateCommand),
    SendText(SendTextCommand),
    NewGame,
    JoinGame(JoinGameCommand),
    LeaveGame,
    MarkReady,
    Continue,

    GamePlay(GamePlayCommand),
    SetPlayerRole(SetPlayerRoleCommand),

    DebugUi(DebugUiCommand), // Used to send a custom state to a client, allows to quickly view the UI at a given state of the game without having to play all the hands leading to this state.
    ShowUuid, // get uuid of connected client : for use with debugUi
    ShowServerStatus, // get server infos : active games, players connected...
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
pub struct JoinGameCommand {
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetPlayerRoleCommand {
    pub role: PlayerRole,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Connected,
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
    pub games: Vec<GameExtendedInfo>,
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
