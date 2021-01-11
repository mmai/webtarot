#![recursion_limit = "2048"]

mod api;
mod components;
mod utils;
mod views;
mod sound_player;

pub(crate) use webtarot_protocol as protocol;
pub(crate) use webgame_protocol as gprotocol;

use wasm_bindgen::prelude::*;
use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, ShouldRender};
use yew::services::IntervalService;
use yew::services::interval::IntervalTask;
use yew::services::storage::{Area, StorageService};
use yew::format::Json;

use weblog::*;

use crate::api::Api;
use crate::protocol::{Message, Command};
use crate::gprotocol::{GameInfo, PlayerInfo, JoinGameCommand};
use crate::views::game::GamePage;
use crate::views::menu::MenuPage;
use crate::views::start::StartPage;

use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use i18n_embed::{
    language_loader, I18nEmbed,
    WebLanguageRequester,
};

const KEY: &str = "webtarot.self";
const KEY_GAME: &str = "webtarot.game";

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);//Creates language loader struct
lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}
static TRANSLATIONS: Translations = Translations;

pub struct App {
    api: Box<dyn Bridge<Api>>,
    link: ComponentLink<Self>,
    storage: StorageService,
    state: AppState,
    player_info: Option<PlayerInfo>,
    game_info: Option<GameInfo>,
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

fn spawn_pings(
    link: &ComponentLink<App>,
) -> IntervalTask {
    IntervalService::spawn(
        std::time::Duration::from_secs(50),
        link.callback(|()| Msg::Ping),
    )
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");

        //i18N
        let requested_languages = WebLanguageRequester::requested_languages();
        let _res = i18n_embed::select(&*LANGUAGE_LOADER, &TRANSLATIONS, &requested_languages);

        //Ping to keep alive websocket
        let _pinger = spawn_pings(&link);

        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);

        let player_info: Option<PlayerInfo> = {
            if let Json(Ok(restored_info)) =  storage.restore(KEY) {
                console_log!(format!("player info: {:?}", restored_info));
                // log!("player info: {:?}", restored_info);
                Some(restored_info)
            } else {
                None 
            }
        };

        let game_info: Option<GameInfo> = {
            if let Json(Ok(restored_info)) =  storage.restore(KEY_GAME) {
                console_log!(format!("game info: {:?}", restored_info));
                Some(restored_info)
            } else {
                None 
            }
        };

        App {
            storage,
            link,
            api,
            state: AppState::Start,
            player_info,
            game_info,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(player_info) => {
                self.state = AppState::Authenticated;
                self.storage.store(KEY, Json(&player_info));
                self.player_info = Some(player_info);

                // Try to connect to a game if the url contains a gamecode
                let str_url = yew::utils::document().url().unwrap();
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
                self.storage.store(KEY_GAME, Json(&game_info));
                self.game_info = Some(game_info);
            }
            Msg::ServerMessage(Message::Connected) => {
                // Authenticate with stored name
                // if let Some(info) = self.player_info.clone() {
                //     self.api.send(Command::Authenticate(AuthenticateCommand {
                //         nickname: info.nickname,
                //     }));
                // }
            }
            Msg::ServerMessage(Message::GameLeft) => {
                // self.state = AppState::Authenticated;
                self.state = AppState::Start;
                self.game_info = None;
            }
            Msg::Ping => {
                // log::debug!("sending ping");
                self.api.send(Command::Ping);
            }
            Msg::ServerMessage(_) => {}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            {match self.state {
                AppState::Start => html! {
                    <StartPage 
                        on_authenticate=self.link.callback(Msg::Authenticated) />
                },
                AppState::Authenticated => {
                    html! {
                        <MenuPage
                            player_info=self.player_info.as_ref().unwrap().clone(),
                            on_game_joined=self.link.callback(Msg::GameJoined) />
                    }
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

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    // console_log!("run app");
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
    Ok(())
}
