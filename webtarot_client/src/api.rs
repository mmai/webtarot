use std::collections::HashSet;

use yew::agent::{Agent, AgentLink, Context, HandlerId};
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::storage::{Area, StorageService};

use crate::protocol::{Command, Message};
use crate::gprotocol::{PlayerInfo, GameInfo};

#[derive(Debug)]
pub enum ApiState {
    Connecting,
    Connected,
    Disconnected,
}

pub enum Msg {
    ServerMessage(Message),
    Connected,
    ConnectionLost,
    Ignore,
}

#[derive(Debug)]
pub struct Api {
    link: AgentLink<Api>,
    ws: WebSocketTask,
    subscribers: HashSet<HandlerId>,
    state: ApiState,
}

fn get_websocket_location(_uuid: Option<&str>) -> String {
    let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
    let player_info: Option<PlayerInfo> = if let Json(Ok(restored_info)) =  storage.restore("webtarot.self") {
        Some(restored_info)
    } else {
        None
    };
    let game_info: Option<GameInfo> = if let Json(Ok(restored_info)) =  storage.restore("webtarot.game") {
        Some(restored_info)
    } else {
        None
    };


    let location = web_sys::window().unwrap().location();
    format!(
        "{}://{}/ws/{}_{}",
        if location.protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        },
        location.host().unwrap(),
        game_info.map(|ginfo| ginfo.game_id.to_string()).unwrap_or("new".into()),
        player_info.map(|pinfo| pinfo.id.to_string()).unwrap_or("new".into()),
    )
}

impl Agent for Api {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Command;
    type Output = Message;

    fn create(link: AgentLink<Api>) -> Api {
        log::info!("Connecting to server");
        let on_message = link.callback(|Json(data)| match data {
            Ok(message) => Msg::ServerMessage(message),
            Err(err) => {
                log::error!("websocket error: {:?}", err);
                Msg::Ignore
            }
        });
        let on_notification = link.callback(|status| match status {
            WebSocketStatus::Opened => Msg::Connected,
            WebSocketStatus::Closed | WebSocketStatus::Error => Msg::ConnectionLost,
        });
        let ws = WebSocketService::connect(&get_websocket_location(None), on_message, on_notification)
            .unwrap();

        Api {
            link,
            ws,
            state: ApiState::Connecting,
            subscribers: HashSet::new(),
        }
    }

    fn handle_input(&mut self, input: Self::Input, _: HandlerId) {
        log::debug!("Sending command: {:?}", &input);
        self.ws.send(Json(&input));
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::ServerMessage(msg) => {
                log::debug!("Server message: {:?}", msg);
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            }
            Msg::Connected => {
                log::info!("Connected web socket!");
                self.state = ApiState::Connected;
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, Message::Connected);
                }
            }
            Msg::ConnectionLost => {
                log::info!("Lost connection on web socket!");
                self.state = ApiState::Disconnected;
            }
            Msg::Ignore => {}
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn destroy(&mut self) {
        log::info!("destroying API service");
    }
}
