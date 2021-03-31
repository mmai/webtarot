mod dispatcher;

pub(crate) use webgame_server;
pub(crate) use webgame_protocol as protocol;
pub(crate) use webtarot_protocol as tarot_protocol;

pub(crate) use webtarot_bot as tarot_bot;

#[tokio::main]
pub async fn main() {
    let version = format!("{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
    // let author = format!("{}", env!("CARGO_PKG_AUTHORS"));
    let author = env!("CARGO_PKG_AUTHORS");
    // let name = format!("{}", env!("CARGO_PKG_NAME"));
    let name = env!("CARGO_PKG_NAME");

    webgame_server::launcher::launch(
        name, version, author,
        dispatcher::on_gameplay,
        dispatcher::on_player_set_role,
        tarot_bot::socket_listener::start
        ).await;
}

