use std::convert::Infallible;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use hyper::{service::make_service_fn, Server};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::{ws, Filter};

//For keep alive ping pong
use std::time::Duration;

use crate::protocol::{
    AuthenticateCommand, ChatMessage, ServerStatus, Command, JoinGameCommand, Message, ProtocolError,
    ProtocolErrorKind, SendTextCommand, SetPlayerRoleCommand,
    ShareCodenameCommand,
    PlayCommand, BidCommand, CallKingCommand, MakeDogCommand,
    PlayEvent,
    DebugUiCommand,
};
use crate::universe::Universe;

async fn on_player_connected(universe: Arc<Universe>, ws: ws::WebSocket) {

    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            log::error!("websocket send error: {}", e);
        }
    }));

    let player_id = universe.add_player(tx).await;
    log::info!("player {:#?} connected", player_id);

    //keep alive : send a ping every 50 seconds
    // let when = Duration::from_millis(50000);
    // let interval = tokio::time::interval(when);
    // let task = interval.for_each(move |_| {
    //     let _  = tx.send(Ok(ws::Message::ping(Vec::new())));
    //     Ok(())
    // })
    // .map_err(|e| panic!("delay errored; err={:?}", e));
    // ... or ?
    // let task = loop {
    //     interval.next().await;
    //     tx.send(Ok(ws::Message::ping(Vec::new())));
    //
    // }
    //end keep alive


    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                log::debug!("Got message from websocket: {:?}", &msg);
                if let Err(err) = on_player_message(universe.clone(), player_id, msg).await {
                    universe.send(player_id, &Message::Error(err)).await;
                }
            }
            Err(e) => {
                log::error!("websocket error(uid={}): {}", player_id, e);
                break;
            }
        }
    }

    on_player_disconnected(universe, player_id).await;
}

async fn on_player_disconnected(universe: Arc<Universe>, player_id: Uuid) {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.remove_player(player_id).await;
    }
    universe.remove_player(player_id).await;
    log::info!("user {:#?} disconnected", player_id);
}

async fn on_player_message(
    universe: Arc<Universe>,
    player_id: Uuid,
    msg: ws::Message,
) -> Result<(), ProtocolError> {
    let req_json = match msg.to_str() {
        Ok(text) => text,
        Err(()) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidCommand,
                "not a valid text frame",
            ))
        }
    };

    let cmd: Command = match serde_json::from_str(&req_json) {
        Ok(req) => req,
        Err(err) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidCommand,
                err.to_string(),
            ));
        }
    };

    log::debug!("command: {:?}", &cmd);

    if !universe.player_is_authenticated(player_id).await {
        match cmd {
            Command::Authenticate(data) => on_player_authenticate(universe, player_id, data).await,

            //For debug purposes only
            Command::ShowServerStatus => on_server_status(universe, player_id).await,
            Command::ShowUuid => on_show_uuid(universe, player_id).await,
            Command::DebugUi(data) => on_debug_ui(universe, data).await,

            _ => Err(ProtocolError::new(
                ProtocolErrorKind::NotAuthenticated,
                "cannot perform this command unauthenticated",
            )),
        }
    } else {
        match cmd {
            Command::NewGame => on_new_game(universe, player_id).await,
            Command::JoinGame(cmd) => on_join_game(universe, player_id, cmd).await,
            Command::LeaveGame => on_leave_game(universe, player_id).await,
            Command::MarkReady => on_player_mark_ready(universe, player_id).await,
            Command::Continue => on_player_continue(universe, player_id).await,
            Command::SendText(cmd) => on_player_send_text(universe, player_id, cmd).await,
            Command::ShareCodename(cmd) => on_player_share_codename(universe, player_id, cmd).await,
            Command::SetPlayerRole(cmd) => on_player_set_role(universe, player_id, cmd).await,
            Command::Bid(cmd) => on_player_bid(universe, player_id, cmd).await,
            Command::Play(cmd) => on_player_play(universe, player_id, cmd).await,
            Command::CallKing(cmd) => on_player_call_king(universe, player_id, cmd).await,
            Command::MakeDog(cmd) => on_player_make_dog(universe, player_id, cmd).await,
            Command::Pass => on_player_pass(universe, player_id).await,
            Command::Ping => on_ping(universe, player_id).await,

            //For debug purposes only
            Command::ShowUuid => on_show_uuid(universe, player_id).await,
            Command::DebugUi(data) => on_debug_ui(universe, data).await,
            Command::ShowServerStatus => on_server_status(universe, player_id).await,

            // this should not happen here.
            Command::Authenticate(..) => Err(ProtocolError::new(
                ProtocolErrorKind::AlreadyAuthenticated,
                "cannot authenticate twice",
            )),
        }
    }
}

async fn on_new_game(universe: Arc<Universe>, player_id: Uuid) -> Result<(), ProtocolError> {
    universe.remove_player_from_game(player_id).await;
    let game = universe.new_game().await;
    game.add_player(player_id).await;
    universe
        .send(player_id, &Message::GameJoined(game.game_info()))
        .await;
    game.broadcast_state().await;
    Ok(())
}

async fn on_join_game(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: JoinGameCommand,
) -> Result<(), ProtocolError> {
    let game = universe.join_game(player_id, cmd.join_code).await?;
    universe
        .send(player_id, &Message::GameJoined(game.game_info()))
        .await;
    game.broadcast_state().await;
    Ok(())
}

async fn on_leave_game(universe: Arc<Universe>, player_id: Uuid) -> Result<(), ProtocolError> {
    universe.remove_player_from_game(player_id).await;
    universe.send(player_id, &Message::GameLeft).await;
    Ok(())
}

async fn on_ping(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    universe
        .send(player_id, &Message::Pong)
        .await;
    Ok(())
}

async fn on_show_uuid(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    let pid = universe.show_players(player_id).await[0];
    universe
        .send(player_id, &Message::Chat(ChatMessage { player_id:pid, text:String::new() }))
        .await;
    Ok(())
}

async fn on_server_status(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    let players = universe.show_players(player_id).await;
    let games = universe.show_games().await;
    universe
        .send(player_id, &Message::ServerStatus(ServerStatus { players, games }))
        .await;
    Ok(())
}

async fn on_debug_ui(
    universe: Arc<Universe>,
    cmd: DebugUiCommand,
) -> Result<(), ProtocolError> {
    universe
        .send(cmd.player_id, &Message::GameStateSnapshot(cmd.snapshot))
        .await;
    Ok(())
}

async fn on_player_authenticate(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: AuthenticateCommand,
) -> Result<(), ProtocolError> {
    let nickname = cmd.nickname.trim().to_owned();
    if nickname.is_empty() || nickname.len() > 16 {
        return Err(ProtocolError::new(
            ProtocolErrorKind::BadInput,
            "nickname must be between 1 and 16 characters",
        ));
    }

    let player_info = universe.authenticate_player(player_id, nickname).await?;
    log::info!(
        "player {:?} authenticated as {:?}",
        player_id,
        &player_info.nickname
    );

    universe
        .send(player_id, &Message::Authenticated(player_info.clone()))
        .await;

    Ok(())
}

pub async fn on_player_continue(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.mark_player_ready(player_id).await;
        game.broadcast_state().await;
    }
    Ok(())
}

pub async fn on_player_mark_ready(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if game.is_joinable().await {
            game.mark_player_ready(player_id).await;
            game.broadcast_state().await;
        }
    }
    Ok(())
}

pub async fn on_player_send_text(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: SendTextCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: cmd.text,
        }))
        .await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_share_codename(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: ShareCodenameCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("codename: {} {}", cmd.codename, cmd.number),
        }))
        .await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_set_role(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: SetPlayerRoleCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if !game.is_joinable().await {
            return Err(ProtocolError::new(
                ProtocolErrorKind::BadState,
                "cannot set role because game is not not joinable",
            ));
        }
        game.set_player_role(player_id, cmd.role).await;
        game.set_player_not_ready(player_id).await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_bid(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: BidCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("bid: {:?}", cmd.target),
        }))
        .await;
        game.set_bid(player_id, cmd.target).await?;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_play(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: PlayCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if let Err(e) = game.set_play(player_id, cmd.card).await {
            game.send(player_id, &Message::Error(e)).await;
        } else {
            game.broadcast(&Message::PlayEvent(PlayEvent::Play ( player_id, cmd.card )))
            .await;
            // game.broadcast(&Message::Chat(ChatMessage {
            //     player_id,
            //     text: format!("play: {}", cmd.card.to_string()),
            // }))
            // .await;
            game.broadcast_state().await;
        }
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}


pub async fn on_player_pass(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.set_pass(player_id).await?;
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("pass"),
        }))
        .await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}


pub async fn on_player_call_king(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: CallKingCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("call king: {}", cmd.card.to_string()),
        }))
        .await;
        game.call_king(player_id, cmd.card).await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_make_dog(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: MakeDogCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.make_dog(player_id, cmd.cards).await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn serve(public_dir: String, port: u16) {
    let universe = Arc::new(Universe::new());

    let make_svc = make_service_fn(move |_| {
        let universe = universe.clone();
        let pdir = public_dir.clone();

        let routes = warp::path("ws") // Websockets on /ws entry point
            .and(warp::ws())
            .and(warp::any().map(move || universe.clone()))
            .map(|ws: warp::ws::Ws, universe: Arc<Universe>| {
                ws.on_upgrade(move |ws| on_player_connected(universe, ws))
            })
        // .or(warp::fs::dir("public/")); // Static files
        .or(warp::fs::dir(pdir)); // Static files
        let svc = warp::service(routes);
        async move { Ok::<_, Infallible>(svc) }
    });

    let mut listenfd = listenfd::ListenFd::from_env();
    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], port).into())
    };
    server.serve(make_svc).await.unwrap();
}
