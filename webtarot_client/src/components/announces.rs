use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, Callback};
use tr::tr;

use tarotgame::{Announce, AnnounceType, cards};

pub enum Msg {
    InitAnnounce(AnnounceType),
    MoveCard(cards::Card),
    Announce,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub nb_players: usize,
    pub hand: cards::Hand,
    pub on_announce: Callback<Announce>,
}

pub struct Announces {
    link: ComponentLink<Self>,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let proof = props.hand.trumps();
        Announces {
            link,
            nb_players: props.nb_players,
            on_announce: props.on_announce,
            announce_type: None,
            hand: props.hand,
            proof,
            keep: cards::Hand::new(),
            done: false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::InitAnnounce(ann_type) => {
                self.announce_type = Some(ann_type);
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
                    //TODO message bad number of cards
                }

            },
        }
        true
    }

    fn view(&self) -> Html {
        if self.announce_type.is_some() {
            html! {
               <div style="width: 90vh; text-align: center;">
                   <div class="hand">
                    {
                        for self.proof.list().iter().map(|card| {
                            let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                            let clicked = card.clone();
                            html! {
                                <div class="card" style={style} onclick=self.link.callback(move |_| Msg::MoveCard(clicked))></div>
                            }
                        })
                    }
                    </div>
                   <div class="hand">
                    {
                        for self.keep.list().iter().map(|card| {
                            let style =format!("--bg-image: url('cards/{}-{}.svg')", &card.rank().to_string(), &card.suit().to_safe_string());
                            let clicked = card.clone();
                            html! {
                                <div class="card" style={style} onclick=self.link.callback(move |_| Msg::MoveCard(clicked))></div>
                            }
                        })
                    }
                    </div>
                    <input type="button" onclick=self.link.callback(move |_| Msg::Announce) value={ tr!("Announce") } />
              </div>
            }
        } else if self.done { html! {} } else {
            let a_eligibles = AnnounceType::eligibles(self.hand);
            if a_eligibles.len() == 0 { html! {} }
            else {
                html! {
                  <div>
                  { for a_eligibles.into_iter().map(|ann_type| { html! {
                    <input type="button" onclick=self.link.callback(move |_| Msg::InitAnnounce(ann_type)) value={ tr!("{}", ann_type) } />
                  } }) }
                  </div>
                }
            }
        }
    }
}
