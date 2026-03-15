use yew::{html, Callback, Component, Context, Html, Properties};

use tarotgame::cards;

pub enum Msg {
    CallKing(cards::Card),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub rank: cards::Rank,
    pub on_call_king: Callback<cards::Card>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub struct CallKingAction {
    on_call_king: Callback<cards::Card>,
    rank: cards::Rank,
}

impl Component for CallKingAction {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        CallKingAction {
            rank: ctx.props().rank,
            on_call_king: ctx.props().on_call_king.clone(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.rank = ctx.props().rank;
        self.on_call_king = ctx.props().on_call_king.clone();
        false
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CallKing(card) => {
                self.on_call_king.emit(card);
            },
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut kings = cards::Hand::new();
        kings.add(cards::Card::new(cards::Suit::Club, self.rank));
        kings.add(cards::Card::new(cards::Suit::Diamond, self.rank));
        kings.add(cards::Card::new(cards::Suit::Spade, self.rank));
        kings.add(cards::Card::new(cards::Suit::Heart, self.rank));
        html! {
            <div class="hand">
            {
                kings.list().iter().map(|card| {
                    let style = format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                    let clicked = card.clone();
                    html! {
                        <div class="card" style={style} onclick={ctx.link().callback(move |_| Msg::CallKing(clicked))}></div>
                    }
                }).collect::<Html>()
            }
            </div>
        }
    }
}
