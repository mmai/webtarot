use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}

pub trait PlayerState<'de>: Send+Serialize+Deserialize<'de>+Debug+Clone+PartialEq {
    fn player(self) -> PlayerInfo;
}

pub trait PlayerStatea<'de>: Send+Serialize+Deserialize<'de>+Debug+Clone+PartialEq {
    fn player(self) -> PlayerInfo;
}
