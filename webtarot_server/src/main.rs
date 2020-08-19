use clap::{Arg, App};

mod dispatcher;

pub(crate) use webgame_server;
pub(crate) use webtarot_protocol as protocol;

use std::net::SocketAddr;

#[tokio::main]
pub async fn main() {
    webgame_server::launcher::launch(dispatcher::on_gameplay).await;
}
