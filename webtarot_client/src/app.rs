use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, ShouldRender};
use yew::services::IntervalService;
use yew::services::interval::IntervalTask;

use crate::api::Api;
use crate::protocol::{GameInfo, Message, PlayerInfo, Command};
use crate::views::game::GamePage;
use crate::views::menu::MenuPage;
use crate::views::start::StartPage;

pub struct App {
    _api: Box<dyn Bridge<Api>>,
    link: ComponentLink<Self>,
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
        std::time::Duration::from_secs(5),
        link.callback(|()| Msg::Ping),
    )
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        //Ping to keep alive websocket
        // let mut interval = IntervalService::new();
        // let duration = std::time::Duration::from_secs(3);
        // let callback = link.callback(|()| Msg::Ping);
        // let task = interval.spawn(duration, callback);
        let mut interval_service = IntervalService::new();
        let pinger = spawn_pings(&mut interval_service, &link);

        let callback = |_| {
            log!("Example of a standalone callback.");
        };
        let mut interval = IntervalService::new();
        let handle = interval.spawn(std::time::Duration::from_secs(10), callback.into());

        let on_server_message = link.callback(Msg::ServerMessage);
        let _api = Api::bridge(on_server_message);
        App {
            link,
            _api,
            state: AppState::Start,
            player_info: None,
            game_info: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(player_info) => {
                self.state = AppState::Authenticated;
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
