use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use im_rc::Vector;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement, KeyboardEvent};
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

use tr::tr;

#[derive(PartialEq)]
pub enum ChatLineData {
    Connected,
    Disconnected,
    Text(String),
}

pub enum Msg {
    Close,
    Ignore,
    SendChat,
    SetChatLine(String),
}

#[derive(PartialEq)]
pub struct ChatLine {
    pub nickname: String,
    pub data: ChatLineData,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub log: Vector<Rc<ChatLine>>,
    pub on_send_chat: Callback<String>,
    pub on_close: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.log == other.log
            && self.on_send_chat == other.on_send_chat
            && self.on_close == other.on_close
    }
}

pub struct ChatBox {
    log: Vector<Rc<ChatLine>>,
    log_ref: NodeRef,
    chat_line: Cow<'static, str>,
    on_send_chat: Callback<String>,
    on_close: Callback<()>,
}

impl ChatLine {
    pub fn text(&self) -> &str {
        match self.data {
            ChatLineData::Connected => "*connected*",
            ChatLineData::Disconnected => "*disconnected*",
            ChatLineData::Text(ref x) => x.as_str(),
        }
    }

    pub fn render(&self) -> String {
        format!("<{}> {}", self.nickname, self.text())
    }
}

impl Component for ChatBox {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ChatBox {
            log: ctx.props().log.clone(),
            log_ref: NodeRef::default(),
            chat_line: "".into(),
            on_send_chat: ctx.props().on_send_chat.clone(),
            on_close: ctx.props().on_close.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let Some(div) = self.log_ref.cast::<Element>() {
            div.set_scroll_top(div.scroll_height());
        }

        match msg {
            Msg::SendChat => {
                let text = mem::replace(&mut self.chat_line, "".into());
                self.on_send_chat.emit(text.into());
            }
            Msg::Close => {
                self.on_close.emit(());
            }
            Msg::SetChatLine(text) => {
                self.chat_line = text.into();
            }
            Msg::Ignore => ()
        };

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if self.log != ctx.props().log {
            self.log = ctx.props().log.clone();
            true
        } else {
            false
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_placeholder_text = tr!("send some text");
        html! {
            <aside class="chat box">
                <h2>
                    <span class="box-title">{"Chat"}</span>
                    <button class="btn-link box-close" onclick={ctx.link().callback(|_| Msg::Close)}>{"X"}</button>
                </h2>
                <div class="chat-messages">
                    <ul id="chat-log" ref={self.log_ref.clone()}>
                    {
                        self.log.iter().map(|item| html! {
                            <li>{item.render()}</li>
                        }).collect::<Html>()
                    }
                    </ul>
                </div>
                <div class="toolbar">
                <input value={self.chat_line.to_string()} placeholder={input_placeholder_text} size="30"
                       onkeypress={ctx.link().callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::SendChat
                            } else {
                                Msg::Ignore
                            }
                        })}
                       oninput={ctx.link().callback(|e: web_sys::InputEvent| {
                            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
                            Msg::SetChatLine(input.value())
                        })} />
               </div>
            </aside>
        }
    }
}
