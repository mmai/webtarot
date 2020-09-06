use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use tr::tr;

use crate::api::Api;
use crate::protocol::{Command, JoinGameCommand, Message};
use crate::gprotocol::{GameInfo, PlayerInfo};
use crate::utils::format_join_code;

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub on_game_joined: Callback<GameInfo>,
}

pub struct MenuPage {
    link: ComponentLink<MenuPage>,
    api: Box<dyn Bridge<Api>>,
    join_code: String,
    player_info: PlayerInfo,
    on_game_joined: Callback<GameInfo>,
    error: Option<String>,
}

pub enum Msg {
    Ignore,
    NewGame,
    JoinGame,
    ServerMessage(Message),
    SetJoinCode(String),
}

impl Component for MenuPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        MenuPage {
            link,
            api,
            join_code: "".into(),
            player_info: props.player_info,
            on_game_joined: props.on_game_joined,
            error: None,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewGame => {
                log::info!("New Game");
                self.api.send(Command::NewGame);
            }
            Msg::JoinGame => {
                log::info!("Join Game");
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
                self.join_code = format_join_code(&join_code);
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="wrapper">
                <h1>{tr!("Hello {0}!", &self.player_info.nickname)}</h1>
                <p class="explanation">{ tr!("Enter the code of a game to join") }</p>
                <div class="toolbar">
                    <input value=&self.join_code
                        size="7"
                        placeholder="CODE"
                        onkeypress=self.link.callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::JoinGame
                            } else {
                                Msg::Ignore
                            }
                        })
                        oninput=self.link.callback(|e: InputData| Msg::SetJoinCode(e.value)) />
                    <button class="primary" onclick=self.link.callback(|_| Msg::JoinGame)>{ tr!("Join Game")}</button>
                </div>
                <p class="explanation">{ tr!("...or start a new game.")}</p>
                <div class="toolbar">
                    <button class="primary" onclick=self.link.callback(|_| Msg::NewGame)>{ tr!("New Game")}</button>
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
