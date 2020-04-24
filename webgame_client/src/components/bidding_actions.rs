use std::rc::Rc;
use std::str::FromStr;

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback, ChangeData};

use crate::protocol::{GameStateSnapshot, PlayerRole, GamePlayerState, Turn};
use tarotgame::{bid, cards};

pub enum Msg {
    SelectTrump(String),
    Bid(bid::Target),
    Surcoinche,
    Empty,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
    pub on_bid: Callback<(bid::Target, cards::Suit)>,
    pub on_surcoinche: Callback<()>,
}

pub struct BiddingActions {
    link: ComponentLink<Self>,
    on_bid: Callback<(bid::Target, cards::Suit)>,
    on_surcoinche: Callback<()>,
    game_state: Rc<GameStateSnapshot>,
    selected_trump: cards::Suit,
}

impl Component for BiddingActions {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        BiddingActions {
            link,
            game_state: props.game_state,
            on_bid: props.on_bid,
            on_surcoinche: props.on_surcoinche,
            selected_trump: cards::Suit::Heart,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SelectTrump(suit_str) => {
                self.selected_trump = cards::Suit::from_str(&*suit_str).unwrap();
            },
            Msg::Bid(target) => {
                self.on_bid.emit((target, self.selected_trump));
            },
            Msg::Surcoinche => {
                self.on_surcoinche.emit(());
            },
            _ => {}
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.game_state = props.game_state;
        true
    }

    fn view(&self) -> Html {
        let curr_target = self.game_state.deal.contract_target();
        // let curr_trump = self.game_state.deal.contract_trump();
        let curr_coinche = self.game_state.deal.contract_coinche();
        let trumps = vec![
            cards::Suit::Heart,
            cards::Suit::Spade,
            cards::Suit::Diamond,
            cards::Suit::Club
        ];
        html! {
            <section class="bidding">
                <select name="trump"
                        onchange=self.link.callback(move |data| {
                            if let ChangeData::Select(sel_value) = data {
                                Msg::SelectTrump(sel_value.value())
                            } else { Msg::Empty }
                        })>
                {
                    for trumps.iter().map( |t| {
                            html! {
                                <option>
                                {format!("{}", t.to_string())}
                                </option>
                            }
                    })
                }
                </select>
                {
                    for bid::Target::iter()
                        .filter(|bidtarget| !curr_target.lt(&Some(*bidtarget)))
                        .map(|bidtarget| {
                            html! {
                                <button onclick=self.link.callback(move |_| Msg::Bid(bidtarget))>
                                    {format!("{}", bidtarget.to_str())}
                                </button>
                            }
                    })

                }
                {
                if curr_coinche == 1 {
                    html! {
                        <button onclick=self.link.callback(move |_| Msg::Surcoinche)>
                            {"Surcoinche"}
                        </button>
                    }
                } else { html! {} }
                }
            </section>
        }
    }
}
