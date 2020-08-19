use uuid::Uuid;
use std::sync::Arc;

use crate::universe::Universe;
use crate::game::Game;

use crate::protocol::{ 
    GamePlayCommand, 
    BidCommand, PlayCommand, CallKingCommand, MakeDogCommand,
    Message, ChatMessage,
    PlayEvent,
    ProtocolError, ProtocolErrorKind 
};

pub async fn on_gameplay(
    universe: Arc<Universe>,
    user_id: Uuid,
    cmd: GamePlayCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_user_game(user_id).await {

        match cmd {
            GamePlayCommand::Bid(cmd) => on_player_bid(game, user_id, cmd).await,
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
}                                
                                 
pub async fn on_player_bid(
    game: Arc<Game>,
    player_id: Uuid,
    cmd: BidCommand,
) -> Result<(), ProtocolError> {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("bid: {:?}", cmd.target),
        }))
        .await;

        let game_state = game.state_handle();
        let mut game_state = game_state.lock().await;
        game_state.set_bid(player_id, cmd.target)?;

        game.broadcast_state().await;
        Ok(())
}

pub async fn on_player_play(
    game: Arc<Game>,
    player_id: Uuid,
    cmd: PlayCommand,
) -> Result<(), ProtocolError> {
        let game_state = game.state_handle();
        let mut game_state = game_state.lock().await;
        if let Err(e) = game_state.set_play(player_id, cmd.card) {
            game.send(player_id, &Message::Error(e)).await;
        } else {
            game.broadcast(&Message::PlayEvent(PlayEvent::Play ( player_id, cmd.card )))
            .await;
            game.broadcast_state().await;
        }
        Ok(())
}


pub async fn on_player_pass(
    game: Arc<Game>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
        let game_state = game.state_handle();
        let mut game_state = game_state.lock().await;
        game_state.set_pass(player_id)?;
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("pass"),
        }))
        .await;
        game.broadcast_state().await;
        Ok(())
}


pub async fn on_player_call_king(
    game: Arc<Game>,
    player_id: Uuid,
    cmd: CallKingCommand,
) -> Result<(), ProtocolError> {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: format!("call king: {}", cmd.card.to_string()),
        }))
        .await;
        let game_state = game.state_handle();
        let mut game_state = game_state.lock().await;
        game_state.set_pass(player_id)?;
        game_state.call_king(player_id, cmd.card);
        game.broadcast_state().await;
        Ok(())
}

pub async fn on_player_make_dog(
    game: Arc<Game>,
    player_id: Uuid,
    cmd: MakeDogCommand,
) -> Result<(), ProtocolError> {
        let game_state = game.state_handle();
        let mut game_state = game_state.lock().await;
        game_state.make_dog(player_id, cmd.cards);
        game.broadcast_state().await;
        Ok(())
}
