use std::rc::Rc;

use tr::tr;

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

                        let scores = self.game_state.scores.last().unwrap_or(&[0.0;5]);
                        let my_points= scores[state.pos.to_n()];
                        let mut player_classes = vec!["player"];
                        if is_my_turn {
                            player_classes.push("current-player");
                        }
                        player_classes.push(
                            match state.role {
                                PlayerRole::Taker => "role-taker",
                                PlayerRole::Partner => "role-partner",
                                PlayerRole::Opponent => "role-opponent",
                                _ => "role-unknown",
                            }
                        );

                        html! {

                        <div class=player_classes>
                        <div class="nickname withtooltip">
                        {&state.player.nickname}
                        {
                            if self.game_state.turn == Turn::Pregame &&
                                state.ready {
                                html! { tr!(" — ready") }
                            } else {
                                html!{}
                            }
                        }
                        {
                            if self.game_state.turn != Turn::Pregame {
                                html! {
                                    <span class="tooltip">{ tr!("points : {0}", my_points )  }</span>
                                }
                            } else {
                                html!{}
                            }
                        }
                        </div>
                        <div class="action">
                        {
                            if let Some(card) = card_played {
                                let style =format!("cursor: default; --bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
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
