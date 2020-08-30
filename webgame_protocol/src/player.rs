use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}

pub trait PlayerState: Send+Serialize+for<'de> Deserialize<'de>+Debug+Clone+PartialEq+Sync {
    fn player(self) -> PlayerInfo;
}
