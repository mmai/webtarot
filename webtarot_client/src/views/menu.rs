use std::borrow::Cow;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew_agent::Bridged;
use yew::{
    html, Callback, Component, Context, Html, Properties,
};
use yew_agent::Bridge;
use web_sys::{HtmlInputElement, KeyboardEvent};

use tr::tr;
use weblog::*;

use crate::api::Api;
use crate::protocol::{Command, Message, TarotVariant, VariantSettings};
use crate::gprotocol::{GameInfo, PlayerInfo, JoinGameCommand};
use crate::utils::format_join_code;

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub on_game_joined: Callback<GameInfo>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub struct MenuPage {
    api: Box<dyn Bridge<Api>>,
    join_code: Cow<'static, str>,
    player_info: PlayerInfo,
    on_game_joined: Callback<GameInfo>,
    error: Option<String>,
}

pub enum Msg {
    Ignore,
    NewGame(TarotVariant),
    JoinGame,
    ServerMessage(Message),
    SetJoinCode(String),
}

const TAROT3: TarotVariant = TarotVariant {
    parameters: VariantSettings { nb_players: 3 }
};

const TAROT4: TarotVariant = TarotVariant {
    parameters: VariantSettings { nb_players: 4 }
};

const TAROT5: TarotVariant = TarotVariant {
    parameters: VariantSettings { nb_players: 5 }
};

impl Component for MenuPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let on_server_message = ctx.link().callback(Msg::ServerMessage);
        let api = Api::bridge(Rc::new(move |msg| on_server_message.emit(msg)));
        MenuPage {
            api,
            join_code: "".into(),
            player_info: ctx.props().player_info.clone(),
            on_game_joined: ctx.props().on_game_joined.clone(),
            error: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.player_info = ctx.props().player_info.clone();
        self.on_game_joined = ctx.props().on_game_joined.clone();
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame(variant) => {
                console_log!("New Game");
                self.api.send(Command::NewGame(variant));
            }
            Msg::JoinGame => {
                console_log!("Join Game");
                self.api.send(Command::JoinGame(JoinGameCommand {
                    join_code: self.join_code.replace("-", ""),
                }));
            }
            Msg::ServerMessage(message) => match message {
                Message::GameJoined(data) => {
                    self.on_game_joined.emit(data);
                }
                Message::Error(err) => {
                    self.error = Some(err.message().to_string());
                }
                _ => {}
            },
            Msg::SetJoinCode(join_code) => {
                self.join_code = format_join_code(&join_code).into();
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="wrapper">
                <h1>{tr!("Hello {0}!", &self.player_info.nickname)}</h1>
                <p class="explanation">{ tr!("Enter the code of a game to join") }</p>
                <div class="toolbar">
                    <input value={self.join_code.to_string()}
                        size="7"
                        placeholder="CODE"
                        onkeypress={ctx.link().callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::JoinGame
                            } else {
                                Msg::Ignore
                            }
                        })}
                        oninput={ctx.link().callback(|e: web_sys::InputEvent| {
                            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
                            Msg::SetJoinCode(input.value())
                        })} />
                    <button class="primary" onclick={ctx.link().callback(|_| Msg::JoinGame)}>{ tr!("Join Game")}</button>
                </div>
                <p class="explanation">{ tr!("...or start a new game.")}</p>
                <div class="toolbar">
                    <button class="primary" onclick={ctx.link().callback(|_| Msg::NewGame(TAROT3))}>{ tr!("New 3 players Game")}</button>
                    <button class="primary" onclick={ctx.link().callback(|_| Msg::NewGame(TAROT4))}>{ tr!("New 4 players Game")}</button>
                    <button class="primary" onclick={ctx.link().callback(|_| Msg::NewGame(TAROT5))}>{ tr!("New 5 players Game")}</button>
                </div>
                {
                    if let Some(ref error) = self.error {
                        html! {
                            <p class="error">{tr!("Error: {0}", error)}</p>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
}
