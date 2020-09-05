use tr::tr;

use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::protocol::{AuthenticateCommand, Command, Message};
use crate::gprotocol::{PlayerInfo};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub on_authenticate: Callback<PlayerInfo>,
}

pub struct StartPage {
    link: ComponentLink<StartPage>,
    api: Box<dyn Bridge<Api>>,
    nickname: String,
    on_authenticate: Callback<PlayerInfo>,
    error: Option<String>,
}

pub enum Msg {
    Authenticate,
    ServerMessage(Message),
    SetNickname(String),
    Ignore,
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        StartPage {                   
            link,                     
            api,                      
            nickname: "".into(),      
            on_authenticate: props.on_authenticate,
            error: None,              
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticate => {
                self.api.send(Command::Authenticate(AuthenticateCommand {
                    nickname: self.nickname.clone(),
                }));
            }
            Msg::ServerMessage(message) => match message {
                Message::Authenticated(data) => {
                    self.on_authenticate.emit(data);
                }
                Message::Error(err) => {
                    self.error = Some(err.message().to_string());
                }
                _ => {}
            },
            Msg::SetNickname(nickname) => {
                self.nickname = nickname;
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        let nickname_placeholder_text = tr!("nickname");
        html! {
            <div class="wrapper">
                <h1>{ tr!("Let's play Tarot together") }</h1>
                <p class="explanation">
                    { tr!("Give yourself a nickname to play:") }
                </p>
                <div class="toolbar">
                    <input value=&self.nickname
                        placeholder=nickname_placeholder_text 
                        onkeypress=self.link.callback(|event: KeyboardEvent| {
                            dbg!(event.key());
                            if event.key() == "Enter" {
                                Msg::Authenticate
                            } else {
                                Msg::Ignore
                            }
                        })
                        oninput=self.link.callback(|e: InputData| Msg::SetNickname(e.value)) />
                    <button
                        class="primary"
                        onclick=self.link.callback(|_| Msg::Authenticate)>{ tr!("Play") }</button>
                </div>
                {
                    if let Some(ref error) = self.error {
                        html! {
                            <p class="error">{tr!("not good: {0}", error)}</p>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
}
