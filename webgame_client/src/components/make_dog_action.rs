use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback};

use tarotgame::cards;

pub enum Msg {
    AddToHand(cards::Card),
    AddToDog(cards::Card),
    MakeDog,
    Empty,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub dog: cards::Hand,
    pub hand: cards::Hand,
    pub on_make_dog: Callback<cards::Hand>,
}

pub struct MakeDogAction {
    link: ComponentLink<Self>,
    on_make_dog: Callback<cards::Hand>,
    dog: cards::Hand,
    hand: cards::Hand,
}

impl Component for MakeDogAction {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        MakeDogAction {
            link,
            on_make_dog: props.on_make_dog,
            dog: props.dog,
            hand: props.hand,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddToHand(card) => {
                self.hand.add(card);
                self.dog.remove(card);
            },
            Msg::AddToDog(card) => {
                self.dog.add(card);
                self.hand.remove(card);
            },
            Msg::MakeDog => {
                self.on_make_dog.emit(self.dog);
            },
            _ => {}
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.dog = props.dog;
        self.hand = props.hand;
        true
    }

    fn view(&self) -> Html {
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
            <section class="hand">
            {
                for self.hand.list().iter().map(|card| {
                    let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                    let clicked = card.clone();
                    html! {
                        <div class="card" style={style} onclick=self.link.callback(move |_| Msg::AddToDog(clicked))></div>
                    }
                })
            }
        </section>
        </div>
        }
    }
}
