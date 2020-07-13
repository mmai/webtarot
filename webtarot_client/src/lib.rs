#![recursion_limit = "2048"]

#[macro_use]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod api;
mod components;
mod utils;
mod views;
mod sound_player;

use wasm_bindgen::prelude::*;
use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, ShouldRender};
use yew::services::IntervalService;
use yew::services::interval::IntervalTask;
use yew::services::storage::{Area, StorageService};
use yew::format::Json;

use crate::api::Api;
use crate::protocol::{GameInfo, Message, PlayerInfo, Command};
use crate::views::game::GamePage;
use crate::views::menu::MenuPage;
use crate::views::start::StartPage;

use yew::prelude::*;

use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use i18n_embed::{
    language_loader, I18nEmbed,
    WebLanguageRequester,
};

const KEY: &str = "webtarot.self";

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);//Creates language loader struct
lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}
static TRANSLATIONS: Translations = Translations;

pub struct App {
    _api: Box<dyn Bridge<Api>>,
    link: ComponentLink<Self>,
    storage: StorageService,
    state: AppState,
    player_info: Option<PlayerInfo>,
    game_info: Option<GameInfo>,
}

#[derive(Debug)]
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

fn spawn_pings(
    interval_service: &mut IntervalService,
    link: &ComponentLink<App>,
) -> IntervalTask {
    interval_service.spawn(
        std::time::Duration::from_secs(50),
        link.callback(|()| Msg::Ping),
    )
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
        let player_info = {
            if let Json(Ok(restored_info)) = storage.restore(KEY) {
                log!("player info: {:?}", restored_info);
                Some(restored_info)
            } else {
                None 
            }
        };

        //i18N
        let requested_languages = WebLanguageRequester::requested_languages();
        i18n_embed::select(&*LANGUAGE_LOADER, &TRANSLATIONS, &requested_languages);

        //Ping to keep alive websocket
        let mut interval_service = IntervalService::new();
        let _pinger = spawn_pings(&mut interval_service, &link);

        let on_server_message = link.callback(Msg::ServerMessage);
        let _api = Api::bridge(on_server_message);
        App {
            storage,
            link,
            _api,
            state: AppState::Start,
            player_info,
            game_info: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(player_info) => {
                self.state = AppState::Authenticated;
                self.storage.store(KEY, Json(&player_info));
                self.player_info = Some(player_info);
            }
            Msg::GameJoined(game_info) => {
                self.state = AppState::InGame;
                self.game_info = Some(game_info);
            }
            Msg::ServerMessage(Message::GameLeft) => {
                self.state = AppState::Authenticated;
                self.game_info = None;
            }
            Msg::Ping => {
                log!("sending ping");
                self._api.send(Command::Ping);
            }
            Msg::ServerMessage(_) => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            {match self.state {
                AppState::Start => html! {
                    <StartPage on_authenticate=self.link.callback(Msg::Authenticated) />
                },
                AppState::Authenticated => html! {
                    <MenuPage
                        player_info=self.player_info.as_ref().unwrap().clone(),
                        on_game_joined=self.link.callback(Msg::GameJoined) />
                },
                AppState::InGame => html! {
                    <GamePage
                        player_info=self.player_info.as_ref().unwrap().clone(),
                        game_info=self.game_info.as_ref().unwrap().clone(),
                         />
                }
            }}
        }
    }
}

pub(crate) use webtarot_protocol as protocol;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_logger::init();
    yew::start_app::<App>();
    Ok(())
}
