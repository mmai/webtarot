use std::collections::HashSet;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use weblog::*;
use yew_agent::{Worker, WorkerLink, HandlerId};
use yew_agent::Public;
use gloo_storage::{LocalStorage, Storage};

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

pub struct Api {
    link: WorkerLink<Api>,
    ws: web_sys::WebSocket,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_open: Closure<dyn FnMut()>,
    _on_close: Closure<dyn FnMut()>,
    subscribers: HashSet<HandlerId>,
    state: ApiState,
}

fn get_websocket_location() -> String {
    let player_info: Option<PlayerInfo> = LocalStorage::get("webtarot.self").ok();
    let game_info: Option<GameInfo> = LocalStorage::get("webtarot.game").ok();

    let location = web_sys::window().unwrap().location();
    format!(
        "{}://{}/ws/{}_{}",
        if location.protocol().unwrap() == "https:" { "wss" } else { "ws" },
        location.host().unwrap(),
        game_info.map(|ginfo| ginfo.game_id.to_string()).unwrap_or_else(|| "new".into()),
        player_info.map(|pinfo| pinfo.id.to_string()).unwrap_or_else(|| "new".into()),
    )
}

impl Worker for Api {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = Command;
    type Output = Message;

    fn create(link: WorkerLink<Api>) -> Api {
        console_log!("Connecting to server");

        let on_message_cb = link.callback(|text: String| {
            match serde_json::from_str::<Message>(&text) {
                Ok(message) => Msg::ServerMessage(message),
                Err(err) => {
                    console_error!(format!("websocket parse error: {:?}", err));
                    Msg::Ignore
                }
            }
        });
        let on_open_cb = link.callback(|_: ()| Msg::Connected);
        let on_close_cb = link.callback(|_: ()| Msg::ConnectionLost);

        let ws_url = get_websocket_location();
        let ws = web_sys::WebSocket::new(&ws_url).unwrap();

        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(text) = e.data().as_string() {
                on_message_cb(text);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        let on_open = Closure::wrap(Box::new(move || {
            on_open_cb(());
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

        let on_close = Closure::wrap(Box::new(move || {
            on_close_cb(());
        }) as Box<dyn FnMut()>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        Api {
            link,
            ws,
            _on_message: on_message,
            _on_open: on_open,
            _on_close: on_close,
            state: ApiState::Connecting,
            subscribers: HashSet::new(),
        }
    }

    fn handle_input(&mut self, input: Self::Input, _: HandlerId) {
        let text = serde_json::to_string(&input).expect("failed to serialize command");
        console_debug!(format!("Sending command: {:?}", &input));
        self.ws.send_with_str(&text).unwrap_or_else(|e| {
            console_error!(format!("websocket send error: {:?}", e));
        });
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::ServerMessage(msg) => {
                console_debug!(format!("Server message: {:?}", msg));
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            }
            Msg::Connected => {
                console_log!("Connected web socket!");
                self.state = ApiState::Connected;
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, Message::Connected);
                }
            }
            Msg::ConnectionLost => {
                console_log!("Lost connection on web socket!");
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
        console_log!("destroying API service");
    }
}
