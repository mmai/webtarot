use std::rc::Rc;

use std::mem;
use im_rc::Vector;
use web_sys::Element;
use yew::{html, Component, ComponentLink, Html, InputData, KeyboardEvent, NodeRef, Properties, ShouldRender, Callback};

use tr::tr;

#[derive(PartialEq)]
pub enum ChatLineData {
    Connected,
    Disconnected,
    Text(String),
}

pub enum Msg {
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
}

pub struct ChatBox {
    log: Vector<Rc<ChatLine>>,
    link: ComponentLink<ChatBox>,
    log_ref: NodeRef,
    chat_line: String,
    on_send_chat: Callback<String>,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        ChatBox {
            log: props.log,
            link,
            log_ref: NodeRef::default(),
            chat_line: "".into(),
            on_send_chat: props.on_send_chat,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(div) = self.log_ref.cast::<Element>() {
            div.set_scroll_top(div.scroll_height());
        }

        match msg {
            Msg::SendChat => {
                let text = mem::replace(&mut self.chat_line, "".into());
                self.on_send_chat.emit(text);
            }
            Msg::SetChatLine(text) => {
                self.chat_line = text;
            }
            Msg::Ignore => ()
        };

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.log != props.log {
            self.log = props.log;
            // self.link.send_message(());
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let input_placeholder_text = tr!("send some text" );
        html! {
            <aside class="chat box">
                <h2>{"Chat"}</h2>
                <div class="chat-messages">
                    <ul id="chat-log" ref=self.log_ref.clone()>
                    {
                        for self.log.iter().map(|item| html! {
                            <li>{item.render()}</li>
                        })
                    }
                    </ul>
                </div>
                <div class="toolbar">
                <input value=&self.chat_line placeholder=input_placeholder_text size="30"
                       onkeypress=self.link.callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::SendChat
                            } else {
                                Msg::Ignore
                            }
                        })
                       oninput=self.link.callback(|e: InputData| Msg::SetChatLine(e.value)) />
               </div>
            </aside>
        }
    }
}
