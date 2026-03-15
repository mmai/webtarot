use yew::{html, Component, Context, Html, Properties};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub players: Vec<String>,
    pub scores: Vec<Vec<f32>>,
}

pub struct Scores {
    players: Vec<String>,
    scores: Vec<Vec<f32>>,
}

impl Component for Scores {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Scores {
            players: ctx.props().players.clone(),
            scores: ctx.props().scores.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.players = ctx.props().players.clone();
        self.scores = ctx.props().scores.clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut total = vec![0.0; self.players.len()];
        for score in self.scores.iter() {
            for (idx, points) in score.iter().enumerate() {
                total[idx] = total[idx] + points;
            }
        }

        let mut count = 0;
        html! {
            <table class="scores">
                <tr>
                <th></th>
                {
                    self.players.iter().map(|nickname| {
                        html! {
                        <th> {nickname} </th>
                        }
                    }).collect::<Html>()
                }
                </tr>
                { self.scores.iter().map(|score| {
                    count = count + 1;
                    html! { <tr><td>{count}</td> {
                            score.iter().map(|points| {
                                html! {
                                    <td> {points} </td>
                                }
                            }).collect::<Html>()
                    } </tr> }
                 }).collect::<Html>() }
                <tr>
                    <th>{"Total"}</th>
                {
                    total.iter().map(|points| {
                        html! {
                        <th> {points} </th>
                        }
                    }).collect::<Html>()
                }
                </tr>
            </table>
        }
    }
}
