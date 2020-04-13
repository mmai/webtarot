use std::rc::Rc;

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GameStateSnapshot, PlayerRole, Turn};

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
}

pub struct PlayerList {
    game_state: Rc<GameStateSnapshot>,
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        PlayerList {
            game_state: props.game_state,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.game_state != props.game_state {
            self.game_state = props.game_state;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <section class="players">
                {
                    for self.game_state.players.iter().map(|state| html! {

                        <div class="player">
                        <div class="nickname">
                        {&state.player.nickname}
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
