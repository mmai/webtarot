use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use yew::Callback;
use weblog::*;
use gloo_storage::{LocalStorage, Storage};

use crate::protocol::{Command, Message};
use crate::gprotocol::{PlayerInfo, GameInfo};

fn get_websocket_location() -> String {
    let player_info: Option<PlayerInfo> = LocalStorage::get("webtarot.self").ok();
    let game_info: Option<GameInfo> = LocalStorage::get("webtarot.game").ok();

    let location = web_sys::window().unwrap().location();
    format!(
        "{}://{}/ws/{}_{}",
        if location.protocol().unwrap() == "https:" { "wss" } else { "ws" },
        location.host().unwrap(),
        game_info.map(|i| i.game_id.to_string()).unwrap_or_else(|| "new".into()),
        player_info.map(|i| i.id.to_string()).unwrap_or_else(|| "new".into()),
    )
}

struct ApiState {
    ws: Option<web_sys::WebSocket>,
    subscribers: HashMap<usize, Callback<Message>>,
    next_id: usize,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_open: Option<Closure<dyn FnMut()>>,
    _on_close: Option<Closure<dyn FnMut()>>,
}

impl ApiState {
    fn new() -> Self {
        Self {
            ws: None,
            subscribers: HashMap::new(),
            next_id: 0,
            _on_message: None,
            _on_open: None,
            _on_close: None,
        }
    }
}

thread_local! {
    static GLOBAL_API: Rc<RefCell<ApiState>> = Rc::new(RefCell::new(ApiState::new()));
}

fn ensure_connected() {
    let already_connected = GLOBAL_API.with(|api| api.borrow().ws.is_some());
    if already_connected {
        return;
    }

    GLOBAL_API.with(|api| {
        let api_rc = api.clone();
        let mut state = api.borrow_mut();

        console_log!("Connecting to server");
        let ws_url = get_websocket_location();
        let ws = web_sys::WebSocket::new(&ws_url).unwrap();

        let api_msg = api_rc.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(text) = e.data().as_string() {
                let state = api_msg.borrow();
                match serde_json::from_str::<Message>(&text) {
                    Ok(msg) => {
                        for cb in state.subscribers.values() {
                            cb.emit(msg.clone());
                        }
                    }
                    Err(err) => {
                        let err_str = format!("websocket parse error: {:?}", err);
                        console_error!(err_str);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        let api_open = api_rc.clone();
        let on_open = Closure::wrap(Box::new(move || {
            console_log!("Connected web socket!");
            let state = api_open.borrow();
            for cb in state.subscribers.values() {
                cb.emit(Message::Connected);
            }
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

        let on_close = Closure::wrap(Box::new(move || {
            console_log!("Lost connection on web socket!");
        }) as Box<dyn FnMut()>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        state.ws = Some(ws);
        state._on_message = Some(on_message);
        state._on_open = Some(on_open);
        state._on_close = Some(on_close);
    });
}

/// Handle to the shared WebSocket connection. Unsubscribes on drop.
pub struct ApiBridge {
    id: usize,
}

impl ApiBridge {
    pub fn send(&mut self, cmd: Command) {
        GLOBAL_API.with(|api| {
            let state = api.borrow();
            if let Some(ws) = &state.ws {
                let text = serde_json::to_string(&cmd).expect("serialize command");
                let dbg_str = format!("Sending command: {:?}", &cmd);
                console_debug!(dbg_str);
                ws.send_with_str(&text).unwrap_or_else(|e| {
                    let err_str = format!("websocket send error: {:?}", e);
                    console_error!(err_str);
                });
            }
        });
    }
}

impl Drop for ApiBridge {
    fn drop(&mut self) {
        GLOBAL_API.with(|api| {
            api.borrow_mut().subscribers.remove(&self.id);
        });
    }
}

pub struct Api;

impl Api {
    /// Subscribe to WebSocket messages. Opens the connection on first call.
    pub fn bridge(callback: Callback<Message>) -> ApiBridge {
        ensure_connected();
        GLOBAL_API.with(|api| {
            let mut state = api.borrow_mut();
            let id = state.next_id;
            state.next_id += 1;
            state.subscribers.insert(id, callback);
            ApiBridge { id }
        })
    }
}
