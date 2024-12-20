use std::sync::Arc;
use uuid::Uuid;

use crate::protocol::GameState;
use crate::webgame_server::game::Game;
use crate::webgame_server::universe::Universe;
use webgame_protocol::GameManager;

use crate::protocol::{ChatMessage, Message, ProtocolError, ProtocolErrorKind};

use crate::tarot_protocol::{
    AnnounceCommand, BidCommand, CallKingCommand, GameEventsListener, GamePlayCommand,
    MakeDogCommand, PlayCommand, PlayEvent, SetPlayerRoleCommand, TarotGameManager, TarotGameState,
};

//see https://users.rust-lang.org/t/how-to-store-async-function-pointer/38343/4
type DynFut<T> = ::std::pin::Pin<Box<dyn Send + ::std::future::Future<Output = T>>>;

pub fn on_gameplay(
    universe: Arc<Universe<TarotGameState, PlayEvent>>,
    user_id: Uuid,
    cmd: GamePlayCommand,
) -> DynFut<Result<(), ProtocolError>> {
    Box::pin(async move {
        if let Some(game) = universe.get_user_game(user_id).await {
            match cmd {
                GamePlayCommand::Bid(cmd) => on_player_bid(game, user_id, cmd).await,
                GamePlayCommand::Announce(cmd) => on_player_announce(game, user_id, cmd).await,
                GamePlayCommand::Play(cmd) => on_player_play(game, user_id, cmd).await,
                GamePlayCommand::CallKing(cmd) => on_player_call_king(game, user_id, cmd).await,
                GamePlayCommand::MakeDog(cmd) => on_player_make_dog(game, user_id, cmd).await,
                GamePlayCommand::Pass => on_player_pass(game, user_id).await,
            }
        } else {
            Err(ProtocolError::new(
                ProtocolErrorKind::BadState,
                "not in a game",
            ))
        }
    })
}

pub fn on_player_set_role(
    universe: Arc<Universe<TarotGameState, PlayEvent>>,
    user_id: Uuid,
    cmd: SetPlayerRoleCommand,
) -> DynFut<Result<(), ProtocolError>> {
    Box::pin(async move {
        if let Some(game) = universe.get_user_game(user_id).await {
            if !game.is_joinable().await {
                return Err(ProtocolError::new(
                    ProtocolErrorKind::BadState,
                    "cannot set role because game is not not joinable",
                ));
            }

            let game_state = game.state_handle();
            {
                let mut game_state = game_state.lock().await;
                game_state.set_player_role(user_id, cmd.role);
            }
            game.mark_player_ready(user_id).await;
            // game.set_player_not_ready(user_id).await;

            game.broadcast_current_state().await;
            Ok(())
        } else {
            Err(ProtocolError::new(
                ProtocolErrorKind::BadState,
                "not in a game",
            ))
        }
    })
}

pub async fn on_player_bid(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: BidCommand,
) -> Result<(), ProtocolError> {
    game.broadcast(&Message::Chat(ChatMessage {
        player_id,
        text: format!("{:?}", cmd.target),
    }))
    .await;

    let game_state = game.state_handle();
    {
        //lock
        let mut game_state = game_state.lock().await;
        game_state.set_bid(player_id, cmd.target, cmd.slam)?;
    }
    game.broadcast_current_state().await;

    Ok(())
}

pub async fn on_player_announce(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: AnnounceCommand,
) -> Result<(), ProtocolError> {
    let game_state = game.state_handle();
    let mut game_state = game_state.lock().await;
    let ann = cmd.announce.clone();
    if let Err(e) = game_state.set_announce(player_id, cmd.announce) {
        drop(game_state);
        game.send(player_id, &Message::Error(e.into())).await;
    } else {
        drop(game_state);
        game.broadcast(&Message::PlayEvent(PlayEvent::Announce(player_id, ann)))
            .await;
    }
    Ok(())
}

struct TarotEventsListener {
    game_id: Uuid,
    // game: Arc<Game<TarotGameState, PlayEvent>>,
    events_states: Vec<(PlayEvent, TarotGameState)>,
}

impl PartialEq for TarotEventsListener {
    fn eq(&self, other: &Self) -> bool {
        self.game_id == other.game_id
    }
}

impl GameEventsListener<(PlayEvent, TarotGameState)> for TarotEventsListener {
    fn notify(&mut self, event: &(PlayEvent, TarotGameState)) {
        println!("Listener received event {:?}!", event.0);
        self.events_states.push((*event).clone());
    }
}

pub async fn on_player_play(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: PlayCommand,
) -> Result<(), ProtocolError> {
    let game_state_handle = game.state_handle();
    let mut listener = TarotEventsListener {
        game_id: game.id,
        events_states: vec![],
    };
    let mut game_state = game_state_handle.lock().await;
    let mut game_manager = TarotGameManager::new(&mut game_state);
    game_manager.register_listener(&mut listener);
    let play_result = game_manager.set_play(player_id, cmd.card);
    drop(game_manager);
    if let Err(e) = play_result {
        drop(game_state);
        drop(game_state_handle);
        game.send(player_id, &Message::Error(e.into())).await; //1
    } else {
        game_state
            .deal_history
            .push((player_id.clone(), cmd.clone()));
        drop(game_state);
        drop(game_state_handle);
        // We don't show played cards anymore in the chat box
        // game.broadcast(&Message::PlayEvent(PlayEvent::Play ( player_id, cmd.card ))).await; // uncomment to show played cards
        for (event, state) in listener.events_states {
            // println!("new state event {:?}!", event);
            game.broadcast_state(&state).await;
            game.broadcast(&Message::PlayEvent(event)).await;
        }
        game.broadcast_current_state().await;
    }
    Ok(())
}

pub async fn on_player_pass(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    let game_state = game.state_handle();
    {
        let mut game_state = game_state.lock().await;
        game_state.set_pass(player_id)?;
    }
    game.broadcast(&Message::Chat(ChatMessage {
        player_id,
        text: format!("pass"),
    }))
    .await;
    game.broadcast_current_state().await;
    Ok(())
}

pub async fn on_player_call_king(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: CallKingCommand,
) -> Result<(), ProtocolError> {
    game.broadcast(&Message::Chat(ChatMessage {
        player_id,
        text: format!("call king: {}", cmd.card.to_string()),
    }))
    .await;
    let game_state = game.state_handle();
    {
        let mut game_state = game_state.lock().await;
        game_state.call_king(player_id, cmd.card);
    }
    game.broadcast_current_state().await;
    Ok(())
}

pub async fn on_player_make_dog(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: MakeDogCommand,
) -> Result<(), ProtocolError> {
    let game_state = game.state_handle();
    let mut game_state = game_state.lock().await;
    if let Err(e) = game_state.make_dog(player_id, cmd.cards, cmd.slam) {
        drop(game_state);
        game.send(player_id, &Message::Error(e.into())).await;
    } else {
        drop(game_state);
        game.broadcast_current_state().await;
    }
    Ok(())
}
