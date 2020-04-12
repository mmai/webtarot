use std::mem;
use std::rc::Rc;

use im_rc::Vector;
use uuid::Uuid;
use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::components::chat_box::{ChatBox, ChatLine, ChatLineData};
use crate::components::player_list::PlayerList;
use crate::protocol::{
    Command, GameInfo, GamePlayerState, GameStateSnapshot, Message, PlayerAction,
    PlayerInfo, PlayerRole, RevealCardCommand, SendTextCommand, SetPlayerRoleCommand,
    Tile, Turn,
};
use crate::utils::format_join_code;

#[derive(Clone, Debug)]
pub enum GamePageCommand {
    Quit,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub game_info: GameInfo,
    pub on_game_command: Callback<GamePageCommand>,
}

pub struct GamePage {
    link: ComponentLink<GamePage>,
    api: Box<dyn Bridge<Api>>,
    game_info: GameInfo,
    player_info: PlayerInfo,
    game_state: Rc<GameStateSnapshot>,
    chat_line: String,
    chat_log: Vector<Rc<ChatLine>>,
    on_game_command: Callback<GamePageCommand>,
}

pub enum Msg {
    Ignore,
    SendChat,
    Disconnect,
    MarkReady,
    SetChatLine(String),
    ServerMessage(Message),
    SetRole(PlayerRole),
    Reveal(usize),
}

impl GamePage {
    pub fn add_chat_message(&mut self, player_id: Uuid, data: ChatLineData) {
        let nickname = self
            .game_state
            .players
            .iter()
            .find(|x| x.player.id == player_id)
            .map(|x| x.player.nickname.as_str())
            .unwrap_or("anonymous")
            .to_string();
        self.chat_log
            .push_back(Rc::new(ChatLine { nickname, data }));
        while self.chat_log.len() > 100 {
            self.chat_log.pop_front();
        }
    }

    pub fn my_state(&self) -> &GamePlayerState {
        self.game_state
            .players
            .iter()
            .find(|state| state.player.id == self.player_info.id)
            .unwrap()
    }
}

fn get_tile_class(tile: &Tile, can_guess: bool) -> String {
    let mut rv = "tile ".to_string();
    if can_guess {
        rv.push_str(" can-guess");
    }
    rv
}

impl Component for GamePage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        GamePage {
            link,
            api,
            game_info: props.game_info,
            chat_line: "".into(),
            chat_log: Vector::unit(Rc::new(ChatLine {
                nickname: props.player_info.nickname.clone(),
                data: ChatLineData::Connected,
            })),
            game_state: Rc::new(GameStateSnapshot::default()),
            player_info: props.player_info,
            on_game_command: props.on_game_command,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Chat(msg) => {
                    self.add_chat_message(msg.player_id, ChatLineData::Text(msg.text));
                }
                Message::PlayerConnected(state) => {
                    let player_id = state.player.id;
                    let game_state = Rc::make_mut(&mut self.game_state);
                    game_state.players.push(state);
                    self.add_chat_message(player_id, ChatLineData::Connected);
                }
                Message::PlayerDisconnected(msg) => {
                    self.add_chat_message(msg.player_id, ChatLineData::Disconnected);
                    let game_state = Rc::make_mut(&mut self.game_state);
                    game_state.players.retain(|x| x.player.id != msg.player_id);
                }
                Message::GameStateSnapshot(snapshot) => {
                    self.game_state = Rc::new(snapshot);
                }
                _ => {}
            },
            Msg::SendChat => {
                let text = mem::replace(&mut self.chat_line, "".into());
                self.api.send(Command::SendText(SendTextCommand { text }));
            }
            Msg::SetChatLine(text) => {
                self.chat_line = text;
            }
            Msg::SetRole(role) => {
                self.api
                    .send(Command::SetPlayerRole(SetPlayerRoleCommand { role }));
            }
            Msg::MarkReady => {
                self.api.send(Command::MarkReady);
            }
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
            }
            Msg::Reveal(index) => {
                if self.my_state().get_turn_player_action(self.game_state.turn)
                    == Some(PlayerAction::Bid)
                {
                    self.api
                        .send(Command::RevealCard(RevealCardCommand { index }));
                }
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        if self.game_state.players.is_empty() {
            return html! {};
        }

        let state = self.my_state();

        let role = state.role;
        let role_button = |new_role: PlayerRole, title: &str| -> Html {
            html! {
                <button
                    disabled=role == new_role
                    onclick=self.link.callback(move |_| Msg::SetRole(new_role))>
                    {title}
                </button>
            }
        };

        let player_action = state.get_turn_player_action(self.game_state.turn);

        html! {
            <div>
                <p class="turn-info">{format!("Turn: {}", self.game_state.turn)}</p>
                <h1>{format!("Game ({})", format_join_code(&self.game_info.join_code))}</h1>
                <PlayerList game_state=self.game_state.clone()/>
                <section class="hand">
                {
                    for self.game_state.deal.hand.list().iter().map(| card| {
                        let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                        html! {
                            <div class="card" style={style}></div>
                        }
                    })
                }
                </section>
                <ChatBox log=self.chat_log.clone()/>
                <div class="toolbar">
                    <span>{format!("{:?} ", &self.my_state().pos)}</span>
                    <span>{format!("{}: ", &self.player_info.nickname)}</span>
                    <input value=&self.chat_line
                        placeholder="send some text"
                        size="30"
                        onkeypress=self.link.callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::SendChat
                            } else {
                                Msg::Ignore
                            }
                        })
                        oninput=self.link.callback(|e: InputData| Msg::SetChatLine(e.value)) />
                    {if player_action == Some(PlayerAction::Bid) {
                        html! {
                            <>
                                <button class="primary" onclick=self.link.callback(|_| Msg::SendChat)>{"Bid"}</button>
                                <button onclick=self.link.callback(|_| Msg::SendChat)>{"Chat"}</button>
                            </>
                        }
                    } else {
                        html! {
                            <button class="primary" onclick=self.link.callback(|_| Msg::SendChat)>{"Chat"}</button>
                        }
                    }}
                </div>
                {if self.game_state.turn == Turn::Pregame {
                    html! {
                        <div class="toolbar">
                        {if !self.my_state().ready  {
                            html! {<button class="primary" onclick=self.link.callback(|_| Msg::MarkReady)>{"Ready!"}</button>}
                        } else {
                            html! {}
                        }}
                            <button class="cancel" onclick=self.link.callback(|_| Msg::Disconnect)>{"Disconnect"}</button>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}
