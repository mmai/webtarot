use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback};

use tarotgame::cards;

pub enum Msg {
    CallKing(cards::Card),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub rank: cards::Rank,
    pub on_call_king: Callback<cards::Card>,
}

pub struct CallKingAction {
    link: ComponentLink<Self>,
    on_call_king: Callback<cards::Card>,
    rank: cards::Rank,
}

impl Component for CallKingAction {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        CallKingAction {
            link,
            rank: props.rank,
            on_call_king: props.on_call_king,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::CallKing(card) => {
                self.on_call_king.emit(card);
            },
        }
        false
    }

    fn view(&self) -> Html {
        let mut kings = cards::Hand::new();
        kings.add(cards::Card::new(cards::Suit::Club, self.rank));
        kings.add(cards::Card::new(cards::Suit::Diamond, self.rank));
        kings.add(cards::Card::new(cards::Suit::Spade, self.rank));
        kings.add(cards::Card::new(cards::Suit::Heart, self.rank));
        html! {
            <div class="hand">
        {
            for kings.list().iter().map(|card| {
                let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                let clicked = card.clone();
                html! {
                    <div class="card" style={style} onclick=self.link.callback(move |_| Msg::CallKing(clicked))></div>
                }
            })
        }
            </div>
        }
    }
}
