use serde::{Deserialize, Serialize};

use webgame_protocol::{ProtocolErrorKind, Message as GenericMessage, Variant as GenericVariant, Command as GenericCommand};
use webgame_protocol::ProtocolError as GenericProtocolError;

use crate::player::{PlayerRole, GamePlayerState};
use crate::game::{GameStateSnapshot, PlayEvent, VariantSettings};
use crate::game_messages::GamePlayCommand;

impl From<ProtocolError> for GenericProtocolError {
    fn from(error: ProtocolError) -> Self {
        GenericProtocolError::new(
            error.kind,
            error.message      
       )
    }
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
pub struct SetPlayerRoleCommand {
    pub role: PlayerRole,
}

pub type Message = GenericMessage<GamePlayerState, GameStateSnapshot, PlayEvent>;
pub type Variant = GenericVariant<VariantSettings>;
pub type Command = GenericCommand<GamePlayCommand, SetPlayerRoleCommand, GameStateSnapshot, Variant>;
