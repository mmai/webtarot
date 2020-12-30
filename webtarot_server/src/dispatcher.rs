use uuid::Uuid;
use std::sync::Arc;

use crate::webgame_server::universe::Universe;
use crate::webgame_server::game::Game;
use crate::protocol::GameState;

use crate::protocol::{ 
    Message, ChatMessage,
    ProtocolError, ProtocolErrorKind 
};

use crate::tarot_protocol::{ 
    GamePlayCommand, 
    SetPlayerRoleCommand, 
    BidCommand, AnnounceCommand, PlayCommand, CallKingCommand, MakeDogCommand,
    PlayEvent,
    TarotGameState,
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
            game.set_player_not_ready(user_id).await;

            game.broadcast_state().await;
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
            text: format!("bid: {:?}", cmd.target),
        }))
        .await;

        let game_state = game.state_handle();
        { //lock
            let mut game_state = game_state.lock().await;
            game_state.set_bid(player_id, cmd.target, cmd.slam)?;
        }
        game.broadcast_state().await;

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
            game.broadcast(&Message::PlayEvent(PlayEvent::Announce ( player_id, ann))).await;
        }
        Ok(())
}


pub async fn on_player_play(
    game: Arc<Game<TarotGameState, PlayEvent>>,
    player_id: Uuid,
    cmd: PlayCommand,
) -> Result<(), ProtocolError> {
        let game_state_handle = game.state_handle();
        let mut game_state = game_state_handle.lock().await;
        match game_state.set_play(player_id, cmd.card) {
            Err(e) => {
                drop(game_state);
                game.send(player_id, &Message::Error(e.into())).await;
            },
            Ok(Some(play_event)) => {
                drop(game_state);
                game.broadcast_state().await;
                let is_end_deal = play_event ==  PlayEvent::EndDeal;
                game.broadcast(&Message::PlayEvent(play_event)).await;

                let mut game_state = game_state_handle.lock().await;
                if is_end_deal { 
                    game_state.next_deal();
                }

                game_state.update_turn();
                drop(game_state);
                game.broadcast_state().await;
            },
            _ =>  {
                game_state.update_turn();
                drop(game_state);
            // We don't show played cards anymore in the chat box
            // game.broadcast(&Message::PlayEvent(PlayEvent::Play ( player_id, cmd.card ))).await;
                game.broadcast_state().await;
            }, 
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
        game.broadcast_state().await;
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
        game.broadcast_state().await;
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
        game.broadcast_state().await;
    }
    Ok(())
}
