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
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.players = props.players;
        self.game_state = props.game_state;
        true
    }

    fn view(&self) -> Html {
        html! {
            <section class="players">
                {
                    for self.players.iter().map(|state| {
                        let card_played = self.game_state.deal.last_trick.card_played(state.pos);
                        let is_my_turn = self.game_state.get_playing_pos() == Some(state.pos);
                        let mut player_classes = vec!["player"];
                        if is_my_turn {
                            player_classes.push("current-player");
                        }

                        html! {

                        <div class=player_classes>
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
                        {
                            if let Some(card) = card_played {
                                let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                html! {
                                    <div class="card" style={style}></div>
                                }
                            } else {
                                html!{}
                            }
                        }
                        </div>
                        </div>
                    }})
                }
            </section>
        }
    }
}
