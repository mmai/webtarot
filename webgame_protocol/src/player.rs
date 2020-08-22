use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}

pub trait PlayerState {
    fn player(self) -> PlayerInfo;
}
