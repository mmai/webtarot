use std::rc::Rc;

use strum::IntoEnumIterator;
use yew::{html, Callback, Component, Context, Html, Properties};

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

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub struct BiddingActions {
    on_bid: Callback<(bid::Target, bool)>,
    on_pass: Callback<()>,
    game_state: Rc<GameStateSnapshot>,
    slam_selected: bool,
}

impl Component for BiddingActions {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        BiddingActions {
            game_state: ctx.props().game_state.clone(),
            on_bid: ctx.props().on_bid.clone(),
            on_pass: ctx.props().on_pass.clone(),
            slam_selected: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.game_state = ctx.props().game_state.clone();
        self.on_bid = ctx.props().on_bid.clone();
        self.on_pass = ctx.props().on_pass.clone();
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let curr_target = self.game_state.deal.contract_target();
        let mut slam_classes = vec!["toggle"];
        if self.slam_selected {
            slam_classes.push("toggle-selected");
        }

        html! {
            <section class="bidding">
                <button onclick={ctx.link().callback(move |_| Msg::Pass)}>
                { tr!("Passe") }
                </button>
                {
                    for bid::Target::iter()
                        .filter(|bidtarget| curr_target.lt(&Some(*bidtarget)))
                        .map(|bidtarget| {
                            html! {
                                <button onclick={ctx.link().callback(move |_| Msg::Bid(bidtarget))}>
                                    {format!("{}", bidtarget.to_str())}
                                </button>
                            }
                    })

                }
                <div class="toggle-wrapper">
                <div class={slam_classes.join(" ")}>
                    <input type="checkbox" id="slam" name="slam"
                        checked={self.slam_selected}
                        onclick={ctx.link().callback(move |_| Msg::ToggleSlam)}
                    />
                    <label for="slam">{ tr!("Slam") }</label>
                </div>
                </div>
            </section>
        }
    }
}
