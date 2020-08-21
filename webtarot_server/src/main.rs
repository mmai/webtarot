use clap::{Arg, App};

mod dispatcher;

pub(crate) use webgame_server;
pub(crate) use webgame_protocol as protocol;
pub(crate) use webtarot_protocol as tarot_protocol;

use std::net::SocketAddr;
use tarot_protocol::GamePlayCommand;

#[tokio::main]
pub async fn main() {
    webgame_server::launcher::launch<GamePlayCommand>(dispatcher::on_gameplay).await;
}
