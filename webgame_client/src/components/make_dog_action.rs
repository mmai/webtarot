use std::rc::Rc;

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback};

use crate::protocol::GameStateSnapshot;
use tarotgame::cards;

pub enum Msg {
    MakeDog(cards::Hand),
    Empty,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
    pub on_make_dog: Callback<cards::Hand>,
}

pub struct MakeDogAction {
    link: ComponentLink<Self>,
    on_make_dog: Callback<cards::Hand>,
    game_state: Rc<GameStateSnapshot>,
}

impl Component for MakeDogAction {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        MakeDogAction {
            link,
            game_state: props.game_state,
            on_make_dog: props.on_make_dog,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::MakeDog(cards) => {
                self.on_make_dog.emit(cards);
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
        html! {
            <section class="make_dog">
            {{ "Making dog" }}
            </section>
        }
    }
}
