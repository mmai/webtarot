use std::rc::Rc;
use std::time::Duration;
use std::f32;
use im_rc::Vector;
use uuid::Uuid;
use web_sys::HtmlAudioElement;
use yew::agent::Bridged;
use yew::services::{IntervalService, Task};
use yew::{
    html, Bridge, Component, ComponentLink, Html, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::components::chat_box::{ChatBox, ChatLine, ChatLineData};
use crate::components::player_list::PlayerList;
use crate::components::bidding_actions::BiddingActions;
use crate::components::call_king_action::CallKingAction;
use crate::components::scores::Scores;
use crate::protocol::{
    Command, GameInfo, GamePlayerState, GameStateSnapshot, Message, PlayerAction,
    PlayerInfo,
    SendTextCommand,
    BidCommand, PlayCommand, CallKingCommand, MakeDogCommand,
    Turn,
    PlayEvent,
};
use tarotgame::{bid, cards};
use crate::utils::format_join_code;
use crate::sound_player::SoundPlayer;

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub game_info: GameInfo,
}

pub struct GamePage {
    keepalive_job: Box<dyn Task>,
    link: ComponentLink<GamePage>,
    api: Box<dyn Bridge<Api>>,
    game_info: GameInfo,
    player_info: PlayerInfo,
    game_state: Rc<GameStateSnapshot>,
    chat_log: Vector<Rc<ChatLine>>,
    dog: cards::Hand,
    hand: cards::Hand,
    is_waiting: bool,
    sound_player: SoundPlayer,
    error: Option<String>,
}

pub enum Msg {
    Ping,
    Disconnect,
    MarkReady,
    Continue,
    CloseError,
    Bid(bid::Target),
    Pass,
    Play(cards::Card),
    CallKing(cards::Card),
    SetChatLine(String),
    MakeDog,
    AddToDog(cards::Card),
    AddToHand(cards::Card),
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
        let mut interval = IntervalService::new();
        // Ping server every 50s in order to keep alive the websocket 
        let keepalive = interval.spawn(
            Duration::from_secs(50), 
            link.callback(|_| Msg::Ping).into()
            );

        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        let sound_paths = vec![
            ("chat".into(), "sounds/misc_menu.ogg"),
            ("card".into(), "sounds/cardPlace4.ogg"),
            ("error".into(), "sounds/negative_2.ogg"),
        ].into_iter().collect();

        GamePage {
            keepalive_job: Box::new(keepalive),
            link,
            api,
            game_info: props.game_info,
            chat_log: Vector::unit(Rc::new(ChatLine {
                nickname: props.player_info.nickname.clone(),
                data: ChatLineData::Connected,
            })),
            game_state: Rc::new(GameStateSnapshot::default()),
            player_info: props.player_info,
            dog: cards::Hand::new(),
            hand: cards::Hand::new(),
            is_waiting: false,
            sound_player: SoundPlayer::new(sound_paths),
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Chat(msg) => {
                    self.sound_player.play("chat".into());
                    self.add_chat_message(msg.player_id, ChatLineData::Text(msg.text));
                }
                Message::PlayEvent(evt) => {
                    self.sound_player.play("card".into());
                    let PlayEvent::Play(uuid, card) = evt;
                    self.add_chat_message(uuid, ChatLineData::Text(format!("play: {}", card.to_string())));
                    log!("play event {:?}", evt);
                }
                Message::Error(e) => {
                    self.is_waiting = false;
                    self.error = Some(e.message().into());
                    self.sound_player.play("error".into());
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
                    self.is_waiting = false;
                    self.game_state = Rc::new(snapshot);
                    self.dog = self.game_state.deal.initial_dog;
                    self.hand = self.game_state.deal.hand;
                }
                _ => {}
            },
            Msg::Ping => {
                self.api.send(Command::Ping);
                // log!("ping ?");
            }
            Msg::SetChatLine(text) => {
                self.api.send(Command::SendText(SendTextCommand { text }));
            }
            Msg::CloseError => {
                self.error = None;
            }
            Msg::Continue => {
                self.is_waiting = true;
                self.api.send(Command::Continue);
            }
            Msg::MarkReady => {
                self.is_waiting = true;
                self.api.send(Command::MarkReady);
            }
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
            }
            Msg::Bid(target) => {
                self.is_waiting = true;
                log!("received bid {:?}", target);
                self.api.send(Command::Bid(BidCommand { target }));
            }
            Msg::Pass => {
                self.is_waiting = true;
                self.api.send(Command::Pass);
            }
            Msg::CallKing(card) => {
                self.is_waiting = true;
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
                self.is_waiting = true;
                self.api.send(Command::MakeDog(MakeDogCommand { cards: self.dog }));
            },
            Msg::Play(card) => {
                self.is_waiting = true;
                self.api.send(Command::Play(PlayCommand { card }));
            }
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

        let mut game_classes = vec!["game"];
        if self.is_waiting {
            game_classes.push("waiting");
        }

        let is_my_turn = self.game_state.get_playing_pos() == Some(self.my_state().pos);
        // let is_my_turn = self.game_state.turn.has_player_pos() && self.game_state.deal.current == self.my_state().pos;
        let mut actions_classes = vec!["actions"];
        if is_my_turn {
            actions_classes.push("current-player");
        }

        let messageContent: Option<Html> = match self.game_state.turn {
               Turn::Intertrick => 
                   if !self.my_state().ready  { 
                       let winner_pos = self.game_state.deal.last_trick.winner;
                       let winner_name = self.game_state.pos_player_name(winner_pos);
                       Some(html! { 
                           <div class="results">
                               { "trick for " }
                               <strong>{ winner_name }</strong>
                           </div>

                       })
                   } else { None },
               Turn::Interdeal => 
                   if !self.my_state().ready  { 
                       let scores: Vec<Vec<f32>> = self.game_state.scores.iter().map(|score| score.to_vec()).collect();
                       let players: Vec<String> = self.game_state.players.iter().map(|pl| pl.player.nickname.clone()).collect();

                       let taker_won = self.game_state.deal.taker_diff > 0.0;
                       let diff_abs = f32::abs(self.game_state.deal.taker_diff);
                       let contract_message = if taker_won {
                           format!("Contract succeded by {} points", diff_abs)
                       } else {
                           format!("Contract failed by {} points", diff_abs)
                       };

                       let dog_message = format!("Dog : {}", self.game_state.deal.dog.to_string());

                       Some(html! {
                     <div>
                         <div> {{ contract_message }} </div>
                         <div> {{ dog_message }} </div>
                        <Scores players=players scores=scores />
                     </div>
                   })} else { None },
              _ => None
        };

        html! {
    <div class=game_classes>
      <header>
        <p class="turn-info">{format!("Turn: {} {}", self.game_state.current_player_name(), self.game_state.turn)}</p>
        {if let Some(contract) = &self.game_state.deal.contract {
             html! {<p class="deal-info">{format!("Contract: {}", contract.to_string())}</p>}
        } else {
             html! {}
        }}
      </header>

      <PlayerList game_state=self.game_state.clone() players=others/>

        { if let Some(error) = &self.error  { html! {
          <div class="notify-wrapper">
            <div class="error notify">
                <div>
                    {format!("Error: {}", error)}
                </div>
                <div class="toolbar">
                    <button class="btn-error" onclick=self.link.callback(|_| Msg::CloseError)>{"Ok"}</button>
                </div>
              </div>
            </div>
        }} else { html! {} }}

        { if let Some(message) = messageContent  { html! {
          <div class="notify-wrapper">
            <div class="notify wrapper">
                { message }

                <div class="toolbar">
                    <button class="primary" onclick=self.link.callback(|_| Msg::Continue)>{"Ok"}</button>
                </div>
            </div>
        </div>
        }} else { html! {} }}

        <section class=actions_classes>
            {match self.game_state.turn {
               Turn::Pregame => html! {
                <div class="wrapper">
                    <div class="toolbar">
                    {if !self.my_state().ready  {
                        html! {<button class="primary" onclick=self.link.callback(|_| Msg::MarkReady)>{"Ready!"}</button>}
                    } else {
                        html! {}
                    }}
                        <button class="cancel" onclick=self.link.callback(|_| Msg::Disconnect)>{"Disconnect"}</button>
                    </div>
                    <h1>{{ "join code:" }} <strong>{format!(" {}", format_join_code(&self.game_info.join_code))}</strong></h1>
                 </div>
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
               Turn::MakingDog => {
                   html! {
                       <div>
                           <section class="hand">
                           {
                               for self.dog.list().iter().map(|card| {
                                   let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                   if player_action == Some(PlayerAction::MakeDog) {
                                       let clicked = card.clone();
                                       html! {
                                           <div class="card" style={style} onclick=self.link.callback(move |_| Msg::AddToHand(clicked))></div>
                                       }
                                   } else {
                                       html! {
                                           <div class="card" style={style}></div>
                                       }
                                   }
                               })
                           }
                           </section>
                           { if player_action == Some(PlayerAction::MakeDog) {
                               html! {
                               <button onclick=self.link.callback(move |_| Msg::MakeDog)>
                               {{ "finish" }}
                               </button>
                             }} else {
                                 html!{}
                             }
                           }
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
                            } else if player_action == Some(PlayerAction::Play) {
                                html!{
                                    <div class="yourturn"> {{ "Your turn to play!" }} </div>
                            }} else {
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

        <ChatBox log=self.chat_log.clone()
                 on_send_chat=self.link.callback(|text| Msg::SetChatLine(text))
        />

    </div>

        }
    }
}
