use std::rc::Rc;

use tr::tr;

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GameStateSnapshot, DealSnapshot, PlayerRole, GamePlayerState, Turn};

#[derive(Clone, Properties)]
pub struct Props {
    pub players: Vec<GamePlayerState>,
    pub players_chat: Vec<Option<String>>,
    pub game_state: Rc<GameStateSnapshot>,
    pub language: String,
}

pub struct PlayerList {
    players: Vec<GamePlayerState>,
    players_chat: Vec<Option<String>>,
    game_state: Rc<GameStateSnapshot>,
    contract_info: String,
    language: String,
}

impl PlayerList {
    fn update_contract_info(&mut self) {
        self.contract_info = if let Some(contract) = &self.game_state.deal.contract {
             let king_info = if let Some(king) = self.game_state.deal.king {
                format!(" ({})", king.to_locale_string(&self.language))
             } else { "".into() };
             format!("{}{}", contract.to_string(), king_info)
        } else {
            "".into()
        }
    }
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        PlayerList {
            players: props.players,
            players_chat: props.players_chat,
            game_state: props.game_state,
            contract_info: "".into(),
            language: props.language,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.players = props.players;
        self.players_chat = props.players_chat;
        self.game_state = props.game_state;
        self.update_contract_info();
        true
    }

    fn view(&self) -> Html {
        let nb_players = self.game_state.players[0].pos.count as usize;
        let empty_scores = vec![0.0;nb_players];
        html! {
            <section class="players">
                {
                    for self.players.iter().map(|state| {
                        let chat = &self.players_chat[state.pos.to_n()];
                        let is_my_turn = self.game_state.get_playing_pos() == Some(state.pos);
                        let card_played = self.game_state.deal.last_trick.card_played(state.pos);
                        let str_card: String = if let Some(card) = card_played { format!(" {}", card.to_locale_string(&self.language)) } else { "".into() };

                        // XXX incorrect : scores are known at the end of the trick 
                        // let scores = self.game_state.scores.last().unwrap_or(&empty_scores);
                        // let my_points= scores[state.pos.to_n()];
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
                        { if chat.is_some() {
                                html!{
                                    <div class="player-chat">
                                        <div class="player-msg">{{ chat.as_ref().unwrap() }}</div>
                                    </div>
                                }
                            } else { html!{} }
                        }
                        <div class="nickname withtooltip">
                            {&state.player.nickname}
                            <span class="card-info"> {str_card} </span>
                        // {
                        //     if self.game_state.turn != Turn::Pregame {
                        //         html! {
                        //             <span class="tooltip">{ tr!("points : {0}", my_points )  }</span>
                        //         }
                        //     } else {
                        //         html!{}
                        //     }
                        // }
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
