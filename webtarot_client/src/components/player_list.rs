use std::rc::Rc;


use yew::{html, Component, Context, Html, Properties};

use crate::protocol::{GamePlayerState, GameStateSnapshot, PlayerRole};

#[derive(Clone, Properties)]
pub struct Props {
    pub players: Vec<GamePlayerState>,
    pub players_chat: Vec<Option<String>>,
    pub game_state: Rc<GameStateSnapshot>,
    pub language: String,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
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
            } else {
                "".into()
            };
            format!("{}{}", contract.to_string(), king_info)
        } else {
            "".into()
        }
    }
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        PlayerList {
            players: ctx.props().players.clone(),
            players_chat: ctx.props().players_chat.clone(),
            game_state: ctx.props().game_state.clone(),
            contract_info: "".into(),
            language: ctx.props().language.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.players = ctx.props().players.clone();
        self.players_chat = ctx.props().players_chat.clone();
        self.game_state = ctx.props().game_state.clone();
        self.language = ctx.props().language.clone();
        self.update_contract_info();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="players">
                {
                    self.players.iter().map(|state| {
                        let chat = &self.players_chat[state.pos.to_n()];
                        let is_my_turn = self.game_state.get_playing_pos() == Some(state.pos);
                        let card_played = self.game_state.deal.last_trick.card_played(state.pos);
                        let str_card: String = if let Some(card) = card_played { format!(" {}", card.to_locale_string(&self.language)) } else { "".into() };
                        let str_king: String = format!("{}", self.game_state.deal.king.map(|c| c.suit().to_string()).unwrap_or("".to_string()));

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
                        <div class={player_classes.join(" ")}>
                        { if chat.is_some() {
                                html!{
                                    <div class="player-chat">
                                        <div class="player-msg">{ chat.as_ref().unwrap() }</div>
                                    </div>
                                }
                            } else { html!{} }
                        }
                        <div class="nickname withtooltip">
                            { if state.role == PlayerRole::Taker {
                                html! {
                                <span>{str_king} </span>
                                }
                            } else {
                                html! {
                                <span></span>
                                }
                            }
                            }
                            {&state.player.nickname}
                            <span class="card-info"> {str_card} </span>
                        </div>
                        <div class="action">
                        {
                            if let Some(card) = card_played {
                                let style = format!("cursor: default; --bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                html! {
                                    <div class="card" style={style}></div>
                                }
                            } else {
                                html!{
                                    <div></div>
                                }
                            }
                        }
                        </div>
                        </div>
                    }}).collect::<Html>()
                }
            </section>
        }
    }
}
