use yew::{html, Callback, Component, Context, Html, Properties};
use tr::tr;
use weblog::*;

use tarotgame::{Announce, AnnounceType, cards};

pub enum Msg {
    InitAnnounce(AnnounceType),
    CancelAnnounce,
    MoveCard(cards::Card),
    Announce,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub nb_players: usize,
    pub hand: cards::Hand,
    pub on_announce: Callback<Announce>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub struct Announces {
    nb_players: usize,
    on_announce: Callback<Announce>,
    announce_type: Option<AnnounceType>,
    hand: cards::Hand,
    proof: cards::Hand,
    keep: cards::Hand,
    done: bool,
}

impl Component for Announces {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let proof = ctx.props().hand.trumps();
        Announces {
            nb_players: ctx.props().nb_players,
            on_announce: ctx.props().on_announce.clone(),
            announce_type: None,
            hand: ctx.props().hand,
            proof,
            keep: cards::Hand::new(),
            done: false,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.nb_players = ctx.props().nb_players;
        self.on_announce = ctx.props().on_announce.clone();
        false
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InitAnnounce(ann_type) => {
                self.announce_type = Some(ann_type);
            },
            Msg::CancelAnnounce => {
                self.announce_type = None;
                self.done = true;
            },
            Msg::MoveCard(card) => {
                if self.proof.has(card) {
                    self.proof.remove(card);
                    self.keep.add(card);
                } else if self.keep.has(card) {
                    self.keep.remove(card);
                    self.proof.add(card);
                }
            },
            Msg::Announce => {
                if let Some(announce_type) = &self.announce_type {
                    if self.proof.size() == announce_type.poignee_size(self.nb_players) {
                        self.on_announce.emit(Announce { atype: announce_type.clone(), proof: Some(self.proof) });
                        self.announce_type = None;
                        self.done = true;
                    }
                }

                if !self.done {
                    console_error!("bad number of cards");
                }
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(announce_type) = &self.announce_type {
            let proof_size = self.proof.size();
            let required_size = announce_type.poignee_size(self.nb_players);
            let is_valid_size = proof_size == required_size;
            let indications_classes = if is_valid_size { "indication-valid" } else { "indication-invalid" };
            html! {
               <div style="width: 90vh; text-align: center;">
                   <div>{ tr!("Select cards to show") }<span class={indications_classes}> { format!("({}/{})", proof_size, required_size)} </span></div>
                   <div class="hand">
                    {
                        self.hand.trumps().list().iter().map(|card| {
                            let style = format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                            let mut card_classes = vec!["card"];
                            if !self.proof.has(*card) {
                                card_classes.push("card-unselected");
                            }
                            let clicked = card.clone();
                            html! {
                                <div class={card_classes.join(" ")} style={style} onclick={ctx.link().callback(move |_| Msg::MoveCard(clicked))}><div></div></div>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                    { if is_valid_size {
                        html!{
                    <button onclick={ctx.link().callback(move |_| Msg::Announce)}>{ tr!("Announce") }</button>
                        }
                    } else { html!{} }
                    }
              </div>
            }
        } else if self.done { html! {} } else {
            let a_eligibles = AnnounceType::eligibles(self.hand);
            if a_eligibles.len() == 0 { html! {} }
            else {
                html! {
                  <div>
                    <button onclick={ctx.link().callback(move |_| Msg::CancelAnnounce)}>{ tr!("No announce") }</button>
                  { a_eligibles.into_iter().map(|ann_type| { html! {
                    <button onclick={ctx.link().callback(move |_| Msg::InitAnnounce(ann_type))}> { tr!("{}", ann_type) }</button>
                  } }).collect::<Html>() }
                  </div>
                }
            }
        }
    }
}
