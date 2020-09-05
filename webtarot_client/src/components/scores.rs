use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Clone, Properties)]
pub struct Props {
    // pub players: Vec<String>,
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

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Scores {
            players: props.players,
            scores: props.scores,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }


    fn view(&self) -> Html {
        let mut total = vec![0.0,0.0,0.0,0.0,0.0];
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
                    for self.players.iter().map(|nickname| {
                        html! {
                        <th> {nickname} </th>
                        }
                    })
                }
                </tr>
                { for self.scores.iter().map(|score| {
                    count = count + 1;
                    html! { <tr><td>{count}</td> {
                            for score.iter().map(|points| {
                                html! {
                                    <td> {points} </td>
                                }
                            })
                    } </tr> }
                 }) }
                <tr>
                    <th>{"Total"}</th>
                {
                    for total.iter().map(|points| {
                        html! {
                        <th> {points} </th>
                        }
                    })
                }
                </tr>
            </table>
        }
    }
}
