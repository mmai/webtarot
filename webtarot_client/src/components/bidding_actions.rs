use std::rc::Rc;

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback};

use tr::tr;

use crate::protocol::GameStateSnapshot;
use tarotgame::bid;

pub enum Msg {
    Bid(bid::Target),
    Pass,
    ToggleSlam,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
    pub on_bid: Callback<(bid::Target, bool)>,
    pub on_pass: Callback<()>,
}

pub struct BiddingActions {
    link: ComponentLink<Self>,
    on_bid: Callback<(bid::Target, bool)>,
    on_pass: Callback<()>,
    game_state: Rc<GameStateSnapshot>,
    slam_selected: bool,
}

impl Component for BiddingActions {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        BiddingActions {
            link,
            game_state: props.game_state,
            on_bid: props.on_bid,
            on_pass: props.on_pass,
            slam_selected: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleSlam => {
                self.slam_selected = !self.slam_selected;
                return true;
            },
            Msg::Bid(target) => {
                self.on_bid.emit((target, self.slam_selected));
            },
            Msg::Pass => {
                self.on_pass.emit(());
            },
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.game_state = props.game_state;
        true
    }

    fn view(&self) -> Html {
        let curr_target = self.game_state.deal.contract_target();
        let mut slam_classes = vec!["toggle"];
        if self.slam_selected {
            slam_classes.push("toggle-selected");
        }

        html! {
            <section class="bidding">
                <button onclick=self.link.callback(move |_| Msg::Pass)>
                { tr!("Passe") }
                </button>
                {
                    for bid::Target::iter()
                        .filter(|bidtarget| curr_target.lt(&Some(*bidtarget)))
                        .map(|bidtarget| {
                            html! {
                                <button onclick=self.link.callback(move |_| Msg::Bid(bidtarget))>
                                    {format!("{}", bidtarget.to_str())}
                                </button>
                            }
                    })

                }
                <div class="toggle-wrapper">
                <div class=slam_classes>
                    <input type="checkbox" id="slam" name="slam"
                        checked=self.slam_selected
                        onclick=self.link.callback(move |_| Msg::ToggleSlam)
                    />
                    <label for="slam">{ tr!("Slam") }</label>
                </div>
                </div>
            </section>
        }
    }
}
