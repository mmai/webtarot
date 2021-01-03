use std::rc::Rc;
use std::time::Duration;
use std::f32;
use im_rc::Vector;
use uuid::Uuid;
use yew::agent::Bridged;
use yew::services::{IntervalService, Task};
use yew::{
    html, Bridge, Component, ComponentLink, Html, Properties,
    ShouldRender,
};
use tr::tr;
use weblog::*;

use crate::api::Api;
use crate::components::chat_box::{ChatBox, ChatLine, ChatLineData};
use crate::components::player_list::PlayerList;
use crate::components::bidding_actions::BiddingActions;
use crate::components::call_king_action::CallKingAction;
use crate::components::announces::Announces;
use crate::components::scores::Scores;
use crate::gprotocol::{GameInfo, PlayerInfo, SendTextCommand};
    
use crate::protocol::{
    Command, GamePlayerState, GameStateSnapshot, Message, PlayerAction,
    GamePlayCommand,
    BidCommand, PlayCommand, CallKingCommand, MakeDogCommand, AnnounceCommand,
    Turn,
    PlayEvent,
};
use tarotgame::{bid, cards, deal_size, Announce};
use crate::utils::format_join_code;
use crate::sound_player::SoundPlayer;

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub game_info: GameInfo,
}

pub struct GamePage {
    #[allow(dead_code)]
    keepalive_job: Box<dyn Task>,
    link: ComponentLink<GamePage>,
    api: Box<dyn Bridge<Api>>,
    game_info: GameInfo,
    player_info: PlayerInfo,
    game_state: Rc<GameStateSnapshot>,
    next_game_messages: Vector<Message>,
    chat_log: Vector<Rc<ChatLine>>,
    dog: cards::Hand,
    hand: cards::Hand,
    is_waiting: bool,
    update_needs_confirm: bool,
    sound_player: SoundPlayer,
    error: Option<String>,
    slam_selected: bool,
    overlay_box: Option<Html>,
}

pub enum Msg {
    Ping,
    Disconnect,
    MarkReady,
    Continue,
    CloseError,
    Bid((bid::Target, bool)),
    Pass,
    Play(cards::Card),
    CallKing(cards::Card),
    SetChatLine(String),
    MakeDog,
    ToggleSlam,
    AddToDog(cards::Card),
    AddToHand(cards::Card),
    ServerMessage(Message),
    Announce(Announce),
}

impl GamePage {
    pub fn add_chat_message(&mut self, player_id: Uuid, data: ChatLineData) {
        let nickname = self.get_nickname(player_id);
        self.chat_log
            .push_back(Rc::new(ChatLine { nickname, data }));
        while self.chat_log.len() > 100 {
            self.chat_log.pop_front();
        }
    }

    fn get_nickname(&self, player_id: Uuid) -> String {
        self
            .game_state
            .players
            .iter()
            .find(|x| x.player.id == player_id)
            .map(|x| x.player.nickname.as_str())
            .unwrap_or("anonymous")
            .to_string()
    }

    pub fn my_state(&self) -> &GamePlayerState {
        self.game_state
            .players
            .iter()
            .find(|state| state.player.id == self.player_info.id)
            .unwrap()
    }

    fn apply_snapshot(&mut self, snapshot: GameStateSnapshot){
        self.game_state = Rc::new(snapshot);
        self.dog = self.game_state.deal.initial_dog;
        self.hand = self.game_state.deal.hand;
    }

    fn is_first_trick(&self) -> bool {
        self.game_state.deal.trick_count == 1
    }

    fn display_overlay_box(&self) -> Html {
        let output;
        if let Some(message) = &self.overlay_box  { 
            output = message.clone();
            output
        } else { html!{} }
    }

    fn manage_game_message(&mut self, msg: Message){
        if self.update_needs_confirm {
            let msg_test = msg.clone();
            if let Message::GameStateSnapshot(snapshot) = msg_test {
            }

            self.next_game_messages.push_front(msg);
        } else {
            match msg {
                Message::GameStateSnapshot(snapshot) => {
                    self.is_waiting = false;
                    self.apply_snapshot(snapshot);
                }
                Message::PlayEvent(evt) => {
                    self.sound_player.play("card".into());
                    match evt {
                        PlayEvent::Play(uuid, card) => {
                            self.add_chat_message(uuid, ChatLineData::Text(format!("play: {}", card.to_string())));
                        }
                        PlayEvent::EndTrick => {
                            let winner_pos = self.game_state.deal.last_trick.winner;
                            let winner_name = self.game_state.pos_player_name(winner_pos);
                            self.overlay_box = Some(html! {
                                <div class="results">
                                { tr!("trick for ") }
                                <strong>{ winner_name }</strong>
                                    </div>
                            });
                            self.update_needs_confirm = true;
                        },
                        PlayEvent::EndDeal => {
                            let scores: Vec<Vec<f32>> = self.game_state.scores.iter().map(|score| score.to_vec()).collect();
                            let players: Vec<String> = self.game_state.players.iter().map(|pl| pl.player.nickname.clone()).collect();

                            let taker_won = self.game_state.deal.taker_diff >= 0.0;
                            let diff_abs = f32::abs(self.game_state.deal.taker_diff);
                            let contract_message = if taker_won {
                                tr!("Contract succeded by {0} points", diff_abs)
                            } else {
                                tr!("Contract failed by {0} points", diff_abs)
                            };

                            let dog_message = tr!("Dog : {0}", self.game_state.deal.dog.to_string());
                            self.overlay_box = Some(html! {
                                <div>
                                    <div> {{ contract_message }} </div>
                                    <div> {{ dog_message }} </div>
                                    <Scores players=players scores=scores />
                                    </div>
                            });
                            self.update_needs_confirm = true;
                        },
                        PlayEvent::Announce(uuid, announce) => {
                            self.add_chat_message(uuid, ChatLineData::Text(format!("announce: {:?}", announce.proof.map(|h| h.to_string()))));
                            let nickname = self.get_nickname(uuid);
                            let proof_html = if let Some(proof_hand) = announce.proof {
                                html!{
                                    <div class="hand">
                                    { for proof_hand.list().iter().map(|card| {
                                                                                  let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                                                                                  html! {
                                                                                      <div class="card" style={style}></div>
                                                                                  }
                                                                              })}
                                    </div>
                                }
                            } else { html!() };
                            self.overlay_box = Some(html! {
                                <div>
                                    <div> { tr!("{} announced a {}", nickname, announce.atype) } </div>
                                    { proof_html }
                                </div>
                            });
                            self.update_needs_confirm = true;
                        },
                        _ => {
                            self.overlay_box = Some(html! {
                                <div> { "Unknown event" } </div>
                            });

                        }
                    }
                }
                _ => { } // unknown server messages
            }
        }
    }

}

impl Component for GamePage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Ping server every 50s in order to keep alive the websocket 
        let keepalive = IntervalService::spawn(
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
            slam_selected: false,
            next_game_messages: Vector::new(),
            update_needs_confirm: false,
            overlay_box: None,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Pong => {}
                Message::Error(e) => {
                    self.is_waiting = false;
                    self.error = Some(e.message().into());
                    self.sound_player.play("error".into());
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
                Message::Chat(msg) => {
                    self.sound_player.play("chat".into());
                    self.add_chat_message(msg.player_id, ChatLineData::Text(msg.text));
                }
                _ => self.manage_game_message(message)
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
                self.overlay_box = None;
                self.update_needs_confirm = false;
                while !self.next_game_messages.is_empty() && !self.update_needs_confirm {
                    let next_msg = self.next_game_messages.pop_back();
                    self.manage_game_message(next_msg.unwrap());
                }
            }
            Msg::MarkReady => {
                self.is_waiting = true;
                self.api.send(Command::MarkReady);
            }
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
            }
            Msg::Bid((target, slam)) => {
                self.is_waiting = true;
                self.slam_selected = slam;
                self.api.send(Command::GamePlay(GamePlayCommand::Bid(BidCommand { target, slam })));
            }
            Msg::Pass => {
                self.is_waiting = true;
                self.api.send(Command::GamePlay(GamePlayCommand::Pass));
            }
            Msg::CallKing(card) => {
                self.is_waiting = true;
                self.api.send(Command::GamePlay(GamePlayCommand::CallKing(CallKingCommand { card })));
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
                self.api.send(Command::GamePlay(GamePlayCommand::MakeDog(MakeDogCommand { cards: self.dog, slam: self.slam_selected })));
            },
            Msg::ToggleSlam => {
                self.slam_selected = !self.slam_selected;
            },
            Msg::Announce(announce) => {
                self.api.send(Command::GamePlay(GamePlayCommand::Announce(AnnounceCommand { announce })));
            }
            Msg::Play(card) => {
                self.is_waiting = true;
                self.api.send(Command::GamePlay(GamePlayCommand::Play(PlayCommand { card })));
            }
        }
        // self.refresh_overlay_box();
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


        // log::debug!("after message content");
        let player = self.game_state.current_player_name();
        let turn_info = match self.game_state.turn {
            Turn::Pregame => tr!("pre-game"),
            Turn::Intertrick => tr!("inter trick"),
            Turn::Interdeal => tr!("inter deal"),
            Turn::Bidding(_) => tr!("{0} bidding", player),
            Turn::Playing(_) => tr!("{0} playing", player),
            Turn::Endgame => tr!("end"),
            Turn::CallingKing => tr!("calling king"),
            Turn::MakingDog => tr!("making dog"),
        };

        html! {
    <div class=game_classes>
      <header>
        <p class="turn-info">{turn_info}</p>
        {if let Some(contract) = &self.game_state.deal.contract {
             let dog_info = if self.game_state.deal.dog.is_empty() {
                 "".to_string()
             } else {
                 self.game_state.deal.dog.to_string()
             };
             let king_info = if let Some(king) = &self.game_state.deal.king {
                format!(" ({})", king.to_string())
             } else { "".into() };
             html! {<p class="deal-info">{format!("{} {} {}", contract.to_string(), king_info, dog_info)}</p>}
        } else {
             html! {}
        }}
      </header>

      <PlayerList game_state=self.game_state.clone() players=others/>

        { if let Some(error) = &self.error  { 
            let error_str = match error.as_str() {
            "bid: auctions are closed" => tr!("auctions are closed"),
            "bid: invalid turn order" => tr!("invalid turn order"),
            "bid: bid must be higher than current contract" => tr!("bid must be higher than current contract"),
            "bid: the auction are still running" => tr!("the auction are still running"),
            "bid: no contract was offered" => tr!("no contract was offered"),
            "play: invalid turn order" => tr!("invalid turn order"),
            "play: you can only play cards you have" => tr!("you can only play cards you have" ),
            "play: wrong suit played" => tr!("wrong suit played" ),
            "play: you must use trumps" => tr!("you must use trumps" ),
            "play: too weak trump played" => tr!("too weak trump played" ),
            "play: you cannot play the suit of the called king in the first trick" => tr!("you cannot play the suit of the called king in the first trick" ),
            "play: no trick has been played yet" => tr!("no trick has been played yet" ),
            "play: you are not the taker" => tr!("you are not the taker"),
            "play: Wrong number of cards" => tr!("Wrong number of cards"),
            "play: Can't put the same card twice in the dog" => tr!("Can't put the same card twice in the dog"),
            "play: Card neither in the taker's hand nor in the dog" => tr!("Card neither in the taker's hand nor in the dog"),
            "play: Can't put an oudler in the dog" => tr!("Can't put an oudler in the dog"),
            "play: Can't put a king in the dog" => tr!("Can't put a king in the dog"),
            "play: Can't put a trump in the dog" => tr!("Can't put a trump in the dog"),
            _ => error.to_string()
            };
            html! {
          <div class="notify-wrapper">
            <div class="error notify">
                <div>
                { error_str } 
                </div>
                <div class="toolbar">
                    <button class="btn-error" onclick=self.link.callback(|_| Msg::CloseError)>{"Ok"}</button>
                </div>
              </div>
            </div>
        }} else { html! {} }}

        { if self.overlay_box.is_some()  { html! {
          <div class="notify-wrapper">
            <div class="notify wrapper">
                { self.display_overlay_box() }
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
                        html! {<button class="primary" onclick=self.link.callback(|_| Msg::MarkReady)>{ tr!("Ready!")}</button>}
                    } else {
                        html! {}
                    }}
                        <button class="cancel" onclick=self.link.callback(|_| Msg::Disconnect)>{ tr!("Disconnect") }</button>
                    </div>
                    <h1>{{ tr!("join code:") }} <strong>{format!(" {}", format_join_code(&self.game_info.join_code))}</strong></h1>
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
                        <div style="width: 90vh;">
                            <CallKingAction
                                rank=rank
                                on_call_king=self.link.callback(|card| Msg::CallKing(card))
                                />
                        </div>
                    }
               },
               Turn::MakingDog => {
                   html! {
                       <div style="width: 90vh; text-align: center;">
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
                                let mut slam_classes = vec!["toggle"];
                                if self.slam_selected {
                                    slam_classes.push("toggle-selected");
                                }
                               html! {
                            <div class=slam_classes>
                               <button onclick=self.link.callback(move |_| Msg::MakeDog)>
                               {{ tr!("finish") }}
                               </button>
                                <input type="checkbox" id="slam"
                                    checked=self.slam_selected
                                    onclick=self.link.callback(move |_| Msg::ToggleSlam)
                                />
                                <label for="slam">{tr!("Slam") }</label>
                            </div>
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
                                    <div>
                                        <div class="yourturn"> {{ tr!("Your turn to play!") }} </div>
                                        { if self.is_first_trick() {
                                            html!{
                                            <Announces
                                                nb_players=self.game_state.players.len()
                                                hand=self.hand.clone()
                                                on_announce=self.link.callback(|announce| Msg::Announce(announce))
                                                />
                                            }
                                        } else { html!{} }}
                                    </div>
                                }
                            } else { html!{} }}
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
