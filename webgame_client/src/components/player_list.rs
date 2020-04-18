use std::rc::Rc;

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GameStateSnapshot, PlayerRole, GamePlayerState, Turn};

#[derive(Clone, Properties)]
pub struct Props {
    pub players: Vec<GamePlayerState>,
    pub game_state: Rc<GameStateSnapshot>,
}

pub struct PlayerList {
    players: Vec<GamePlayerState>,
    game_state: Rc<GameStateSnapshot>,
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        PlayerList {
            players: props.players,
            game_state: props.game_state,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.players = props.players;
        true
    }

    fn view(&self) -> Html {
        html! {
            <section class="players">
                {
                    for self.players.iter().map(|state| html! {

                        <div class="player">
                        <div class="nickname">
                        {&state.player.nickname}
                        {format!("{:?}", &state.pos)}
                        {format!(
                            " {}",
                            match state.role {
                                PlayerRole::Taker => "(Taker)",
                                PlayerRole::Spectator => "(Spectator)",
                                _ => "",
                            }
                        )}
                        {
                            if self.game_state.turn == Turn::Pregame &&
                                state.ready {
                                html! { " — ready" }
                            } else {
                                html!{}
                            }
                        }
                        </div>
                        <div class="action">
                        </div>
                        </div>
                    })
                }
            </section>
        }
    }
}
