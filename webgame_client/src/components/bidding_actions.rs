use std::rc::Rc;

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GameStateSnapshot, PlayerRole, GamePlayerState, Turn};
use tarotgame::bid;

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
}

pub struct BiddingActions {
    game_state: Rc<GameStateSnapshot>,
}

impl Component for BiddingActions {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        BiddingActions {
            game_state: props.game_state,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.game_state = props.game_state;
        true
    }

    fn view(&self) -> Html {
        html! {
            <section class="bidding">
                {
                    for bid::Target::iter().map(|bidtarget| {
                      html! {
                        <div>
                        {format!("{}", bidtarget.to_str())}
                        </div>
                      }
                    })
                }
            </section>
        }
    }
}
