use std::fmt::Debug;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}

pub trait PlayerState: Send+Serialize+DeserializeOwned+Debug+Clone+PartialEq+Sync {
    fn player(self) -> PlayerInfo;
}
