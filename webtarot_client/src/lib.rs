#![recursion_limit = "2048"]

mod api;
mod components;
mod utils;
mod views;
mod sound_player;

pub(crate) use webtarot_protocol as protocol;
pub(crate) use webgame_protocol as gprotocol;

use std::rc::Rc;
use wasm_bindgen::prelude::*;
use yew_agent::Bridged;
use yew::{html, Component, Context, Html};
use yew_agent::Bridge;
use gloo_timers::callback::Interval;
use gloo_storage::{LocalStorage, Storage};

use weblog::*;

use crate::api::Api;
use crate::protocol::{Message, Command};
use crate::gprotocol::{GameInfo, PlayerInfo, JoinGameCommand};
use crate::views::game::GamePage;
use crate::views::menu::MenuPage;
use crate::views::start::StartPage;

use rust_embed::RustEmbed;
use i18n_embed::{
    gettext::gettext_language_loader,
    WebLanguageRequester,
};

const KEY: &str = "webtarot.self";
const KEY_GAME: &str = "webtarot.game";

#[derive(RustEmbed)]
#[folder = "i18n/mo"]
struct Translations;

static TRANSLATIONS: Translations = Translations;

pub struct App {
    api: Box<dyn Bridge<Api>>,
    _pinger: Interval,
    state: AppState,
    player_info: Option<PlayerInfo>,
    game_info: Option<GameInfo>,
    language: Option<String>,
}

#[derive(Debug, PartialEq)]
enum AppState {
    Start,
    Authenticated,
    InGame,
}

pub enum Msg {
    Ping,
    Authenticated(PlayerInfo),
    GameJoined(GameInfo),
    ServerMessage(Message),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // i18n
        let requested_languages = WebLanguageRequester::requested_languages();
        let language_loader = gettext_language_loader!();
        let _res = i18n_embed::select(&language_loader, &TRANSLATIONS, &requested_languages);

        // Keepalive ping
        let link = ctx.link().clone();
        let pinger = Interval::new(50_000, move || {
            link.send_message(Msg::Ping);
        });

        let on_server_message = ctx.link().callback(Msg::ServerMessage);
        let api = Api::bridge(Rc::new(move |msg| on_server_message.emit(msg)));

        let player_info: Option<PlayerInfo> = LocalStorage::get(KEY).ok();
        if let Some(ref info) = player_info {
            console_log!(format!("player info: {:?}", info));
        }

        let game_info: Option<GameInfo> = LocalStorage::get(KEY_GAME).ok();
        if let Some(ref info) = game_info {
            console_log!(format!("game info: {:?}", info));
        }

        let language = requested_languages.first().clone().map(|l| {
            l.language.as_str().to_owned()
        });

        App {
            api,
            _pinger: pinger,
            state: AppState::Start,
            player_info,
            game_info,
            language,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Authenticated(player_info) => {
                self.state = AppState::Authenticated;
                LocalStorage::set(KEY, &player_info).ok();
                self.player_info = Some(player_info);

                // Try to connect to a game if the url contains a gamecode
                let str_url = web_sys::window().unwrap().document().unwrap().url().unwrap();
                let game_code: Option<String> = url::Url::parse(&str_url).unwrap()
                    .query_pairs()
                    .find(|(name, _)| name == "game")
                    .map(|pair| pair.1.into());
                if let Some(join_code) = game_code {
                    self.api.send(Command::JoinGame(JoinGameCommand { join_code }));
                }
            }
            Msg::GameJoined(game_info) => {
                self.state = AppState::InGame;
                LocalStorage::set(KEY_GAME, &game_info).ok();
                self.game_info = Some(game_info);
            }
            Msg::ServerMessage(Message::Connected) => {}
            Msg::ServerMessage(Message::GameLeft) => {
                self.state = AppState::Start;
                self.game_info = None;
            }
            Msg::Ping => {
                self.api.send(Command::Ping);
            }
            Msg::ServerMessage(_) => {}
        }
        true
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            {match self.state {
                AppState::Start => html! {
                    <StartPage
                        on_authenticate={ctx.link().callback(Msg::Authenticated)} />
                },
                AppState::Authenticated => {
                    html! {
                        <MenuPage
                            player_info={self.player_info.as_ref().unwrap().clone()}
                            on_game_joined={ctx.link().callback(Msg::GameJoined)} />
                    }
                },
                AppState::InGame => html! {
                    <GamePage
                        player_info={self.player_info.as_ref().unwrap().clone()}
                        game_info={self.game_info.as_ref().unwrap().clone()}
                        language={self.language.clone().unwrap_or_else(|| String::from("en"))}
                         />
                }
            }}
        }
    }
}

#[wasm_bindgen]
pub fn run_app() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
