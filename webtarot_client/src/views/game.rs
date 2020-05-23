use std::mem;
use std::rc::Rc;
use im_rc::Vector;
use uuid::Uuid;
use yew::agent::Bridged;
use yew::{
    html, Bridge, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::components::chat_box::{ChatBox, ChatLine, ChatLineData};
use crate::components::player_list::PlayerList;
use crate::components::bidding_actions::BiddingActions;
use crate::components::call_king_action::CallKingAction;
use crate::protocol::{
    Command, GameInfo, GamePlayerState, GameStateSnapshot, Message, PlayerAction,
    PlayerInfo,
    SendTextCommand,
    BidCommand, PlayCommand, CallKingCommand, MakeDogCommand,
    Turn,
};
use tarotgame::{bid, cards};
use crate::utils::format_join_code;

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub game_info: GameInfo,
}

pub struct GamePage {
    link: ComponentLink<GamePage>,
    api: Box<dyn Bridge<Api>>,
    game_info: GameInfo,
    player_info: PlayerInfo,
    game_state: Rc<GameStateSnapshot>,
    chat_line: String,
    chat_log: Vector<Rc<ChatLine>>,
    dog: cards::Hand,
    hand: cards::Hand,
}

pub enum Msg {
    Ignore,
    SendChat,
    Disconnect,
    MarkReady,
    Continue,
    Bid(bid::Target),
    Pass,
    Play(cards::Card),
    CallKing(cards::Card),
    MakeDog,
    AddToDog(cards::Card),
    AddToHand(cards::Card),
    SetChatLine(String),
    ServerMessage(Message),
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
            dog: cards::Hand::new(),
            hand: cards::Hand::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Chat(msg) => {
                    self.add_chat_message(msg.player_id, ChatLineData::Text(msg.text));
                }
                Message::Error(e) => {
                    log!("error from server {:?}", e);
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
                    self.dog = self.game_state.deal.initial_dog;
                    self.hand = self.game_state.deal.hand;
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
            Msg::Continue => {
                self.api.send(Command::Continue);
            }
            Msg::MarkReady => {
                self.api.send(Command::MarkReady);
            }
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
            }
            Msg::Bid(target) => {
                log!("received bid {:?}", target);
                self.api.send(Command::Bid(BidCommand { target }));
            }
            Msg::Pass => {
                self.api.send(Command::Pass);
            }
            Msg::CallKing(card) => {
                self.api.send(Command::CallKing(CallKingCommand { card }));
            }
            Msg::AddToHand(card) => {
                self.hand.add(card);
                self.dog.remove(card);
            },
            Msg::AddToDog(card) => {
                self.dog.add(card);
                self.hand.remove(card);
            },
            Msg::MakeDog => {
                self.api.send(Command::MakeDog(MakeDogCommand { cards: self.dog }));
            },
            Msg::Play(card) => {
                self.api.send(Command::Play(PlayCommand { card }));
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        if self.game_state.players.is_empty() {
            return html! {};
        }

        let my_state = self.my_state();
        let card_played = self.game_state.deal.last_trick.card_played(my_state.pos);
        let player_action = my_state.get_turn_player_action(self.game_state.turn);

        // display players in order of playing starting from the current player
        let mut others_before = vec![];
        let mut others = vec![];
        let mypos = my_state.pos.to_n();

        // let mut positioned = Vec::from_iter(self.game_state.players.clone());
        // positioned.sort_by(|a, b| a.pos.to_n().cmp(&b.pos.to_n()));
        // for pstate in positioned.iter() {
        for pstate in self.game_state.players.iter() {
            let pos = pstate.pos.to_n();
            if pos < mypos {
                others_before.push(pstate.clone());
            } else if mypos < pos{
               others.push(pstate.clone());
            }
        }

        // log!("others: {:?} others_before: {:?}", others, others_before);
        others.append(&mut others_before);

        let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        // let is_my_turn = self.game_state.turn.has_player_pos() && self.game_state.deal.current == self.my_state().pos;
        let mut actions_classes = vec!["actions"];
        if is_my_turn {
            actions_classes.push("current-player");
        }

        html! {
    <div class="game">
      <header>
        <p class="turn-info">{format!("Turn: {}", self.game_state.turn)}</p>
        {if let Some(contract) = &self.game_state.deal.contract {
             html! {<p class="deal-info">{format!("Contract: {}", contract.to_string())}</p>}
        } else {
             html! {}
        }}
        <h1>{format!("Game ({})", format_join_code(&self.game_info.join_code))}</h1>
      </header>

      <PlayerList game_state=self.game_state.clone() players=others/>

        <section class=actions_classes>
            {match self.game_state.turn {
               Turn::Pregame => html! {
                    <div class="toolbar">
                    {if !self.my_state().ready  {
                        html! {<button class="primary" onclick=self.link.callback(|_| Msg::MarkReady)>{"Ready!"}</button>}
                    } else {
                        html! {}
                    }}
                        <button class="cancel" onclick=self.link.callback(|_| Msg::Disconnect)>{"Disconnect"}</button>
                    </div>
                },
               Turn::Intertrick => 
                   if !self.my_state().ready  { html! {
                       <div>
                           <div class="results">
                               {format!("trick for : {:?}", self.game_state.deal.last_trick.winner)}
                           </div>
                           <div class="toolbar">
                               <button class="primary" onclick=self.link.callback(|_| Msg::Continue)>{"Ok"}</button>
                           </div>
                       </div>
                   }} else {
                       html! {}
                },
               Turn::Interdeal => 
                   if !self.my_state().ready  { html! {
                     <div>
                        <div class="results">
                            <pre>
                                {format!("historique : {:?}", self.game_state.scores)}
                            </pre>
                            <strong>
                                <pre>
                                    {format!("scores : {:?}", self.game_state.deal.scores)}
                                </pre>
                            </strong>
                        </div>
                        <div class="toolbar">
                            <button class="primary" onclick=self.link.callback(|_| Msg::Continue)>{"Ok"}</button>
                        </div>
                     </div>
                   }} else {
                     html! {}
                },
               Turn::CallingKing if player_action == Some(PlayerAction::CallKing) => {
                   // Choose a queen if player has all kings
                   // Choose a jack if player has all kings and all queens
                   let my_hand = self.game_state.deal.hand;
                   let rank = if my_hand.has_all_rank(cards::Rank::RankK) {
                       if my_hand.has_all_rank(cards::Rank::RankQ) { cards::Rank::RankJ } else { cards::Rank::RankQ } 
                   } else {
                       cards::Rank::RankK
                   };

                    html! {
                        <CallKingAction
                            rank=rank
                            on_call_king=self.link.callback(|card| Msg::CallKing(card))
                            />
                    }
               },
               Turn::MakingDog if player_action == Some(PlayerAction::MakeDog) => {
                   html! {
                       <div>
                           <section class="hand">
                           {
                               for self.dog.list().iter().map(|card| {
                                   let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                   let clicked = card.clone();
                                   html! {
                                       <div class="card" style={style} onclick=self.link.callback(move |_| Msg::AddToHand(clicked))></div>
                                   }
                               })
                           }
                           </section>
                           <button onclick=self.link.callback(move |_| Msg::MakeDog)>
                           {{ "finish" }}
                           </button>
                       </div>
                   }
               },
                _ if player_action == Some(PlayerAction::Bid) => 
                    html! {
                        <BiddingActions
                            game_state=self.game_state.clone()
                            on_bid=self.link.callback(|contract| Msg::Bid(contract))
                            on_pass=self.link.callback(|contract| Msg::Pass)
                            />
                    },
                _ => 
                    html! {
                        <div>
                            {if let Some(card) = card_played {
                                let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                html! {
                                    <div class="card" style={style}></div>
                                }
                            } else {
                                html!{}
                            }}
                        </div>
                    }
             }}
        </section>

        <section class="hand">
        { if self.game_state.turn != Turn::Pregame && self.game_state.turn != Turn::Interdeal {
            html! {
              for self.hand.list().iter().map(|card| {
                let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                let clicked = card.clone();
                html! {
                    <div class="card" style={style} 
                    onclick=self.link.callback(move |_| 
                        if player_action == Some(PlayerAction::MakeDog) {
                            Msg::AddToDog(clicked)
                        } else {
                            Msg::Play(clicked)
                        }) >
                    </div>
                }
            })
        }} else {
            html!{}
        }}
        </section>

        <ChatBox log=self.chat_log.clone()/>

        <div class="footer">
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
                    <button class="primary" onclick=self.link.callback(|_| Msg::SendChat)>{"Chat"}</button>
            </div>
        </div>
    </div>

        }
    }
}
