use tr::tr;

use std::borrow::Cow;
use wasm_bindgen::JsCast;
use yew::{
    html, Callback, Component, Context, Html, Properties,
};
use web_sys::{HtmlInputElement, KeyboardEvent};

use crate::api::{Api, ApiBridge};
use crate::protocol::{Command, Message};
use crate::gprotocol::{AuthenticateCommand, PlayerInfo};

#[derive(Clone, Properties)]
pub struct Props {
    pub on_authenticate: Callback<PlayerInfo>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.on_authenticate == other.on_authenticate
    }
}

pub struct StartPage {
    api: ApiBridge,
    nickname: Cow<'static, str>,
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

    fn create(ctx: &Context<Self>) -> Self {
        let api = Api::bridge(ctx.link().callback(Msg::ServerMessage));
        StartPage {
            api,
            nickname: "".into(),
            on_authenticate: ctx.props().on_authenticate.clone(),
            error: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.on_authenticate = ctx.props().on_authenticate.clone();
        false
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Authenticate => {
                self.api.send(Command::Authenticate(AuthenticateCommand {
                    nickname: self.nickname.clone().into(),
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
                self.nickname = nickname.into();
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let nickname_placeholder_text = tr!("nickname");
        html! {
            <div class="wrapper centered">
                <h1>{ "Webtarot" }</h1>
                <p class="explanation">
                    { tr!("Give yourself a nickname to play:") }
                </p>
                <div class="toolbar">
                    <input value={self.nickname.to_string()}
                        placeholder={nickname_placeholder_text}
                        onkeypress={ctx.link().callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::Authenticate
                            } else {
                                Msg::Ignore
                            }
                        })}
                        oninput={ctx.link().callback(|e: web_sys::InputEvent| {
                            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
                            Msg::SetNickname(input.value())
                        })} />
                    <button
                        class="primary"
                        onclick={ctx.link().callback(|_| Msg::Authenticate)}>{ tr!("Play") }</button>
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
